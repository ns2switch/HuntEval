use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{DiagnosticApplicability, DiagnosticSourceReference};
use crate::{BenchmarkCellId, ContractValidationError};
use crate::{SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionTargetKind {
    Agent,
    Role,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributionTarget {
    pub kind: ContributionTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributionInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributionMetricEffect {
    pub metric_name: String,
    pub metric_version: SchemaVersion,
    pub baseline_value: f64,
    pub candidate_value: f64,
    pub difference: f64,
    pub interval: Option<ContributionInterval>,
    pub claim_strength: ContributionClaimStrength,
    pub sources: BTreeSet<DiagnosticSourceReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionClaimStrength {
    Descriptive,
    Exploratory,
    Conclusive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledContributionAnalysis {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub experiment_id: String,
    pub experiment_sha256: Sha256Digest,
    pub equivalence_sha256: Sha256Digest,
    pub baseline_topology_sha256: Sha256Digest,
    pub candidate_topology_sha256: Sha256Digest,
    pub target: ContributionTarget,
    pub changed_variables: BTreeSet<String>,
    pub paired_cell_ids: BTreeSet<String>,
    pub metric_effects: Vec<ContributionMetricEffect>,
    pub applicability: DiagnosticApplicability,
    pub reason_code: Option<String>,
    pub experimental: bool,
    pub topology_dependent: bool,
    pub limitations: BTreeSet<String>,
}

impl ControlledContributionAnalysis {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        let available = self.applicability == DiagnosticApplicability::Available;
        if self.schema_version != SchemaVersion::new(0, 7)
            || !self.experimental
            || !self.topology_dependent
            || self.changed_variables.is_empty()
            || self.changed_variables.len() > 32
            || self.paired_cell_ids.len() > 1_000_000
            || self
                .paired_cell_ids
                .iter()
                .any(|id| id.parse::<BenchmarkCellId>().is_err())
            || self.metric_effects.len() > 128
            || available != (!self.metric_effects.is_empty() && self.reason_code.is_none())
            || (!available && self.reason_code.is_none())
            || !(self.limitations.contains("experimental_topology_dependent")
                || (self.limitations.contains("experimental")
                    && self.limitations.contains("topology_dependent")))
            || self.metric_effects.iter().any(|effect| {
                !effect.baseline_value.is_finite()
                    || !effect.candidate_value.is_finite()
                    || !effect.difference.is_finite()
                    || (effect.difference - (effect.candidate_value - effect.baseline_value)).abs()
                        > f64::EPSILON * 16.0
                    || effect.sources.is_empty()
                    || effect.sources.iter().any(|source| !source.has_safe_shape())
                    || !effect.sources.iter().any(|source| {
                        matches!(
                            source,
                            DiagnosticSourceReference::TopologyExperiment { .. }
                                | DiagnosticSourceReference::TopologyEquivalence { .. }
                                | DiagnosticSourceReference::TopologyAnalysis { .. }
                        )
                    })
                    || effect.interval.as_ref().is_some_and(|interval| {
                        !interval.lower.is_finite()
                            || !interval.upper.is_finite()
                            || !interval.confidence.is_finite()
                            || interval.lower > interval.upper
                            || !(0.0..=1.0).contains(&interval.confidence)
                    })
            })
        {
            return Err(ContractValidationError::new(
                "controlled_contribution_analysis",
                "controlled contribution is malformed or unsupported",
            ));
        }
        Ok(())
    }
}
