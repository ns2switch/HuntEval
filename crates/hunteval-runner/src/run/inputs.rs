use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use hunteval_domain::{SchemaVersion, Sha256Digest};
use hunteval_evaluation::ScoringProfile;
use serde::Deserialize;
use thiserror::Error;

use crate::{EpisodeLoadError, EpisodePackage, hash_file};

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
        let episode = EpisodePackage::load(episode_root)?;
        let deployment_bytes = read_regular(deployment_manifest)?;
        let deployment: DeploymentManifest = serde_yaml_ng::from_slice(&deployment_bytes)
            .map_err(|_| RunInputError::InvalidDeployment)?;
        validate_deployment(&deployment)?;
        let deployment_root = deployment_manifest
            .parent()
            .ok_or(RunInputError::UnsafePath)?;
        let executable = resolve_regular(deployment_root, &deployment.process.executable)?;
        let scoring_bytes = read_regular(scoring_profile)?;
        let scoring_profile: ScoringProfile = serde_yaml_ng::from_slice(&scoring_bytes)
            .map_err(|_| RunInputError::InvalidScoringProfile)?;
        validate_scoring_profile(&scoring_profile)?;
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

fn validate_scoring_profile(profile: &ScoringProfile) -> Result<(), RunInputError> {
    const METRICS: &[&str] = &[
        "entity_precision",
        "entity_recall",
        "event_precision",
        "event_recall",
        "evidence_grounding",
        "provenance_validity",
        "resilience",
        "task_completion",
        "tool_call_utilization",
    ];
    let total = profile.weights.values().sum::<f64>();
    if profile.schema_version != SchemaVersion::new(0, 3)
        || profile.id.trim().is_empty()
        || profile.weights.is_empty()
        || profile.weights.iter().any(|(name, weight)| {
            !METRICS.contains(&name.as_str()) || !weight.is_finite() || *weight < 0.0
        })
        || (total - 1.0).abs() > 1e-9
    {
        return Err(RunInputError::InvalidScoringProfile);
    }
    Ok(())
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
