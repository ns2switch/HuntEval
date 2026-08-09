use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const MAX_WALL_TIME_MS: u64 = 86_400_000;
const MAX_MEMORY_BYTES: u64 = 1 << 40;
const MAX_FILE_BYTES: u64 = 1 << 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxBackendId {
    LinuxBubblewrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    pub wall_time_ms: u64,
    pub cpu_time_seconds: u64,
    pub address_space_bytes: u64,
    pub file_size_bytes: u64,
    pub open_files: u64,
    pub processes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

impl ResourceLimits {
    pub fn validate(self) -> Result<(), PolicyError> {
        if !(1..=MAX_WALL_TIME_MS).contains(&self.wall_time_ms) {
            return Err(PolicyError::InvalidLimit("wall_time_ms"));
        }
        if !(1..=86_400).contains(&self.cpu_time_seconds) {
            return Err(PolicyError::InvalidLimit("cpu_time_seconds"));
        }
        if !(16_777_216..=MAX_MEMORY_BYTES).contains(&self.address_space_bytes) {
            return Err(PolicyError::InvalidLimit("address_space_bytes"));
        }
        if !(1_024..=MAX_FILE_BYTES).contains(&self.file_size_bytes) {
            return Err(PolicyError::InvalidLimit("file_size_bytes"));
        }
        if !(16..=1_048_576).contains(&self.open_files) {
            return Err(PolicyError::InvalidLimit("open_files"));
        }
        if !(1..=65_536).contains(&self.processes) {
            return Err(PolicyError::InvalidLimit("processes"));
        }
        if !(256..=1_073_741_824).contains(&self.stdout_bytes) {
            return Err(PolicyError::InvalidLimit("stdout_bytes"));
        }
        if !(256..=1_073_741_824).contains(&self.stderr_bytes) {
            return Err(PolicyError::InvalidLimit("stderr_bytes"));
        }
        Ok(())
    }

    #[must_use]
    pub const fn wall_time(self) -> Duration {
        Duration::from_millis(self.wall_time_ms)
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            wall_time_ms: 60_000,
            cpu_time_seconds: 60,
            address_space_bytes: 2 * 1024 * 1024 * 1024,
            file_size_bytes: 16 * 1024 * 1024,
            open_files: 256,
            processes: 64,
            stdout_bytes: 16 * 1024 * 1024,
            stderr_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedExecutionPolicy {
    pub schema_version: String,
    pub backend: SandboxBackendId,
    pub network: NetworkPolicy,
    pub limits: ResourceLimits,
}

impl ResolvedExecutionPolicy {
    #[must_use]
    pub fn hardened_default() -> Self {
        Self {
            schema_version: "0.5".to_owned(),
            backend: SandboxBackendId::LinuxBubblewrap,
            network: NetworkPolicy::Denied,
            limits: ResourceLimits::default(),
        }
    }

    pub fn validate(&self) -> Result<(), PolicyError> {
        if self.schema_version != "0.5" {
            return Err(PolicyError::UnsupportedVersion);
        }
        self.limits.validate()
    }

    pub fn sha256(&self) -> Result<String, PolicyError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(PolicyError::Serialize)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("execution policy version is unsupported")]
    UnsupportedVersion,
    #[error("execution policy limit is invalid: {0}")]
    InvalidLimit(&'static str),
    #[error("execution policy could not be serialized")]
    Serialize(#[source] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_is_valid_and_content_addressed() {
        let policy = ResolvedExecutionPolicy::hardened_default();
        let first = policy.sha256();
        let second = policy.sha256();
        assert!(matches!((first, second), (Ok(a), Ok(b)) if a == b && a.len() == 64));
    }

    #[test]
    fn policy_rejects_zero_limits_and_unknown_versions() {
        let mut policy = ResolvedExecutionPolicy::hardened_default();
        policy.limits.processes = 0;
        assert!(matches!(
            policy.validate(),
            Err(PolicyError::InvalidLimit("processes"))
        ));
        policy.limits = ResourceLimits::default();
        policy.schema_version = "0.6".to_owned();
        assert!(matches!(
            policy.validate(),
            Err(PolicyError::UnsupportedVersion)
        ));
    }
}
