use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ContractValidationError, DeploymentId, EpisodeId, MetricValue, RunId, SchemaVersion};

/// Terminal status of an attempted run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Completed,
    Incomplete,
    Failed,
    BudgetExceeded,
    PolicyViolation,
}

/// Provenance category for resource measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceProvenance {
    Measured,
    VerifiedAdapter,
    SelfReported,
    Unavailable,
}

/// Monetary cost paired with verification provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcedCost {
    pub value: Option<f64>,
    pub provenance: ResourceProvenance,
    pub currency: Option<String>,
}

/// Runner-observed and provider-dependent resource usage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceUsage {
    pub duration_ms: u64,
    pub tool_calls: u32,
    pub sql_queries: u32,
    pub messages: u32,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub token_provenance: ResourceProvenance,
    pub estimated_cost: SourcedCost,
}

/// Six independent evaluation dimensions. Unsupported dimensions remain `null`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricVector {
    pub investigation_quality: Option<f64>,
    pub evidence_quality: Option<f64>,
    pub coordination_quality: Option<f64>,
    pub resilience: Option<f64>,
    pub efficiency: Option<f64>,
    pub reproducibility: Option<f64>,
}

/// Stable policy or benchmark constraint violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintViolation {
    pub code: String,
    pub disqualifying: bool,
}

/// Relative paths to normalized run artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReferences {
    pub trajectory: String,
    pub submission: String,
    pub metrics: String,
}

/// Normalized result persisted by the trusted runner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResult {
    pub schema_version: SchemaVersion,
    pub run_id: RunId,
    pub episode_id: EpisodeId,
    pub deployment_id: DeploymentId,
    pub status: RunStatus,
    pub raw_metrics: BTreeMap<String, MetricValue>,
    pub metric_vector: MetricVector,
    pub aggregate_scores: BTreeMap<String, f64>,
    pub aggregate_score_omissions: BTreeMap<String, String>,
    pub constraint_violations: Vec<ConstraintViolation>,
    pub resource_usage: ResourceUsage,
    pub artifacts: ArtifactReferences,
}

impl RunResult {
    /// Validates metric ranges, sourced resources, and relative artifact paths.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        for metric in self.raw_metrics.values() {
            metric.validate()?;
        }
        for value in self.aggregate_scores.values() {
            if !value.is_finite() || !(0.0..=1.0).contains(value) {
                return Err(ContractValidationError::new(
                    "aggregate_scores",
                    "aggregate scores must be finite and within zero and one",
                ));
            }
        }
        validate_cost(&self.resource_usage.estimated_cost)?;
        for path in [
            &self.artifacts.trajectory,
            &self.artifacts.submission,
            &self.artifacts.metrics,
        ] {
            if path.is_empty() || path.starts_with('/') || path.split('/').any(|part| part == "..")
            {
                return Err(ContractValidationError::new(
                    "artifacts",
                    "artifact paths must be relative and traversal-free",
                ));
            }
        }
        Ok(())
    }
}

fn validate_cost(cost: &SourcedCost) -> Result<(), ContractValidationError> {
    if cost
        .value
        .is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(ContractValidationError::new(
            "resource_usage.estimated_cost",
            "cost must be finite and nonnegative",
        ));
    }
    match (cost.provenance, cost.value) {
        (ResourceProvenance::Unavailable, None) => Ok(()),
        (ResourceProvenance::Unavailable, Some(_)) | (_, None) => {
            Err(ContractValidationError::new(
                "resource_usage.estimated_cost",
                "cost value and provenance are inconsistent",
            ))
        }
        (_, Some(_)) => Ok(()),
    }
}
