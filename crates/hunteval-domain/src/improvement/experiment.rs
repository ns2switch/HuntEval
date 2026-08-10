use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    DiffOperationKind, ImmutableSectionClass, ImprovementContractError, MutableSectionClass,
    SafetyStatus, require_v08, valid_id,
};
use crate::{SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    MinimumMetric,
    MaximumRegression,
    MaximumVerifiedCostIncrease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredProvenance {
    None,
    Measured,
    VerifiedAdapter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementConstraint {
    pub kind: ConstraintKind,
    pub metric: String,
    pub threshold: f64,
    pub required_provenance: RequiredProvenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementPolicy {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub immutable_section_classes: BTreeSet<ImmutableSectionClass>,
    pub allowed_targets: BTreeSet<MutableSectionClass>,
    pub allowed_operations: BTreeSet<DiffOperationKind>,
    pub max_artifact_bytes: u64,
    pub max_growth_percent: u16,
    pub answer_leakage_check_required: bool,
    pub hidden_test_feedback_during_selection: bool,
    pub human_review_required: bool,
    pub autonomous_adoption: bool,
    pub constraints: Vec<ImprovementConstraint>,
}

impl ImprovementPolicy {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        let immutable = BTreeSet::from([
            ImmutableSectionClass::AuthorizationPolicy,
            ImmutableSectionClass::ToolAccessPolicy,
            ImmutableSectionClass::FilesystemPolicy,
            ImmutableSectionClass::NetworkPolicy,
            ImmutableSectionClass::DataHandlingPolicy,
            ImmutableSectionClass::GroundTruthIsolation,
            ImmutableSectionClass::BenchmarkConstraints,
            ImmutableSectionClass::OutputIntegrity,
            ImmutableSectionClass::SecurityControls,
        ]);
        let has_minimum = self
            .constraints
            .iter()
            .any(|item| item.kind == ConstraintKind::MinimumMetric);
        let has_regression = self
            .constraints
            .iter()
            .any(|item| item.kind == ConstraintKind::MaximumRegression);
        if !valid_id(&self.id)
            || self.immutable_section_classes != immutable
            || self.allowed_targets.is_empty()
            || self.allowed_operations.is_empty()
            || self.max_artifact_bytes == 0
            || self.max_artifact_bytes > 1024 * 1024
            || self.max_growth_percent > 100
            || !self.answer_leakage_check_required
            || self.hidden_test_feedback_during_selection
            || !self.human_review_required
            || self.autonomous_adoption
            || !has_minimum
            || !has_regression
            || self.constraints.iter().any(|item| {
                !item.threshold.is_finite() || item.threshold < 0.0 || !valid_id(&item.metric)
            })
        {
            return Err(ImprovementContractError::InvalidExperiment);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementControlHashes {
    pub episode_set: Sha256Digest,
    pub seed_set: Sha256Digest,
    pub budgets: Sha256Digest,
    pub models: Sha256Digest,
    pub topology: Sha256Digest,
    pub managed_tool_policy: Sha256Digest,
    pub execution_policy: Sha256Digest,
    pub schemas: Sha256Digest,
    pub runtime_binaries: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedCellReference {
    pub baseline_cell_id: String,
    pub candidate_cell_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementExperiment {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub lineage_id: String,
    pub baseline_artifact_sha256: Sha256Digest,
    pub candidate_artifact_sha256: Sha256Digest,
    pub artifact_diff_sha256: Sha256Digest,
    pub improvement_policy_sha256: Sha256Digest,
    pub partition_policy_sha256: Sha256Digest,
    pub scoring_profile_sha256: Sha256Digest,
    pub statistical_policy_sha256: Sha256Digest,
    pub changed_variable: String,
    pub control_hashes: ImprovementControlHashes,
    pub paired_cells: Vec<PairedCellReference>,
    pub candidate_frozen: bool,
}

impl ImprovementExperiment {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        let mut ids = BTreeSet::new();
        if !valid_id(&self.id)
            || !valid_id(&self.lineage_id)
            || !valid_id(&self.changed_variable)
            || self.baseline_artifact_sha256 == self.candidate_artifact_sha256
            || self.paired_cells.is_empty()
            || self.paired_cells.len() > 4096
            || !self.candidate_frozen
            || self.paired_cells.iter().any(|pair| {
                !valid_id(&pair.baseline_cell_id)
                    || !valid_id(&pair.candidate_cell_id)
                    || !ids.insert((
                        pair.baseline_cell_id.as_str(),
                        pair.candidate_cell_id.as_str(),
                    ))
            })
        {
            return Err(ImprovementContractError::InvalidExperiment);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementEquivalenceStatus {
    Eligible,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementEquivalenceResult {
    pub schema_version: SchemaVersion,
    pub experiment_id: String,
    pub experiment_sha256: Sha256Digest,
    pub artifact_diff_sha256: Sha256Digest,
    pub status: ImprovementEquivalenceStatus,
    pub declared_changed_variable: String,
    pub actual_changed_variables: BTreeSet<String>,
    pub controls_equal: bool,
    pub safety_status: SafetyStatus,
    pub leakage_status: SafetyStatus,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricDelta {
    pub metric: String,
    pub version: SchemaVersion,
    pub baseline: Option<f64>,
    pub candidate: Option<f64>,
    pub difference: Option<f64>,
    pub interval: Option<MetricInterval>,
    pub applicability: ImprovementMetricApplicability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementMetricApplicability {
    Available,
    Unavailable,
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricInterval {
    pub lower: f64,
    pub upper: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintStatus {
    Satisfied,
    Violated,
    Unverifiable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintResult {
    pub code: String,
    pub status: ConstraintStatus,
    pub source_metric: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledValidationDecision {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub experiment_id: String,
    pub experiment_sha256: Sha256Digest,
    pub equivalence_sha256: Sha256Digest,
    pub improvement_policy_sha256: Sha256Digest,
    pub status: ValidationStatus,
    pub paired_samples: u32,
    pub missing_pairs: Vec<String>,
    pub metric_deltas: Vec<MetricDelta>,
    pub constraints: Vec<ConstraintResult>,
    pub hidden_test_used_in_selection: bool,
    pub human_review_required: bool,
    pub limitations: BTreeSet<String>,
}
