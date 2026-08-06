use serde::{Deserialize, Serialize};

use crate::ContractValidationError;

/// Whether a larger or smaller metric value represents improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
}

/// Stable reason describing whether a metric has a meaningful value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Applicability {
    Applicable,
    NotRequired,
    ZeroDenominator,
    RequiresRepeatedRuns,
    RequiresFaultPair,
    UnavailableResource,
}

/// Inclusive numeric bounds for a metric.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricRange {
    pub minimum: f64,
    pub maximum: f64,
}

/// Raw metric with direction, applicability, and denominator provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricValue {
    pub value: Option<f64>,
    pub applicability: Applicability,
    pub direction: MetricDirection,
    pub range: MetricRange,
    pub numerator: Option<u64>,
    pub denominator: Option<u64>,
}

impl MetricValue {
    /// Validates finite bounds, applicability, range, and ratio components.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if !self.range.minimum.is_finite()
            || !self.range.maximum.is_finite()
            || self.range.minimum > self.range.maximum
        {
            return Err(ContractValidationError::new(
                "metric.range",
                "metric range must be finite and ordered",
            ));
        }
        match (self.applicability, self.value) {
            (Applicability::Applicable, Some(value))
                if value.is_finite()
                    && (self.range.minimum..=self.range.maximum).contains(&value) => {}
            (Applicability::Applicable, _) => {
                return Err(ContractValidationError::new(
                    "metric.value",
                    "applicable metric requires an in-range finite value",
                ));
            }
            (_, None) => {}
            (_, Some(_)) => {
                return Err(ContractValidationError::new(
                    "metric.value",
                    "non-applicable metric must have a null value",
                ));
            }
        }
        if let (Some(numerator), Some(denominator)) = (self.numerator, self.denominator)
            && numerator > denominator
        {
            return Err(ContractValidationError::new(
                "metric.numerator",
                "ratio numerator cannot exceed denominator",
            ));
        }
        Ok(())
    }
}
