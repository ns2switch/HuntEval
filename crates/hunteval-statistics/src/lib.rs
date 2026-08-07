//! Deterministic paired summaries that retain sample counts and uncertainty.

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

fn bootstrap_interval(values: &[f64], seed: u64) -> ConfidenceInterval {
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
    ConfidenceInterval {
        lower: means[24],
        upper: means[974],
        confidence: 0.95,
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
