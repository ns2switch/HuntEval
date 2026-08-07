use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use hunteval_domain::{EpisodeId, EpisodeManifest, GroundTruth, SchemaVersion, Sha256Digest};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageIndex {
    schema_version: SchemaVersion,
    episode_id: EpisodeId,
    public_root: String,
    private_ground_truth: String,
}

/// Hashes recorded by the trusted runner for reproducibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDigests {
    pub package_index: Sha256Digest,
    pub public_manifest: Sha256Digest,
    pub public_telemetry: BTreeMap<String, Sha256Digest>,
    pub private_ground_truth: Sha256Digest,
}

/// Deployment-safe view of an episode package.
#[derive(Debug, Clone)]
pub struct PublicEpisodePackage {
    pub manifest: EpisodeManifest,
    pub public_root: PathBuf,
    pub manifest_sha256: Sha256Digest,
    pub telemetry_sha256: BTreeMap<String, Sha256Digest>,
}

/// Trusted package containing physically separated public and private values.
#[derive(Debug)]
pub struct EpisodePackage {
    public: PublicEpisodePackage,
    ground_truth: GroundTruth,
    digests: ArtifactDigests,
}

impl EpisodePackage {
    /// Loads and validates one package without following unexpected symlinks.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, EpisodeLoadError> {
        let root = root.as_ref();
        reject_symlink(root)?;
        let root = fs::canonicalize(root).map_err(EpisodeLoadError::Io)?;
        let package_path = resolve_existing(&root, "package.yaml")?;
        let package_bytes = fs::read(&package_path).map_err(EpisodeLoadError::Io)?;
        let package: PackageIndex = serde_yaml_ng::from_slice(&package_bytes)
            .map_err(|_| EpisodeLoadError::InvalidPackageIndex)?;
        if !supported_schema(package.schema_version) {
            return Err(EpisodeLoadError::UnsupportedSchema);
        }

        let public_root = resolve_existing(&root, &package.public_root)?;
        let ground_truth_path = resolve_existing(&root, &package.private_ground_truth)?;
        if !public_root.is_dir() || !ground_truth_path.is_file() {
            return Err(EpisodeLoadError::InvalidPackageIndex);
        }
        if ground_truth_path.starts_with(&public_root) {
            return Err(EpisodeLoadError::PrivateRootExposed);
        }
        let manifest_path = resolve_existing(&public_root, "manifest.yaml")?;
        let manifest_bytes = fs::read(&manifest_path).map_err(EpisodeLoadError::Io)?;
        let manifest: EpisodeManifest = serde_yaml_ng::from_slice(&manifest_bytes)
            .map_err(|_| EpisodeLoadError::InvalidPublicManifest)?;
        manifest
            .validate()
            .map_err(|_| EpisodeLoadError::InvalidPublicManifest)?;
        if manifest.id != package.episode_id || manifest.schema_version != package.schema_version {
            return Err(EpisodeLoadError::EpisodeIdMismatch);
        }

        let ground_truth_bytes = fs::read(&ground_truth_path).map_err(EpisodeLoadError::Io)?;
        let ground_truth: GroundTruth = serde_json::from_slice(&ground_truth_bytes)
            .map_err(|_| EpisodeLoadError::InvalidGroundTruth)?;
        if !supported_schema(ground_truth.schema_version) {
            return Err(EpisodeLoadError::UnsupportedSchema);
        }
        ground_truth
            .validate()
            .map_err(|_| EpisodeLoadError::InvalidGroundTruth)?;
        if ground_truth.episode_id != package.episode_id
            || ground_truth.schema_version != package.schema_version
        {
            return Err(EpisodeLoadError::EpisodeIdMismatch);
        }

        let mut telemetry_sha256 = BTreeMap::new();
        for table in &manifest.telemetry.tables {
            let path = resolve_existing(&public_root, &table.path)?;
            if !path.is_file() {
                return Err(EpisodeLoadError::InvalidPublicArtifact);
            }
            let bytes = fs::read(path).map_err(EpisodeLoadError::Io)?;
            telemetry_sha256.insert(table.name.clone(), Sha256Digest::from_bytes(bytes));
        }
        for document in &manifest.knowledge.documents {
            let path = resolve_existing(&public_root, document)?;
            if !path.is_file() {
                return Err(EpisodeLoadError::InvalidPublicArtifact);
            }
        }

        let package_index = Sha256Digest::from_bytes(package_bytes);
        let public_manifest = Sha256Digest::from_bytes(manifest_bytes);
        let private_ground_truth = Sha256Digest::from_bytes(ground_truth_bytes);
        let public = PublicEpisodePackage {
            manifest,
            public_root,
            manifest_sha256: public_manifest,
            telemetry_sha256: telemetry_sha256.clone(),
        };
        let digests = ArtifactDigests {
            package_index,
            public_manifest,
            public_telemetry: telemetry_sha256,
            private_ground_truth,
        };
        Ok(Self {
            public,
            ground_truth,
            digests,
        })
    }

    /// Returns the only package view that may be shared with deployment adapters.
    #[must_use]
    pub const fn public(&self) -> &PublicEpisodePackage {
        &self.public
    }

    /// Returns evaluator-only ground truth inside the trusted runner boundary.
    #[must_use]
    pub const fn ground_truth(&self) -> &GroundTruth {
        &self.ground_truth
    }

    /// Returns trusted hashes for run metadata and reproducibility.
    #[must_use]
    pub const fn digests(&self) -> &ArtifactDigests {
        &self.digests
    }
}

const fn supported_schema(version: SchemaVersion) -> bool {
    version.major() == 0 && matches!(version.minor(), 3 | 4)
}

fn resolve_existing(root: &Path, relative: &str) -> Result<PathBuf, EpisodeLoadError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(EpisodeLoadError::UnsafePath);
    }

    let mut candidate = root.to_path_buf();
    for component in relative.components() {
        if let Component::Normal(value) = component {
            candidate.push(value);
            reject_symlink(&candidate)?;
        }
    }
    let resolved = fs::canonicalize(candidate).map_err(EpisodeLoadError::Io)?;
    if !resolved.starts_with(root) {
        return Err(EpisodeLoadError::UnsafePath);
    }
    Ok(resolved)
}

fn reject_symlink(path: &Path) -> Result<(), EpisodeLoadError> {
    let metadata = fs::symlink_metadata(path).map_err(EpisodeLoadError::Io)?;
    if metadata.file_type().is_symlink() {
        return Err(EpisodeLoadError::UnsafePath);
    }
    Ok(())
}

/// Safe typed episode-loading failures that do not disclose private paths.
#[derive(Debug, Error)]
pub enum EpisodeLoadError {
    #[error("episode package I/O failed")]
    Io(#[source] std::io::Error),
    #[error("episode package index is invalid")]
    InvalidPackageIndex,
    #[error("public episode manifest is invalid")]
    InvalidPublicManifest,
    #[error("private ground truth is invalid")]
    InvalidGroundTruth,
    #[error("episode package uses an unsupported schema")]
    UnsupportedSchema,
    #[error("episode identifiers do not match")]
    EpisodeIdMismatch,
    #[error("episode package contains an unsafe path or symlink")]
    UnsafePath,
    #[error("private ground truth is inside the public root")]
    PrivateRootExposed,
    #[error("public episode artifact is invalid")]
    InvalidPublicArtifact,
}
