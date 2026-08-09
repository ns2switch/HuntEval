use std::collections::BTreeMap;

use hunteval_domain::{
    EquivalenceStatus, Sha256Digest, TopologyAnalysis, TopologyEquivalenceResult,
    TopologyExperiment, TopologyMetricApplicability, TopologyMetricValue,
};
use hunteval_statistics::{
    ClaimStrength, PolicyComparisonError, PolicyComparisonResult, StatisticalPolicy,
    compare_with_policy, enforce_multiplicity_guard,
};
use serde::{Deserialize, Serialize};

use crate::topology_control::{TopologyControlError, build_controlled_topology_analysis};

/// Paired, normalized observations keyed by a stable metric name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyAblationObservations {
    pub baseline: BTreeMap<String, Vec<Option<f64>>>,
    pub candidate: BTreeMap<String, Vec<Option<f64>>>,
}

/// Auditable output of a controlled topology ablation reduction.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlledTopologyAblation {
    pub analysis: TopologyAnalysis,
    pub comparisons: BTreeMap<String, PolicyComparisonResult>,
}

/// Reduces already executed paired cells into controlled, policy-bound topology deltas.
pub fn execute_controlled_topology_ablation(
    experiment: &TopologyExperiment,
    equivalence: &TopologyEquivalenceResult,
    policy: &StatisticalPolicy,
    policy_sha256: Sha256Digest,
    observations: &TopologyAblationObservations,
    seed: u64,
) -> Result<ControlledTopologyAblation, TopologyControlError> {
    validate_observations(experiment, equivalence, observations)?;

    let mut comparisons = BTreeMap::new();
    let mut metrics = BTreeMap::new();
    for (name, baseline) in &observations.baseline {
        let candidate = observations
            .candidate
            .get(name)
            .ok_or(TopologyControlError::InvalidObservations)?;
        let comparison = compare_with_policy(policy, policy_sha256, candidate, baseline, seed)?;
        metrics.insert(
            format!("{name}_delta"),
            comparison.paired_difference.mean_difference.map_or_else(
                || unavailable_metric("insufficient_comparable_pairs"),
                applicable_metric,
            ),
        );
        comparisons.insert(name.clone(), comparison);
    }
    let multiplicity_guarded = enforce_multiplicity_guard(policy, &mut comparisons)
        .map_err(PolicyComparisonError::from)?;

    let role_contribution = comparisons.get("investigation_quality").map_or_else(
        || unavailable_metric("quality_metric_unavailable"),
        |comparison| {
            if comparison.claim_strength == ClaimStrength::Descriptive {
                unavailable_metric("insufficient_controlled_samples")
            } else {
                comparison.paired_difference.mean_difference.map_or_else(
                    || unavailable_metric("insufficient_comparable_pairs"),
                    applicable_metric,
                )
            }
        },
    );
    metrics.insert("role_contribution".to_owned(), role_contribution);

    let mut analysis = build_controlled_topology_analysis(experiment, equivalence, metrics)?;
    if multiplicity_guarded {
        analysis
            .limitations
            .insert("multiplicity_adjusted_inference_unavailable".to_owned());
    }
    Ok(ControlledTopologyAblation {
        analysis,
        comparisons,
    })
}

fn validate_observations(
    experiment: &TopologyExperiment,
    equivalence: &TopologyEquivalenceResult,
    observations: &TopologyAblationObservations,
) -> Result<(), TopologyControlError> {
    if equivalence.status != EquivalenceStatus::Eligible
        || observations.baseline.is_empty()
        || observations.baseline.len() > 128
        || observations
            .baseline
            .keys()
            .ne(observations.candidate.keys())
    {
        return Err(TopologyControlError::InvalidObservations);
    }
    let expected_pairs = experiment.paired_cell_ids.len() / 2;
    if expected_pairs == 0
        || observations.baseline.iter().any(|(name, values)| {
            !valid_metric_name(name)
                || values.len() != expected_pairs
                || observations
                    .candidate
                    .get(name)
                    .is_none_or(|candidate| candidate.len() != expected_pairs)
        })
    {
        return Err(TopologyControlError::InvalidObservations);
    }
    Ok(())
}

fn valid_metric_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn applicable_metric(value: f64) -> TopologyMetricValue {
    TopologyMetricValue {
        applicability: TopologyMetricApplicability::Applicable,
        value: Some(value.clamp(-1.0, 1.0)),
        reason_code: None,
    }
}

fn unavailable_metric(reason: &str) -> TopologyMetricValue {
    TopologyMetricValue {
        applicability: TopologyMetricApplicability::Unavailable,
        value: None,
        reason_code: Some(reason.to_owned()),
    }
}
