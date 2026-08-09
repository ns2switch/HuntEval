use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_CALIBRATION_SAMPLES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationObservation {
    pub confidence: f64,
    pub outcome: bool,
    pub predicted_severity: Option<u8>,
    pub actual_severity: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationStatus {
    Applicable,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationResult {
    pub sample_count: usize,
    pub status: CalibrationStatus,
    pub brier_score: Option<f64>,
    pub severity_accuracy: Option<f64>,
    pub reason_code: Option<String>,
}

pub fn evaluate_calibration(
    observations: &[CalibrationObservation],
) -> Result<CalibrationResult, CalibrationError> {
    if observations.len() > MAX_CALIBRATION_SAMPLES {
        return Err(CalibrationError::TooManySamples);
    }
    if observations.is_empty() {
        return Ok(CalibrationResult {
            sample_count: 0,
            status: CalibrationStatus::Unavailable,
            brier_score: None,
            severity_accuracy: None,
            reason_code: Some("no_calibration_samples".to_owned()),
        });
    }
    if observations.iter().any(|item| {
        !item.confidence.is_finite()
            || !(0.0..=1.0).contains(&item.confidence)
            || item.predicted_severity.is_some_and(|value| value > 4)
            || item.actual_severity.is_some_and(|value| value > 4)
            || item.predicted_severity.is_some() != item.actual_severity.is_some()
    }) {
        return Err(CalibrationError::InvalidObservation);
    }

    let brier_score = observations
        .iter()
        .map(|item| {
            let outcome = f64::from(u8::from(item.outcome));
            (item.confidence - outcome).powi(2)
        })
        .sum::<f64>()
        / observations.len() as f64;
    let severity_pairs: Vec<_> = observations
        .iter()
        .filter_map(|item| Some((item.predicted_severity?, item.actual_severity?)))
        .collect();
    let severity_accuracy = if severity_pairs.is_empty() {
        None
    } else {
        let matches = severity_pairs
            .iter()
            .filter(|(predicted, actual)| predicted == actual)
            .count();
        Some(matches as f64 / severity_pairs.len() as f64)
    };
    Ok(CalibrationResult {
        sample_count: observations.len(),
        status: CalibrationStatus::Applicable,
        brier_score: Some(brier_score),
        severity_accuracy,
        reason_code: None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CalibrationError {
    #[error("calibration sample count exceeds the supported bound")]
    TooManySamples,
    #[error("calibration observation is malformed")]
    InvalidObservation,
}
