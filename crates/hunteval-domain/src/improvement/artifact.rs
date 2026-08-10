use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ImprovementContractError, MAX_ITEMS, require_v08, valid_id, valid_text};
use crate::{SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    DeploymentConfiguration,
    Instruction,
    OutputContract,
    ToolDescription,
    CoordinationPolicy,
    OtherConfiguration,
}

impl ArtifactKind {
    #[must_use]
    pub const fn is_structurally_eligible(self) -> bool {
        !matches!(self, Self::OtherConfiguration)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArtifactMediaType {
    #[serde(rename = "application/json")]
    Json,
    #[serde(rename = "application/yaml")]
    Yaml,
    #[serde(rename = "text/markdown")]
    Markdown,
    #[serde(rename = "text/plain")]
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactProvenance {
    Repository,
    Generated,
    OperatorProvided,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredArtifact {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub kind: ArtifactKind,
    pub media_type: ArtifactMediaType,
    pub sha256: Sha256Digest,
    pub size_bytes: u64,
    pub label: String,
    pub provenance: ArtifactProvenance,
    pub structured_artifact_sha256: Option<Sha256Digest>,
}

impl RegisteredArtifact {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id)
            || self.label.is_empty()
            || self.label.len() > 256
            || self.label.contains(['/', '\\', '\0'])
            || self.size_bytes == 0
            || self.size_bytes > 1024 * 1024
            || (self.structured_artifact_sha256.is_some() && !self.kind.is_structurally_eligible())
        {
            return Err(ImprovementContractError::InvalidValue);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRegistry {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub artifacts: Vec<RegisteredArtifact>,
}

impl ArtifactRegistry {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id) || self.artifacts.is_empty() || self.artifacts.len() > MAX_ITEMS {
            return Err(ImprovementContractError::InvalidCollection);
        }
        let mut ids = BTreeSet::new();
        let mut digests = BTreeSet::new();
        for artifact in &self.artifacts {
            artifact.validate()?;
            if !ids.insert(&artifact.id) || !digests.insert(artifact.sha256) {
                return Err(ImprovementContractError::InvalidCollection);
            }
        }
        if !self
            .artifacts
            .windows(2)
            .all(|pair| pair[0].id < pair[1].id)
        {
            return Err(ImprovementContractError::InvalidCollection);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImmutableSectionClass {
    AuthorizationPolicy,
    ToolAccessPolicy,
    FilesystemPolicy,
    NetworkPolicy,
    DataHandlingPolicy,
    GroundTruthIsolation,
    BenchmarkConstraints,
    OutputIntegrity,
    SecurityControls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutableSectionClass {
    TaskPlanning,
    EvidenceRequirements,
    DelegationStrategy,
    StoppingConditions,
    CommunicationFormat,
    ErrorRecovery,
    OutputContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mutability", content = "class", rename_all = "snake_case")]
pub enum SectionPolicy {
    Immutable(ImmutableSectionClass),
    Mutable(MutableSectionClass),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSection {
    pub id: String,
    #[serde(flatten)]
    pub policy: SectionPolicy,
    pub content: String,
    pub sha256: Sha256Digest,
}

impl ArtifactSection {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        if !valid_id(&self.id) || !valid_text(&self.content) || self.content.len() > 65_536 {
            return Err(ImprovementContractError::InvalidStructure);
        }
        if Sha256Digest::from_bytes(self.content.as_bytes()) != self.sha256 {
            return Err(ImprovementContractError::DigestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredArtifact {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub registered_artifact_sha256: Sha256Digest,
    pub sections: Vec<ArtifactSection>,
}

impl StructuredArtifact {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id) || self.sections.is_empty() || self.sections.len() > MAX_ITEMS {
            return Err(ImprovementContractError::InvalidStructure);
        }
        let mut ids = BTreeSet::new();
        for section in &self.sections {
            section.validate()?;
            if !ids.insert(&section.id) {
                return Err(ImprovementContractError::InvalidStructure);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffOperationKind {
    AddSection,
    ReplaceSection,
    RemoveSection,
    AddConstraint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffOperation {
    pub operation: DiffOperationKind,
    pub section_id: String,
    pub section_class: MutableSectionClass,
    pub baseline_sha256: Option<Sha256Digest>,
    pub candidate_sha256: Option<Sha256Digest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyStatus {
    Passed,
    Rejected,
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDiff {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub baseline_artifact_sha256: Sha256Digest,
    pub candidate_artifact_sha256: Sha256Digest,
    pub changed_variable: String,
    pub operations: Vec<DiffOperation>,
    pub immutable_policy_status: SafetyStatus,
    pub reason_codes: Vec<String>,
}
