use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{ContractValidationError, SchemaVersion, Sha256Digest};

const VERSION: SchemaVersion = SchemaVersion::new(0, 9);
const MAX_VERSIONS: usize = 16;
const MAX_TOOLS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    ManagedTool,
    DeploymentAdapter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionCapability {
    PublicEpisodeRead,
    ManagedToolRequest,
    ProcessSpawn,
    LocalReadOnlyData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionNetworkPolicy {
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionLimits {
    pub wall_time_ms: u64,
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_processes: u32,
    pub max_concurrency: u32,
}

impl ExtensionLimits {
    fn validate(&self) -> Result<(), ContractValidationError> {
        if self.wall_time_ms == 0
            || self.max_input_bytes == 0
            || self.max_output_bytes == 0
            || self.max_processes == 0
            || self.max_concurrency == 0
        {
            return Err(ContractValidationError::new(
                "extension.limits",
                "every extension limit must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub kind: ExtensionKind,
    pub executable_sha256: Sha256Digest,
    pub supported_versions: BTreeSet<SchemaVersion>,
    pub requested_capabilities: BTreeSet<ExtensionCapability>,
    pub network: ExtensionNetworkPolicy,
    pub tools: BTreeSet<String>,
    pub limits: ExtensionLimits,
}

impl ExtensionManifest {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        bounded_id(&self.id, "extension.id")?;
        if self.supported_versions.is_empty() || self.supported_versions.len() > MAX_VERSIONS {
            return Err(ContractValidationError::new(
                "extension.supported_versions",
                "supported version count is outside the allowed range",
            ));
        }
        if self.tools.len() > MAX_TOOLS || self.tools.iter().any(|tool| !safe_id(tool)) {
            return Err(ContractValidationError::new(
                "extension.tools",
                "tool inventory is invalid or exceeds its bound",
            ));
        }
        if self.kind == ExtensionKind::ManagedTool && self.tools.is_empty() {
            return Err(ContractValidationError::new(
                "extension.tools",
                "a managed-tool adapter must declare at least one tool",
            ));
        }
        if self.kind == ExtensionKind::DeploymentAdapter && !self.tools.is_empty() {
            return Err(ContractValidationError::new(
                "extension.tools",
                "a deployment adapter cannot declare managed tools",
            ));
        }
        self.limits.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionCapabilityPolicy {
    pub schema_version: SchemaVersion,
    pub policy_sha256: Sha256Digest,
    pub allowed_capabilities: BTreeSet<ExtensionCapability>,
    pub network: ExtensionNetworkPolicy,
    pub maximum_limits: ExtensionLimits,
}

impl ExtensionCapabilityPolicy {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        self.maximum_limits.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionResolutionStatus {
    Eligible,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionConformanceStatus {
    Conformant,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionConformanceResult {
    pub schema_version: SchemaVersion,
    pub manifest_sha256: Sha256Digest,
    pub executable_sha256: Sha256Digest,
    pub policy_sha256: Sha256Digest,
    pub protocol_transcript_sha256: Option<Sha256Digest>,
    pub status: ExtensionConformanceStatus,
    pub checks: BTreeSet<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionResolution {
    pub schema_version: SchemaVersion,
    pub manifest_sha256: Sha256Digest,
    pub policy_sha256: Sha256Digest,
    pub granted_capabilities: BTreeSet<ExtensionCapability>,
    pub status: ExtensionResolutionStatus,
    pub reasons: Vec<String>,
}

impl ExtensionResolution {
    #[must_use]
    pub fn resolve(
        manifest_sha256: Sha256Digest,
        manifest: &ExtensionManifest,
        policy: &ExtensionCapabilityPolicy,
    ) -> Self {
        let mut reasons = Vec::new();
        if manifest.validate().is_err() || policy.validate().is_err() {
            reasons.push("invalid_contract".to_owned());
        }
        if !manifest
            .requested_capabilities
            .is_subset(&policy.allowed_capabilities)
        {
            reasons.push("capability_not_allowed".to_owned());
        }
        if exceeds(&manifest.limits, &policy.maximum_limits) {
            reasons.push("limit_exceeds_policy".to_owned());
        }
        let status = if reasons.is_empty() {
            ExtensionResolutionStatus::Eligible
        } else {
            ExtensionResolutionStatus::Rejected
        };
        let granted_capabilities = if status == ExtensionResolutionStatus::Eligible {
            manifest.requested_capabilities.clone()
        } else {
            BTreeSet::new()
        };
        Self {
            schema_version: VERSION,
            manifest_sha256,
            policy_sha256: policy.policy_sha256,
            granted_capabilities,
            status,
            reasons,
        }
    }
}

fn exceeds(requested: &ExtensionLimits, maximum: &ExtensionLimits) -> bool {
    requested.wall_time_ms > maximum.wall_time_ms
        || requested.max_input_bytes > maximum.max_input_bytes
        || requested.max_output_bytes > maximum.max_output_bytes
        || requested.max_processes > maximum.max_processes
        || requested.max_concurrency > maximum.max_concurrency
}

pub(crate) fn require_version(version: SchemaVersion) -> Result<(), ContractValidationError> {
    if version != VERSION {
        return Err(ContractValidationError::new(
            "schema_version",
            "unsupported extension schema version",
        ));
    }
    Ok(())
}

pub(crate) fn bounded_id(value: &str, field: &'static str) -> Result<(), ContractValidationError> {
    if !safe_id(value) {
        return Err(ContractValidationError::new(
            field,
            "identifier is empty, too long, or contains unsafe characters",
        ));
    }
    Ok(())
}

fn safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}
