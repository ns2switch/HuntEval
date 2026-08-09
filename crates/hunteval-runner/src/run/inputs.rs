use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use hunteval_domain::{SchemaVersion, Sha256Digest};
use hunteval_evaluation::{ScoringProfile, ScoringProfileArtifact, normalize_profile};
use serde::Deserialize;
use thiserror::Error;

use crate::{EpisodeLoadError, EpisodePackage, hash_file};

const MAX_SCORING_PROFILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeploymentManifest {
    schema_version: SchemaVersion,
    id: String,
    kind: String,
    architecture: String,
    agents: Vec<serde_yaml_ng::Value>,
    network_access: bool,
    scored_tools: String,
    process: DeploymentProcess,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeploymentProcess {
    executable: String,
    #[serde(default)]
    arguments: Vec<String>,
    #[serde(default)]
    environment_allowlist: Vec<String>,
}

/// All mutable filesystem inputs resolved and hashed before process launch.
#[derive(Debug)]
pub struct ResolvedRunInputs {
    pub episode: EpisodePackage,
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub scoring_profile: ScoringProfile,
    pub hashes: BTreeMap<String, Sha256Digest>,
}

impl ResolvedRunInputs {
    pub fn resolve(
        episode_root: &Path,
        deployment_manifest: &Path,
        scoring_profile: &Path,
        schema_contract: &Path,
    ) -> Result<Self, RunInputError> {
        Self::resolve_with_executable(
            episode_root,
            deployment_manifest,
            scoring_profile,
            schema_contract,
            None,
        )
    }

    /// Resolves inputs while allowing a trusted controller to supply the built deployment binary.
    pub fn resolve_with_executable(
        episode_root: &Path,
        deployment_manifest: &Path,
        scoring_profile: &Path,
        schema_contract: &Path,
        executable_override: Option<&Path>,
    ) -> Result<Self, RunInputError> {
        let episode = EpisodePackage::load(episode_root)?;
        let deployment_bytes = read_regular(deployment_manifest)?;
        let deployment: DeploymentManifest = serde_yaml_ng::from_slice(&deployment_bytes)
            .map_err(|_| RunInputError::InvalidDeployment)?;
        validate_deployment(&deployment)?;
        let deployment_root = deployment_manifest
            .parent()
            .ok_or(RunInputError::UnsafePath)?;
        let executable = match executable_override {
            Some(path) => resolve_override(path)?,
            None => resolve_regular(deployment_root, &deployment.process.executable)?,
        };
        let scoring_bytes = read_scoring_profile(scoring_profile)?;
        let scoring_profile = parse_scoring_profile(&scoring_bytes)?;
        let schema_bytes = read_regular(schema_contract)?;
        let environment = deployment
            .process
            .environment_allowlist
            .iter()
            .filter_map(|name| std::env::var(name).ok().map(|value| (name.clone(), value)))
            .collect::<BTreeMap<_, _>>();

        let mut hashes = BTreeMap::from([
            (
                "deployment_configuration".to_owned(),
                Sha256Digest::from_bytes(deployment_bytes),
            ),
            ("deployment_executable".to_owned(), hash_file(&executable)?),
            (
                "episode_package".to_owned(),
                episode.digests().package_index,
            ),
            (
                "episode_manifest".to_owned(),
                episode.digests().public_manifest,
            ),
            (
                "ground_truth".to_owned(),
                episode.digests().private_ground_truth,
            ),
            (
                "protocol".to_owned(),
                Sha256Digest::from_bytes(b"hunteval-jsonl-0.3"),
            ),
            ("schema".to_owned(), Sha256Digest::from_bytes(schema_bytes)),
            (
                "scoring_profile".to_owned(),
                Sha256Digest::from_bytes(scoring_bytes),
            ),
            (
                "runner_binary".to_owned(),
                hash_file(&std::env::current_exe().map_err(RunInputError::Io)?)?,
            ),
            (
                "sandbox_backend".to_owned(),
                hash_file(hunteval_sandbox::backend_executable())?,
            ),
            (
                "resource_launcher".to_owned(),
                hash_file(hunteval_sandbox::resource_launcher_executable())?,
            ),
        ]);
        for (table, digest) in &episode.digests().public_telemetry {
            hashes.insert(format!("dataset:{table}"), *digest);
        }
        Ok(Self {
            episode,
            executable,
            arguments: deployment.process.arguments,
            environment,
            scoring_profile,
            hashes,
        })
    }
}

/// Loads and explicitly normalizes a v0.3 or v0.4 scoring profile.
pub fn load_scoring_profile(path: &Path) -> Result<ScoringProfile, RunInputError> {
    parse_scoring_profile(&read_scoring_profile(path)?)
}

fn parse_scoring_profile(bytes: &[u8]) -> Result<ScoringProfile, RunInputError> {
    let artifact: ScoringProfileArtifact =
        serde_yaml_ng::from_slice(bytes).map_err(|_| RunInputError::InvalidScoringProfile)?;
    normalize_profile(artifact).map_err(|_| RunInputError::InvalidScoringProfile)
}

fn resolve_override(path: &Path) -> Result<PathBuf, RunInputError> {
    let metadata = fs::symlink_metadata(path).map_err(RunInputError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RunInputError::UnsafePath);
    }
    fs::canonicalize(path).map_err(RunInputError::Io)
}

fn validate_deployment(manifest: &DeploymentManifest) -> Result<(), RunInputError> {
    if manifest.schema_version != SchemaVersion::new(0, 4)
        || manifest.id.trim().is_empty()
        || manifest.kind != "external_reference_process"
        || manifest.architecture.trim().is_empty()
        || manifest.agents.is_empty()
        || manifest.network_access
        || manifest.scored_tools != "hunteval_managed_only"
        || manifest
            .process
            .arguments
            .iter()
            .any(|value| value.contains('\0'))
        || manifest.process.environment_allowlist.iter().any(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        })
    {
        return Err(RunInputError::InvalidDeployment);
    }
    Ok(())
}

fn read_regular(path: &Path) -> Result<Vec<u8>, RunInputError> {
    let metadata = fs::symlink_metadata(path).map_err(RunInputError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RunInputError::UnsafePath);
    }
    fs::read(path).map_err(RunInputError::Io)
}

fn read_scoring_profile(path: &Path) -> Result<Vec<u8>, RunInputError> {
    let metadata = fs::symlink_metadata(path).map_err(RunInputError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RunInputError::UnsafePath);
    }
    if metadata.len() > MAX_SCORING_PROFILE_BYTES {
        return Err(RunInputError::InvalidScoringProfile);
    }
    let file = fs::File::open(path).map_err(RunInputError::Io)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_SCORING_PROFILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(RunInputError::Io)?;
    if bytes.len() as u64 > MAX_SCORING_PROFILE_BYTES {
        return Err(RunInputError::InvalidScoringProfile);
    }
    Ok(bytes)
}

fn resolve_regular(root: &Path, relative: &str) -> Result<PathBuf, RunInputError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(RunInputError::UnsafePath);
    }
    let root = fs::canonicalize(root).map_err(RunInputError::Io)?;
    let candidate = root.join(relative);
    let metadata = fs::symlink_metadata(&candidate).map_err(RunInputError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RunInputError::UnsafePath);
    }
    let resolved = fs::canonicalize(candidate).map_err(RunInputError::Io)?;
    if !resolved.starts_with(root) {
        return Err(RunInputError::UnsafePath);
    }
    Ok(resolved)
}

#[derive(Debug, Error)]
pub enum RunInputError {
    #[error("episode input is invalid: {0}")]
    Episode(#[from] EpisodeLoadError),
    #[error("run input I/O failed")]
    Io(#[source] std::io::Error),
    #[error("deployment configuration is invalid")]
    InvalidDeployment,
    #[error("scoring profile is invalid")]
    InvalidScoringProfile,
    #[error("run input contains an unsafe path or symlink")]
    UnsafePath,
    #[error("run input hashing failed: {0}")]
    Hashing(#[from] crate::HashingError),
}
