mod constraints;
mod validation;

use std::collections::BTreeMap;

use hunteval_domain::{Applicability, SchemaVersion};
use thiserror::Error;

use crate::{AggregateScore, MetricSelection, MetricVector, MissingMetricPolicy, ScoringProfile};
use validation::{validate_metric_value, validate_profile};

pub use constraints::{ConstraintInput, evaluate_constraints};
pub use validation::normalize_profile;

pub fn score_profile(
    metrics: &MetricVector,
    profile: &ScoringProfile,
) -> Result<AggregateScore, ProfileError> {
    validate_profile(profile)?;
    let mut weighted = 0.0;
    let mut applicable_weight = 0.0;
    let mut omitted = BTreeMap::new();
    let mut blocked = false;
    for (name, selection) in &profile.metrics {
        let contract = crate::metric_contract(name, selection.version)
            .ok_or_else(|| ProfileError::UnknownMetricVersion(name.clone(), selection.version))?;
        match metrics.0.get(name) {
            Some(metric) if metric.value.is_some() => {
                validate_metric_value(name, metric, contract.direction)?;
                let value = metric
                    .value
                    .ok_or_else(|| ProfileError::MetricContractMismatch(name.clone()))?;
                let normalized = match contract.direction {
                    hunteval_domain::MetricDirection::HigherIsBetter => value,
                    hunteval_domain::MetricDirection::LowerIsBetter => 1.0 - value,
                };
                weighted += normalized * selection.weight;
                applicable_weight += selection.weight;
            }
            Some(metric) => {
                validate_metric_value(name, metric, contract.direction)?;
                record_missing(
                    name,
                    selection,
                    metric.applicability,
                    profile,
                    &mut omitted,
                    &mut applicable_weight,
                    &mut blocked,
                );
            }
            None => record_missing(
                name,
                selection,
                Applicability::UnavailableResource,
                profile,
                &mut omitted,
                &mut applicable_weight,
                &mut blocked,
            ),
        }
    }
    Ok(AggregateScore {
        profile_id: profile.id.clone(),
        value: (!blocked && applicable_weight > 0.0).then_some(weighted / applicable_weight),
        omitted_metrics: omitted,
    })
}

fn record_missing(
    name: &str,
    selection: &MetricSelection,
    applicability: Applicability,
    profile: &ScoringProfile,
    omitted: &mut BTreeMap<String, String>,
    applicable_weight: &mut f64,
    blocked: &mut bool,
) {
    omitted.insert(
        name.to_owned(),
        applicability_name(applicability).to_owned(),
    );
    let protected = matches!(
        name,
        "resilience"
            | "graceful_degradation"
            | "reproducibility"
            | "submission_stability"
            | "metric_stability"
            | "verified_cost_utilization"
    );
    match profile.missing_metric_policy {
        MissingMetricPolicy::Reject => *blocked = true,
        MissingMetricPolicy::Renormalize if protected => *blocked = true,
        MissingMetricPolicy::Renormalize => {}
        MissingMetricPolicy::Zero => *applicable_weight += selection.weight,
    }
}

const fn applicability_name(value: Applicability) -> &'static str {
    match value {
        Applicability::Applicable => "applicable",
        Applicability::NotRequired => "not_required",
        Applicability::ZeroDenominator => "zero_denominator",
        Applicability::TimelineNotSubmitted => "timeline_not_submitted",
        Applicability::TimelineTruthUnavailable => "timeline_truth_unavailable",
        Applicability::AcceptableStatusesUnavailable => "acceptable_statuses_unavailable",
        Applicability::InsufficientEvidenceRequirements => "insufficient_evidence_requirements",
        Applicability::RequiresRepeatedRuns => "requires_repeated_runs",
        Applicability::RequiresComparableCells => "requires_comparable_cells",
        Applicability::RequiresVerifiedResourceUsage => "requires_verified_resource_usage",
        Applicability::RequiresFaultPair => "requires_fault_pair",
        Applicability::UnavailableResource => "unavailable_resource",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProfileError {
    #[error("scoring profile identifier and metric selections are required")]
    InvalidProfile,
    #[error("scoring profile contains an invalid weight or metric reference")]
    InvalidWeightsOrMetric,
    #[error("scoring profile version {0} is unsupported")]
    UnsupportedProfileVersion(SchemaVersion),
    #[error("scoring profile references unknown metric {0} version {1}")]
    UnknownMetricVersion(String, SchemaVersion),
    #[error("metric {0} does not match its registered contract")]
    MetricContractMismatch(String),
    #[error("scoring profile constraint is invalid")]
    InvalidConstraint,
}
