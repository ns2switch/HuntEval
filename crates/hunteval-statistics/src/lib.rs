//! Deterministic paired summaries that retain sample counts and uncertainty.

mod calibration;
mod policy;
mod stability;

pub use calibration::{
    CalibrationObservation, CalibrationResult, CalibrationStatus, evaluate_calibration,
};
pub use policy::{
    CalibrationPolicy, ClaimStrength, ComparisonClass, EffectSizeMethod, IntervalMethod,
    MultiplicityMethod, MultiplicityPolicy, StatisticalPolicy, StatisticalPolicyError,
    claim_strength, enforce_multiplicity_guard, holm_bonferroni_thresholds,
};
pub use stability::{
    StabilityError, StabilityInput, StabilitySample, StabilitySummary, UnavailableRepetition,
    UnavailableRepetitionReason, evaluate_stability,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticalSummary {
    pub count: usize,
    pub mean: Option<f64>,
    pub interval: Option<ConfidenceInterval>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedDifference {
    pub count: usize,
    pub mean_difference: Option<f64>,
    pub interval: Option<ConfidenceInterval>,
    pub wins: usize,
    pub ties: usize,
    pub losses: usize,
    pub conclusive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedEffectSize {
    pub count: usize,
    pub mean_difference: Option<f64>,
    pub standardized_difference: Option<f64>,
}

/// Policy-bound normalized comparison; raw paired observations remain authoritative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyComparisonResult {
    pub policy_sha256: hunteval_domain::Sha256Digest,
    pub comparison_class: ComparisonClass,
    pub paired_difference: PairedDifference,
    pub effect_size: PairedEffectSize,
    pub claim_strength: ClaimStrength,
    pub multiplicity: MultiplicityPolicy,
}

pub fn compare_with_policy(
    policy: &StatisticalPolicy,
    policy_sha256: hunteval_domain::Sha256Digest,
    left: &[Option<f64>],
    right: &[Option<f64>],
    seed: u64,
) -> Result<PolicyComparisonResult, PolicyComparisonError> {
    policy.validate()?;
    if left.len() != right.len() {
        return Err(StatisticsError::UnpairedCells.into());
    }
    let differences: Vec<_> = left
        .iter()
        .zip(right)
        .filter_map(|(left, right)| Some(left.as_ref()? - right.as_ref()?))
        .collect();
    validate(&differences)?;
    let wins = differences.iter().filter(|value| **value > 0.0).count();
    let losses = differences.iter().filter(|value| **value < 0.0).count();
    let ties = differences.len() - wins - losses;
    let mean = (!differences.is_empty()).then(|| average(&differences));
    let interval = (differences.len() >= 2)
        .then(|| bootstrap_interval_at(&differences, seed, policy.confidence_level));
    let excludes_zero = interval.is_some_and(|value| value.lower > 0.0 || value.upper < 0.0);
    let strength = claim_strength(policy, differences.len(), excludes_zero)?;
    let paired_difference = PairedDifference {
        count: differences.len(),
        mean_difference: mean,
        interval,
        wins,
        ties,
        losses,
        conclusive: strength == ClaimStrength::Conclusive,
    };
    Ok(PolicyComparisonResult {
        policy_sha256,
        comparison_class: policy.comparison_class,
        paired_difference,
        effect_size: paired_effect_size(left, right)?,
        claim_strength: strength,
        multiplicity: policy.multiplicity.clone(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankingEntry {
    pub deployment: String,
    pub disqualifying_violations: usize,
    pub aggregate_score: Option<f64>,
    pub raw_metrics: std::collections::BTreeMap<String, Option<f64>>,
}

/// Orders constraints before scores while retaining raw metric vectors.
#[must_use]
pub fn rank(mut entries: Vec<RankingEntry>) -> Vec<RankingEntry> {
    entries.sort_by(|left, right| {
        left.disqualifying_violations
            .cmp(&right.disqualifying_violations)
            .then_with(|| {
                right
                    .aggregate_score
                    .unwrap_or(f64::NEG_INFINITY)
                    .total_cmp(&left.aggregate_score.unwrap_or(f64::NEG_INFINITY))
            })
            .then_with(|| left.deployment.cmp(&right.deployment))
    });
    entries
}

pub fn summarize(
    values: &[f64],
    bootstrap_seed: u64,
) -> Result<StatisticalSummary, StatisticsError> {
    validate(values)?;
    if values.is_empty() {
        return Ok(StatisticalSummary {
            count: 0,
            mean: None,
            interval: None,
        });
    }
    let mean = average(values);
    let interval = if values.len() < 2 {
        None
    } else {
        Some(bootstrap_interval(values, bootstrap_seed))
    };
    Ok(StatisticalSummary {
        count: values.len(),
        mean: Some(mean),
        interval,
    })
}

pub fn paired_difference(
    left: &[Option<f64>],
    right: &[Option<f64>],
    seed: u64,
) -> Result<PairedDifference, StatisticsError> {
    if left.len() != right.len() {
        return Err(StatisticsError::UnpairedCells);
    }
    let differences: Vec<_> = left
        .iter()
        .zip(right)
        .filter_map(|(left, right)| Some(left.as_ref()? - right.as_ref()?))
        .collect();
    validate(&differences)?;
    let wins = differences.iter().filter(|value| **value > 0.0).count();
    let losses = differences.iter().filter(|value| **value < 0.0).count();
    let ties = differences.len() - wins - losses;
    let summary = summarize(&differences, seed)?;
    let conclusive = summary
        .interval
        .is_some_and(|interval| interval.lower > 0.0 || interval.upper < 0.0);
    Ok(PairedDifference {
        count: differences.len(),
        mean_difference: summary.mean,
        interval: summary.interval,
        wins,
        ties,
        losses,
        conclusive,
    })
}

pub fn paired_effect_size(
    left: &[Option<f64>],
    right: &[Option<f64>],
) -> Result<PairedEffectSize, StatisticsError> {
    if left.len() != right.len() {
        return Err(StatisticsError::UnpairedCells);
    }
    let differences: Vec<_> = left
        .iter()
        .zip(right)
        .filter_map(|(left, right)| Some(left.as_ref()? - right.as_ref()?))
        .collect();
    validate(&differences)?;
    if differences.is_empty() {
        return Ok(PairedEffectSize {
            count: 0,
            mean_difference: None,
            standardized_difference: None,
        });
    }
    let mean = average(&differences);
    let standardized_difference = if differences.len() < 2 {
        None
    } else {
        let variance = differences
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (differences.len() - 1) as f64;
        let deviation = variance.sqrt();
        (deviation > f64::EPSILON * mean.abs().max(1.0)).then_some(mean / deviation)
    };
    Ok(PairedEffectSize {
        count: differences.len(),
        mean_difference: Some(mean),
        standardized_difference,
    })
}

fn bootstrap_interval(values: &[f64], seed: u64) -> ConfidenceInterval {
    bootstrap_interval_at(values, seed, 0.95)
}

fn bootstrap_interval_at(values: &[f64], seed: u64, confidence: f64) -> ConfidenceInterval {
    let mut state = seed.max(1);
    let mut means = Vec::with_capacity(1_000);
    for _ in 0..1_000 {
        let mut sum = 0.0;
        for _ in values {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            sum += values[((state >> 32) as usize) % values.len()];
        }
        means.push(sum / values.len() as f64);
    }
    means.sort_by(f64::total_cmp);
    let alpha_tail = (1.0 - confidence) / 2.0;
    let lower = (alpha_tail * means.len() as f64).floor() as usize;
    let upper = ((1.0 - alpha_tail) * means.len() as f64).ceil() as usize - 1;
    ConfidenceInterval {
        lower: means[lower.min(means.len() - 1)],
        upper: means[upper.min(means.len() - 1)],
        confidence,
    }
}

fn average(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn validate(values: &[f64]) -> Result<(), StatisticsError> {
    if values.iter().any(|value| !value.is_finite()) {
        Err(StatisticsError::NonFinite)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StatisticsError {
    #[error("statistical input contains a non-finite value")]
    NonFinite,
    #[error("paired inputs have different cell counts")]
    UnpairedCells,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PolicyComparisonError {
    #[error("statistical policy is invalid")]
    Policy(#[from] StatisticalPolicyError),
    #[error("statistical comparison input is invalid")]
    Statistics(#[from] StatisticsError),
}
