use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

/// Filesystem and environment policy for a deployment process.
#[derive(Debug, Clone)]
pub struct IsolationPolicy {
    public_root: PathBuf,
    environment: BTreeMap<String, String>,
}

impl IsolationPolicy {
    pub fn new(
        public_root: PathBuf,
        environment: BTreeMap<String, String>,
    ) -> Result<Self, PolicyError> {
        let root = public_root
            .canonicalize()
            .map_err(PolicyError::PublicRoot)?;
        for key in environment.keys() {
            if !key
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_' || byte.is_ascii_digit())
            {
                return Err(PolicyError::EnvironmentKey(key.clone()));
            }
        }
        Ok(Self {
            public_root: root,
            environment,
        })
    }

    #[must_use]
    pub fn public_root(&self) -> &Path {
        &self.public_root
    }

    #[must_use]
    pub const fn environment(&self) -> &BTreeMap<String, String> {
        &self.environment
    }

    pub fn resolve_public(&self, relative: &Path) -> Result<PathBuf, PolicyError> {
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| matches!(part, Component::ParentDir))
        {
            return Err(PolicyError::PathTraversal);
        }
        let path = self
            .public_root
            .join(relative)
            .canonicalize()
            .map_err(PolicyError::PublicPath)?;
        if !path.starts_with(&self.public_root) {
            return Err(PolicyError::PathTraversal);
        }
        Ok(path)
    }
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("public root is unavailable: {0}")]
    PublicRoot(std::io::Error),
    #[error("public path is unavailable: {0}")]
    PublicPath(std::io::Error),
    #[error("path escapes the public root")]
    PathTraversal,
    #[error("environment key is not allowlist-safe: {0}")]
    EnvironmentKey(String),
}
