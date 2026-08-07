use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{Applicability, MetricDirection, MetricRange, MetricValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const RANGE: MetricRange = MetricRange {
    minimum: 0.0,
    maximum: 1.0,
};
const MAX_STABILITY_SAMPLES: usize = 10_000;
const MAX_COMPARISON_WORK: usize = 50_000_000;

/// Canonical structured observations from one completed benchmark repetition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilitySample {
    pub seed: u64,
    pub submission_claims: BTreeSet<String>,
    pub metrics: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableRepetitionReason {
    Missing,
    Failed,
    InvalidArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnavailableRepetition {
    pub seed: u64,
    pub reason: UnavailableRepetitionReason,
}

/// Complete declared repetition set and the observations available for it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilityInput {
    pub required_seeds: Vec<u64>,
    pub samples: Vec<StabilitySample>,
    pub unavailable: Vec<UnavailableRepetition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilitySummary {
    pub submission_stability: MetricValue,
    pub metric_stability: MetricValue,
    pub required_samples: usize,
    pub completed_samples: usize,
    pub comparable_pairs: usize,
    pub metric_comparable_pairs: usize,
    pub unavailable: Vec<UnavailableRepetition>,
}

pub fn evaluate_stability(mut input: StabilityInput) -> Result<StabilitySummary, StabilityError> {
    validate(&input)?;
    input.samples.sort_by_key(|sample| sample.seed);
    input.unavailable.sort_by_key(|item| item.seed);
    let required_samples = input.required_seeds.len();
    let completed_samples = input.samples.len();
    let expected_pairs = required_samples.saturating_mul(required_samples.saturating_sub(1)) / 2;
    let comparable_pairs =
        completed_samples.saturating_mul(completed_samples.saturating_sub(1)) / 2;
    let applicability = if required_samples < 2 {
        Some(Applicability::RequiresRepeatedRuns)
    } else if !input.unavailable.is_empty() {
        Some(Applicability::RequiresComparableCells)
    } else {
        None
    };
    let (submission_stability, metric_stability, metric_comparable_pairs) = match applicability {
        Some(reason) => (unavailable_metric(reason), unavailable_metric(reason), 0),
        None => pair_metrics(&input.samples, expected_pairs),
    };
    Ok(StabilitySummary {
        submission_stability,
        metric_stability,
        required_samples,
        completed_samples,
        comparable_pairs,
        metric_comparable_pairs,
        unavailable: input.unavailable,
    })
}

fn pair_metrics(
    samples: &[StabilitySample],
    expected_pairs: usize,
) -> (MetricValue, MetricValue, usize) {
    let mut submission_total = 0.0;
    let mut metric_total = 0.0;
    let mut metric_comparable_pairs = 0;
    for left in 0..samples.len() {
        for right in (left + 1)..samples.len() {
            submission_total += jaccard(
                &samples[left].submission_claims,
                &samples[right].submission_claims,
            );
            if let Some(similarity) =
                metric_similarity(&samples[left].metrics, &samples[right].metrics)
            {
                metric_total += similarity;
                metric_comparable_pairs += 1;
            }
        }
    }
    let denominator = expected_pairs as f64;
    let metric_stability = if metric_comparable_pairs == expected_pairs {
        applicable_metric(metric_total / denominator)
    } else {
        unavailable_metric(Applicability::RequiresComparableCells)
    };
    (
        applicable_metric(submission_total / denominator),
        metric_stability,
        metric_comparable_pairs,
    )
}

fn metric_similarity(left: &BTreeMap<String, f64>, right: &BTreeMap<String, f64>) -> Option<f64> {
    if left.keys().ne(right.keys()) || left.is_empty() {
        return None;
    }
    let distance = left
        .iter()
        .zip(right)
        .map(|((_, left), (_, right))| (left - right).abs())
        .sum::<f64>()
        / left.len() as f64;
    Some(1.0 - distance)
}

fn jaccard(left: &BTreeSet<String>, right: &BTreeSet<String>) -> f64 {
    let union = left.union(right).count();
    if union == 0 {
        1.0
    } else {
        left.intersection(right).count() as f64 / union as f64
    }
}

fn validate(input: &StabilityInput) -> Result<(), StabilityError> {
    if input.required_seeds.len() > MAX_STABILITY_SAMPLES {
        return Err(StabilityError::ComparisonTooLarge);
    }
    let required = input
        .required_seeds
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if required.len() != input.required_seeds.len() || required.is_empty() {
        return Err(StabilityError::InvalidSeedSet);
    }
    let observed = input
        .samples
        .iter()
        .map(|sample| sample.seed)
        .chain(input.unavailable.iter().map(|item| item.seed))
        .collect::<Vec<_>>();
    let unique = observed.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != observed.len() || unique != required {
        return Err(StabilityError::InvalidSeedSet);
    }
    if input
        .samples
        .iter()
        .flat_map(|sample| sample.metrics.values())
        .any(|value| !value.is_finite() || !(RANGE.minimum..=RANGE.maximum).contains(value))
    {
        return Err(StabilityError::InvalidMetricValue);
    }
    let pair_count = input
        .samples
        .len()
        .saturating_mul(input.samples.len().saturating_sub(1))
        / 2;
    let compared_items = input.samples.iter().fold(0usize, |total, sample| {
        total.saturating_add(
            sample
                .submission_claims
                .len()
                .saturating_add(sample.metrics.len()),
        )
    });
    let work = pair_count.saturating_add(
        input
            .samples
            .len()
            .saturating_sub(1)
            .saturating_mul(compared_items),
    );
    if work > MAX_COMPARISON_WORK {
        return Err(StabilityError::ComparisonTooLarge);
    }
    Ok(())
}

const fn applicable_metric(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        applicability: Applicability::Applicable,
        direction: MetricDirection::HigherIsBetter,
        range: RANGE,
        numerator: None,
        denominator: None,
    }
}

const fn unavailable_metric(applicability: Applicability) -> MetricValue {
    MetricValue {
        value: None,
        applicability,
        direction: MetricDirection::HigherIsBetter,
        range: RANGE,
        numerator: None,
        denominator: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StabilityError {
    #[error("stability seeds must form an exact non-empty partition")]
    InvalidSeedSet,
    #[error("stability metric values must be finite and within zero and one")]
    InvalidMetricValue,
    #[error("stability comparison exceeds deterministic work limits")]
    ComparisonTooLarge,
}
