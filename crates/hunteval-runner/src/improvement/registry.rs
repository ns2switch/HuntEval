use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use hunteval_domain::{
    ArtifactKind, ArtifactMediaType, ArtifactProvenance, ArtifactRegistry, RegisteredArtifact,
    SchemaVersion, Sha256Digest,
};
use thiserror::Error;

const MAX_ARTIFACT_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ArtifactRegistrationRequest<'a> {
    pub id: &'a str,
    pub kind: ArtifactKind,
    pub media_type: ArtifactMediaType,
    pub label: &'a str,
    pub provenance: ArtifactProvenance,
    pub structured_artifact_sha256: Option<Sha256Digest>,
}

pub fn register_artifact(
    public_root: &Path,
    relative_source: &Path,
    registry_root: &Path,
    request: &ArtifactRegistrationRequest<'_>,
) -> Result<RegisteredArtifact, ArtifactRegistryError> {
    validate_relative(relative_source)?;
    let source = public_root.join(relative_source);
    let metadata =
        fs::symlink_metadata(&source).map_err(|_| ArtifactRegistryError::UnsafeSource)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_ARTIFACT_BYTES
    {
        return Err(ArtifactRegistryError::UnsafeSource);
    }
    #[cfg(unix)]
    if metadata.nlink() != 1 {
        return Err(ArtifactRegistryError::UnsafeSource);
    }
    let bytes = fs::read(&source).map_err(|_| ArtifactRegistryError::Io)?;
    if requires_utf8(request.media_type) && std::str::from_utf8(&bytes).is_err() {
        return Err(ArtifactRegistryError::InvalidEncoding);
    }
    let digest = Sha256Digest::from_bytes(&bytes);
    fs::create_dir_all(registry_root).map_err(|_| ArtifactRegistryError::Io)?;
    let stored_path = stored_path(registry_root, digest);
    persist_once(&stored_path, &bytes)?;
    let artifact = RegisteredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: request.id.to_owned(),
        kind: request.kind,
        media_type: request.media_type,
        sha256: digest,
        size_bytes: metadata.len(),
        label: request.label.to_owned(),
        provenance: request.provenance,
        structured_artifact_sha256: request.structured_artifact_sha256,
    };
    artifact
        .validate()
        .map_err(|_| ArtifactRegistryError::InvalidContract)?;
    Ok(artifact)
}

pub fn verify_registered_artifact(
    registry_root: &Path,
    artifact: &RegisteredArtifact,
) -> Result<(), ArtifactRegistryError> {
    artifact
        .validate()
        .map_err(|_| ArtifactRegistryError::InvalidContract)?;
    let path = stored_path(registry_root, artifact.sha256);
    let metadata =
        fs::symlink_metadata(&path).map_err(|_| ArtifactRegistryError::MissingArtifact)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() != artifact.size_bytes
    {
        return Err(ArtifactRegistryError::DigestMismatch);
    }
    let bytes = fs::read(path).map_err(|_| ArtifactRegistryError::Io)?;
    if Sha256Digest::from_bytes(bytes) != artifact.sha256 {
        return Err(ArtifactRegistryError::DigestMismatch);
    }
    Ok(())
}

pub fn write_artifact_registry(
    registry_root: &Path,
    id: &str,
    mut artifacts: Vec<RegisteredArtifact>,
) -> Result<Sha256Digest, ArtifactRegistryError> {
    artifacts.sort_by(|left, right| left.id.cmp(&right.id));
    let registry = ArtifactRegistry {
        schema_version: SchemaVersion::new(0, 8),
        id: id.to_owned(),
        artifacts,
    };
    registry
        .validate()
        .map_err(|_| ArtifactRegistryError::InvalidContract)?;
    let mut bytes =
        serde_json::to_vec_pretty(&registry).map_err(|_| ArtifactRegistryError::InvalidContract)?;
    bytes.push(b'\n');
    let path = registry_root.join("artifact-registry.json");
    persist_once(&path, &bytes)?;
    Ok(Sha256Digest::from_bytes(bytes))
}

fn stored_path(root: &Path, digest: Sha256Digest) -> PathBuf {
    root.join("artifacts").join(digest.to_string())
}

fn persist_once(path: &Path, bytes: &[u8]) -> Result<(), ArtifactRegistryError> {
    let parent = path.parent().ok_or(ArtifactRegistryError::Io)?;
    fs::create_dir_all(parent).map_err(|_| ArtifactRegistryError::Io)?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            file.write_all(bytes)
                .map_err(|_| ArtifactRegistryError::Io)?;
            file.sync_all().map_err(|_| ArtifactRegistryError::Io)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let stored = fs::read(path).map_err(|_| ArtifactRegistryError::Io)?;
            if stored == bytes {
                Ok(())
            } else {
                Err(ArtifactRegistryError::DigestMismatch)
            }
        }
        Err(_) => Err(ArtifactRegistryError::Io),
    }
}

fn validate_relative(path: &Path) -> Result<(), ArtifactRegistryError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            !matches!(component, Component::Normal(_) | Component::CurDir)
                || component == Component::CurDir
        })
    {
        return Err(ArtifactRegistryError::UnsafeSource);
    }
    Ok(())
}

const fn requires_utf8(media_type: ArtifactMediaType) -> bool {
    matches!(
        media_type,
        ArtifactMediaType::Json
            | ArtifactMediaType::Yaml
            | ArtifactMediaType::Markdown
            | ArtifactMediaType::Text
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ArtifactRegistryError {
    #[error("artifact source is not a bounded unlinked regular file")]
    UnsafeSource,
    #[error("artifact bytes are not valid for the declared media type")]
    InvalidEncoding,
    #[error("registered artifact contract is invalid")]
    InvalidContract,
    #[error("registered artifact is missing")]
    MissingArtifact,
    #[error("registered artifact bytes do not match their identity")]
    DigestMismatch,
    #[error("artifact registry I/O failed")]
    Io,
}
