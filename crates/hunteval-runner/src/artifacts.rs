use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use hunteval_domain::{RunId, SchemaVersion, Sha256Digest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Reproducibility manifest written only by the trusted runner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunManifest {
    pub schema_version: SchemaVersion,
    pub run_id: RunId,
    pub hashes: BTreeMap<String, Sha256Digest>,
    pub partial: bool,
}

/// Append-only writer rooted in a per-run partial directory.
#[derive(Debug)]
pub struct ArtifactWriter {
    partial_root: PathBuf,
    final_root: PathBuf,
}

impl ArtifactWriter {
    pub fn create(parent: &Path, run_id: &RunId) -> Result<Self, ArtifactError> {
        fs::create_dir_all(parent).map_err(ArtifactError::Io)?;
        let final_root = parent.join(run_id.as_str());
        let partial_root = parent.join(format!("{}.partial", run_id.as_str()));
        if final_root.exists() || partial_root.exists() {
            return Err(ArtifactError::AlreadyExists);
        }
        fs::create_dir(&partial_root).map_err(ArtifactError::Io)?;
        Ok(Self {
            partial_root,
            final_root,
        })
    }

    pub fn append(&self, relative: &Path, bytes: &[u8]) -> Result<(), ArtifactError> {
        let path = self.resolve(relative)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ArtifactError::Io)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(ArtifactError::Io)?;
        file.write_all(bytes)
            .and_then(|_| file.sync_data())
            .map_err(ArtifactError::Io)
    }

    pub fn write_json<T: Serialize>(
        &self,
        relative: &Path,
        value: &T,
    ) -> Result<(), ArtifactError> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(ArtifactError::Serialize)?;
        bytes.push(b'\n');
        self.append(relative, &bytes)
    }

    pub fn finalize(self) -> Result<PathBuf, ArtifactError> {
        fs::rename(&self.partial_root, &self.final_root).map_err(ArtifactError::Io)?;
        Ok(self.final_root)
    }

    #[must_use]
    pub fn partial_root(&self) -> &Path {
        &self.partial_root
    }

    fn resolve(&self, relative: &Path) -> Result<PathBuf, ArtifactError> {
        if relative.as_os_str().is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(ArtifactError::InvalidPath);
        }
        Ok(self.partial_root.join(relative))
    }
}

#[derive(Debug, Error)]
pub enum ArtifactError {
    #[error("run artifact directory already exists")]
    AlreadyExists,
    #[error("artifact path must be relative and traversal-free")]
    InvalidPath,
    #[error("artifact I/O failed: {0}")]
    Io(io::Error),
    #[error("artifact serialization failed: {0}")]
    Serialize(serde_json::Error),
}
