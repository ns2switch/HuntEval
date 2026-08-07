use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::{
    AggregateScore, ConstraintEvaluation, MetricVector, MissingMetricPolicy, ScoringProfile,
};

/// Labels observed policy violations using the selected profile.
#[must_use]
pub fn evaluate_constraints(
    observed: &BTreeSet<String>,
    profile: &ScoringProfile,
) -> Vec<ConstraintEvaluation> {
    observed
        .iter()
        .map(|code| ConstraintEvaluation {
            code: code.clone(),
            violated: true,
            disqualifying: profile.disqualifying_constraints.contains(code),
        })
        .collect()
}

pub fn score_profile(
    metrics: &MetricVector,
    profile: &ScoringProfile,
) -> Result<AggregateScore, ProfileError> {
    if profile.id.trim().is_empty() || profile.weights.is_empty() {
        return Err(ProfileError::InvalidProfile);
    }
    let total: f64 = profile.weights.values().sum();
    if profile
        .weights
        .values()
        .any(|weight| !weight.is_finite() || *weight < 0.0)
        || (total - 1.0).abs() > 1e-9
    {
        return Err(ProfileError::InvalidWeights);
    }
    let mut weighted = 0.0;
    let mut applicable_weight = 0.0;
    let mut omitted = BTreeMap::new();
    for (name, weight) in &profile.weights {
        let metric = metrics
            .0
            .get(name)
            .ok_or_else(|| ProfileError::UnknownMetric(name.clone()))?;
        match metric.value {
            Some(value) => {
                let normalized = match metric.direction {
                    hunteval_domain::MetricDirection::HigherIsBetter => value,
                    hunteval_domain::MetricDirection::LowerIsBetter => 1.0 - value,
                };
                weighted += normalized * weight;
                applicable_weight += weight;
            }
            None => {
                omitted.insert(
                    name.clone(),
                    format!("{:?}", metric.applicability).to_lowercase(),
                );
                if profile.missing_metric_policy == MissingMetricPolicy::Reject {
                    return Ok(AggregateScore {
                        profile_id: profile.id.clone(),
                        value: None,
                        omitted_metrics: omitted,
                    });
                }
                if profile.missing_metric_policy == MissingMetricPolicy::Zero {
                    applicable_weight += weight;
                }
            }
        }
    }
    let value = if applicable_weight == 0.0 {
        None
    } else {
        Some(weighted / applicable_weight)
    };
    Ok(AggregateScore {
        profile_id: profile.id.clone(),
        value,
        omitted_metrics: omitted,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProfileError {
    #[error("scoring profile identifier and weights are required")]
    InvalidProfile,
    #[error("scoring profile weights must be finite, nonnegative, and sum to one")]
    InvalidWeights,
    #[error("scoring profile references unknown metric {0}")]
    UnknownMetric(String),
}
