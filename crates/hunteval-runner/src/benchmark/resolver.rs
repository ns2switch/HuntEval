use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use hunteval_domain::{
    BenchmarkDefinition, EpisodeId, FaultProfileId, ResolvedArtifact, ResolvedDeployment,
    ResolvedEpisode, ScoringProfileId, Sha256Digest,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{AuthoredBenchmarkManifest, BenchmarkError, load_benchmark};

const MAX_ARTIFACT_FILES: usize = 100_000;
const MAX_ARTIFACT_BYTES: u64 = 8 * 1024 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct ArtifactDescriptor {
    id: Option<String>,
    episode_id: Option<String>,
}

/// Loads an authored manifest and resolves it into infrastructure-independent identities.
pub fn resolve_benchmark(
    manifest_path: &Path,
    artifact_root: &Path,
) -> Result<BenchmarkDefinition, BenchmarkError> {
    let manifest = load_benchmark(manifest_path)?;
    resolve_manifest(&manifest, artifact_root)
}

fn resolve_manifest(
    manifest: &AuthoredBenchmarkManifest,
    artifact_root: &Path,
) -> Result<BenchmarkDefinition, BenchmarkError> {
    manifest.validate()?;
    let root = artifact_root.canonicalize()?;
    let mut deployments = Vec::with_capacity(manifest.deployments.len());
    for reference in &manifest.deployments {
        let path = resolve_path(&root, reference)?;
        let descriptor = descriptor_path(&path, "deployment.yaml")?;
        deployments.push(ResolvedDeployment {
            configuration_sha256: hash_artifact(&path)?,
            id: read_descriptor_id(&descriptor, false)?
                .parse()
                .map_err(|_| BenchmarkError::InvalidDescriptor)?,
        });
    }

    let mut episodes = Vec::with_capacity(manifest.episodes.len());
    for reference in &manifest.episodes {
        let path = resolve_path(&root, reference)?;
        let descriptor = descriptor_path(&path, "package.yaml")?;
        episodes.push(ResolvedEpisode {
            id: read_descriptor_id(&descriptor, true)?
                .parse::<EpisodeId>()
                .map_err(|_| BenchmarkError::InvalidDescriptor)?,
            package_sha256: hash_artifact(&path)?,
        });
    }

    let scoring_path = resolve_path(&root, &manifest.scoring_profile)?;
    let scoring_profile = ResolvedArtifact {
        id: read_descriptor_id(&scoring_path, false)?
            .parse::<ScoringProfileId>()
            .map_err(|_| BenchmarkError::InvalidDescriptor)?,
        sha256: hash_artifact(&scoring_path)?,
    };
    let fault_profile = manifest
        .fault_profile
        .as_ref()
        .map(
            |reference| -> Result<ResolvedArtifact<FaultProfileId>, BenchmarkError> {
                let path = resolve_path(&root, reference)?;
                Ok(ResolvedArtifact {
                    id: read_descriptor_id(&path, false)?
                        .parse::<FaultProfileId>()
                        .map_err(|_| BenchmarkError::InvalidDescriptor)?,
                    sha256: hash_artifact(&path)?,
                })
            },
        )
        .transpose()?;

    BenchmarkDefinition::new(
        manifest.id.clone(),
        deployments,
        episodes,
        manifest.seeds.clone(),
        scoring_profile,
        fault_profile,
    )
    .map_err(BenchmarkError::from)
}

fn resolve_path(root: &Path, reference: &str) -> Result<PathBuf, BenchmarkError> {
    let relative = Path::new(reference);
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component);
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(BenchmarkError::SymlinkArtifact);
        }
    }
    let canonical = current.canonicalize()?;
    if !canonical.starts_with(root) {
        return Err(BenchmarkError::ArtifactOutsideRoot);
    }
    Ok(canonical)
}

fn descriptor_path(path: &Path, file_name: &str) -> Result<PathBuf, BenchmarkError> {
    if path.is_dir() {
        let descriptor = path.join(file_name);
        if fs::symlink_metadata(&descriptor)?.file_type().is_symlink() {
            return Err(BenchmarkError::SymlinkArtifact);
        }
        Ok(descriptor)
    } else {
        Ok(path.to_path_buf())
    }
}

fn read_descriptor_id(path: &Path, episode: bool) -> Result<String, BenchmarkError> {
    let bytes = fs::read(path)?;
    let descriptor: ArtifactDescriptor =
        serde_yaml_ng::from_slice(&bytes).map_err(|_| BenchmarkError::InvalidDescriptor)?;
    let id = if episode {
        descriptor.episode_id
    } else {
        descriptor.id
    };
    id.filter(|value| !value.is_empty())
        .ok_or(BenchmarkError::InvalidDescriptor)
}

fn hash_artifact(path: &Path) -> Result<Sha256Digest, BenchmarkError> {
    let mut files = Vec::new();
    collect_files(path, path, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    if files.is_empty() || files.len() > MAX_ARTIFACT_FILES {
        return Err(BenchmarkError::ArtifactLimit);
    }

    let mut total = 0_u64;
    let mut hasher = Sha256::new();
    for (relative, file) in files {
        let metadata = fs::metadata(&file)?;
        total = total
            .checked_add(metadata.len())
            .ok_or(BenchmarkError::ArtifactLimit)?;
        if total > MAX_ARTIFACT_BYTES {
            return Err(BenchmarkError::ArtifactLimit);
        }
        let relative = relative
            .to_str()
            .ok_or(BenchmarkError::InvalidDescriptor)?
            .as_bytes();
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative);
        hasher.update(metadata.len().to_be_bytes());
        hash_file(&file, metadata.len(), &mut hasher)?;
    }
    hex::encode(hasher.finalize())
        .parse()
        .map_err(|_: hunteval_domain::DigestParseError| BenchmarkError::InvalidDescriptor)
}

fn collect_files(
    root: &Path,
    path: &Path,
    files: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), BenchmarkError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(BenchmarkError::SymlinkArtifact);
    }
    if metadata.is_file() {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| BenchmarkError::ArtifactOutsideRoot)?;
        let relative = if relative.as_os_str().is_empty() {
            path.file_name()
                .map(PathBuf::from)
                .ok_or(BenchmarkError::InvalidDescriptor)?
        } else {
            relative.to_path_buf()
        };
        files.push((relative, path.to_path_buf()));
        if files.len() > MAX_ARTIFACT_FILES {
            return Err(BenchmarkError::ArtifactLimit);
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(BenchmarkError::InvalidDescriptor);
    }
    for entry in fs::read_dir(path)? {
        collect_files(root, &entry?.path(), files)?;
    }
    Ok(())
}

fn hash_file(path: &Path, expected_bytes: u64, hasher: &mut Sha256) -> Result<(), BenchmarkError> {
    let mut file = File::open(path)?;
    let opened = file.metadata()?;
    if !opened.is_file() || opened.len() != expected_bytes {
        return Err(BenchmarkError::InvalidDescriptor);
    }
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes_read = 0_u64;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            return if bytes_read == expected_bytes {
                Ok(())
            } else {
                Err(BenchmarkError::InvalidDescriptor)
            };
        }
        let count_u64 = u64::try_from(count).map_err(|_| BenchmarkError::ArtifactLimit)?;
        bytes_read = bytes_read
            .checked_add(count_u64)
            .ok_or(BenchmarkError::ArtifactLimit)?;
        if bytes_read > expected_bytes {
            return Err(BenchmarkError::InvalidDescriptor);
        }
        hasher.update(&buffer[..count]);
    }
}
