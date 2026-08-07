use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::SchemaVersion;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Partition {
    Training,
    Validation,
    HiddenTest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CandidateConstraint {
    MinimumMetric { metric: String, minimum: f64 },
    MaximumRegression { metric: String, maximum: f64 },
    MaximumVerifiedCostIncrease { maximum: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentManifest {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub baseline_id: String,
    pub candidate_id: String,
    pub changed_variables: BTreeSet<String>,
    pub baseline_immutable_hashes: BTreeMap<String, String>,
    pub candidate_immutable_hashes: BTreeMap<String, String>,
    pub selection_partitions: BTreeSet<Partition>,
    pub validation_partitions: BTreeSet<Partition>,
    pub constraints: Vec<CandidateConstraint>,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExperimentObservation {
    pub baseline_metrics: BTreeMap<String, f64>,
    pub candidate_metrics: BTreeMap<String, f64>,
    pub baseline_verified_cost: Option<f64>,
    pub candidate_verified_cost: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationDecision {
    pub controlled_validation_passed: bool,
    pub violations: Vec<String>,
    pub human_review_required: bool,
}

pub fn validate_experiment_manifest(manifest: &ExperimentManifest) -> Result<(), ExperimentError> {
    if manifest.id.trim().is_empty()
        || manifest.baseline_id == manifest.candidate_id
        || manifest.changed_variables.len() != 1
        || !manifest.human_review_required
    {
        return Err(ExperimentError::InvalidDesign);
    }
    if manifest.baseline_immutable_hashes != manifest.candidate_immutable_hashes {
        return Err(ExperimentError::ImmutablePolicyChanged);
    }
    for section in ["authorization", "data_handling", "tool_access"] {
        if manifest.changed_variables.contains(section)
            || manifest
                .baseline_immutable_hashes
                .get(section)
                .is_none_or(|hash| hash.trim().is_empty())
        {
            return Err(ExperimentError::ImmutablePolicyChanged);
        }
    }
    if manifest
        .selection_partitions
        .contains(&Partition::HiddenTest)
    {
        return Err(ExperimentError::HiddenTestDisclosure);
    }
    if !manifest.selection_partitions.contains(&Partition::Training)
        || !manifest
            .validation_partitions
            .contains(&Partition::Validation)
    {
        return Err(ExperimentError::InvalidDesign);
    }
    validate_constraints(&manifest.constraints)
}

pub fn validate_candidate(
    manifest: &ExperimentManifest,
    observation: &ExperimentObservation,
) -> Result<ValidationDecision, ExperimentError> {
    validate_experiment_manifest(manifest)?;
    let mut violations = Vec::new();
    for constraint in &manifest.constraints {
        match constraint {
            CandidateConstraint::MinimumMetric { metric, minimum } => {
                let value = metric_value(&observation.candidate_metrics, metric)?;
                if value < *minimum {
                    violations.push(format!("minimum_metric:{metric}"));
                }
            }
            CandidateConstraint::MaximumRegression { metric, maximum } => {
                let baseline = metric_value(&observation.baseline_metrics, metric)?;
                let candidate = metric_value(&observation.candidate_metrics, metric)?;
                if baseline - candidate > *maximum {
                    violations.push(format!("maximum_regression:{metric}"));
                }
            }
            CandidateConstraint::MaximumVerifiedCostIncrease { maximum } => {
                let baseline = observation
                    .baseline_verified_cost
                    .ok_or(ExperimentError::UnverifiedCost)?;
                let candidate = observation
                    .candidate_verified_cost
                    .ok_or(ExperimentError::UnverifiedCost)?;
                if candidate - baseline > *maximum {
                    violations.push("maximum_verified_cost_increase".into());
                }
            }
        }
    }
    Ok(ValidationDecision {
        controlled_validation_passed: violations.is_empty(),
        violations,
        human_review_required: true,
    })
}

fn validate_constraints(constraints: &[CandidateConstraint]) -> Result<(), ExperimentError> {
    for constraint in constraints {
        let valid = match constraint {
            CandidateConstraint::MinimumMetric { minimum, .. }
            | CandidateConstraint::MaximumRegression {
                maximum: minimum, ..
            } => minimum.is_finite() && (0.0..=1.0).contains(minimum),
            CandidateConstraint::MaximumVerifiedCostIncrease { maximum } => {
                maximum.is_finite() && *maximum >= 0.0
            }
        };
        if !valid {
            return Err(ExperimentError::InvalidConstraint);
        }
    }
    Ok(())
}

fn metric_value(values: &BTreeMap<String, f64>, metric: &str) -> Result<f64, ExperimentError> {
    values
        .get(metric)
        .copied()
        .filter(|value| value.is_finite())
        .ok_or(ExperimentError::MissingMetric)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExperimentError {
    #[error("experiment must change exactly one variable and require human review")]
    InvalidDesign,
    #[error("candidate changes an immutable safety or data-handling section")]
    ImmutablePolicyChanged,
    #[error("hidden-test feedback cannot be used during candidate selection")]
    HiddenTestDisclosure,
    #[error("candidate constraint is invalid")]
    InvalidConstraint,
    #[error("required validation metric is missing or non-finite")]
    MissingMetric,
    #[error("cost constraint requires verified baseline and candidate cost")]
    UnverifiedCost,
}
