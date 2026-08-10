use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    DiffOperationKind, ImprovementContractError, MutableSectionClass, require_v08, valid_id,
    valid_text,
};
use crate::{DiagnosticSourceReference, SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptWeaknessCode {
    RoleAmbiguity,
    MissingOutputContract,
    MissingEvidenceRequirements,
    MissingAcceptanceCriteria,
    MissingStoppingCondition,
    UnclearToolUsePolicy,
    InsufficientErrorHandling,
    InsufficientDelegationPolicy,
    DuplicatedResponsibility,
    MissingTaskOwnership,
    MissingConflictResolutionPolicy,
    ExcessiveCommunicationRequirements,
    InsufficientEvidenceSharingRules,
    OverlyBroadSpecialistInvocationCriteria,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservableSourceFamily {
    Agent,
    Task,
    Trajectory,
    Finding,
    Evidence,
    Metric,
    Action,
    Coordination,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptWeaknessDefinition {
    pub code: PromptWeaknessCode,
    pub description: String,
    pub target_section_classes: BTreeSet<MutableSectionClass>,
    pub required_diagnostic_codes: BTreeSet<String>,
    pub required_source_families: BTreeSet<ObservableSourceFamily>,
    pub allowed_operations: BTreeSet<DiffOperationKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptFailureTaxonomy {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub taxonomy_version: String,
    pub definitions: Vec<PromptWeaknessDefinition>,
}

impl PromptFailureTaxonomy {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        let mut codes = BTreeSet::new();
        if !valid_id(&self.id)
            || !valid_id(&self.taxonomy_version)
            || self.definitions.len() != 14
            || self.definitions.iter().any(|definition| {
                !codes.insert(definition.code)
                    || !valid_text(&definition.description)
                    || definition.target_section_classes.is_empty()
                    || definition.required_diagnostic_codes.is_empty()
                    || definition.required_source_families.is_empty()
                    || definition.allowed_operations.is_empty()
                    || definition
                        .required_diagnostic_codes
                        .iter()
                        .any(|code| !valid_id(code))
            })
        {
            return Err(ImprovementContractError::InvalidCollection);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationTargetKind {
    Agent,
    Deployment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendationTarget {
    pub kind: RecommendationTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuspectedWeakness {
    pub code: PromptWeaknessCode,
    pub evidence_sufficiency: EvidenceSufficiency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSufficiency {
    Direct,
    Corroborated,
    Controlled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReferenceV08 {
    pub id: String,
    pub sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestedChange {
    pub operation: DiffOperationKind,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendationValidation {
    pub required: bool,
    pub experiment_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposedStatus {
    Proposed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromptRecommendation {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub target: RecommendationTarget,
    pub issue_code: String,
    pub observed_evidence: Vec<DiagnosticSourceReference>,
    pub suspected_weakness: SuspectedWeakness,
    pub target_artifact: ArtifactReferenceV08,
    pub target_section: String,
    pub suggested_change: SuggestedChange,
    pub expected_effects: BTreeSet<String>,
    pub possible_trade_offs: BTreeSet<String>,
    pub validation: RecommendationValidation,
    pub status: ProposedStatus,
}

impl PromptRecommendation {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id)
            || !valid_id(&self.target.id)
            || !valid_id(&self.issue_code)
            || !valid_id(&self.target_artifact.id)
            || !valid_id(&self.target_section)
            || self.observed_evidence.is_empty()
            || self.observed_evidence.len() > 128
            || !self.validation.required
            || self.expected_effects.is_empty()
            || !valid_text(&self.suggested_change.rationale)
            || self
                .observed_evidence
                .iter()
                .any(|source| !source.has_safe_shape())
        {
            return Err(ImprovementContractError::InvalidValue);
        }
        Ok(())
    }
}
