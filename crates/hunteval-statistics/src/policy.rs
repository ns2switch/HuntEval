use serde::{Deserialize, Serialize};
use thiserror::Error;

use hunteval_domain::{SchemaVersion, StatisticalPolicyId};

use super::PolicyComparisonResult;

const POLICY_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 6);
const MAX_PAIRED_SAMPLES: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonClass {
    Exploratory,
    Validation,
    HiddenTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntervalMethod {
    DeterministicPairedBootstrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectSizeMethod {
    PairedMeanDifference,
    PairedStandardizedDifference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiplicityMethod {
    ExploratoryUnadjusted,
    HolmBonferroni,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiplicityPolicy {
    pub method: MultiplicityMethod,
    pub family: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationPolicy {
    NotRequired,
    ConfidenceBrierAndSeverityConfusion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatisticalPolicy {
    pub schema_version: SchemaVersion,
    pub id: StatisticalPolicyId,
    pub comparison_class: ComparisonClass,
    pub minimum_paired_samples: usize,
    pub confidence_level: f64,
    pub interval_method: IntervalMethod,
    pub effect_size_method: EffectSizeMethod,
    pub multiplicity: MultiplicityPolicy,
    pub calibration: CalibrationPolicy,
}

impl StatisticalPolicy {
    pub fn validate(&self) -> Result<(), StatisticalPolicyError> {
        if self.schema_version != POLICY_SCHEMA_VERSION {
            return Err(StatisticalPolicyError::UnsupportedSchema);
        }
        if !(2..=MAX_PAIRED_SAMPLES).contains(&self.minimum_paired_samples) {
            return Err(StatisticalPolicyError::InvalidMinimumSamples);
        }
        if !self.confidence_level.is_finite()
            || self.confidence_level <= 0.5
            || self.confidence_level >= 1.0
        {
            return Err(StatisticalPolicyError::InvalidConfidence);
        }
        if !valid_family(&self.multiplicity.family) {
            return Err(StatisticalPolicyError::InvalidFamily);
        }
        if self.comparison_class != ComparisonClass::Exploratory
            && self.multiplicity.method == MultiplicityMethod::ExploratoryUnadjusted
        {
            return Err(StatisticalPolicyError::UnadjustedValidation);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStrength {
    Descriptive,
    Exploratory,
    Conclusive,
}

pub fn claim_strength(
    policy: &StatisticalPolicy,
    paired_samples: usize,
    interval_excludes_zero: bool,
) -> Result<ClaimStrength, StatisticalPolicyError> {
    policy.validate()?;
    if paired_samples < policy.minimum_paired_samples || !interval_excludes_zero {
        return Ok(ClaimStrength::Descriptive);
    }
    match policy.comparison_class {
        ComparisonClass::Exploratory => Ok(ClaimStrength::Exploratory),
        ComparisonClass::Validation | ComparisonClass::HiddenTest => Ok(ClaimStrength::Conclusive),
    }
}

pub fn holm_bonferroni_thresholds(
    comparisons: usize,
    alpha: f64,
) -> Result<Vec<f64>, StatisticalPolicyError> {
    if comparisons == 0
        || comparisons > 10_000
        || !alpha.is_finite()
        || !(0.0..1.0).contains(&alpha)
    {
        return Err(StatisticalPolicyError::InvalidMultiplicityInput);
    }
    Ok((0..comparisons)
        .map(|rank| alpha / (comparisons - rank) as f64)
        .collect())
}

/// Applies a fail-closed family guard when adjusted per-comparison evidence is unavailable.
///
/// A declared Holm-Bonferroni family cannot be treated as conclusive merely because each
/// unadjusted interval excludes zero. Until adjusted evidence is represented explicitly,
/// multi-comparison families remain descriptive.
pub fn enforce_multiplicity_guard(
    policy: &StatisticalPolicy,
    comparisons: &mut std::collections::BTreeMap<String, PolicyComparisonResult>,
) -> Result<bool, StatisticalPolicyError> {
    policy.validate()?;
    let guarded =
        policy.multiplicity.method == MultiplicityMethod::HolmBonferroni && comparisons.len() > 1;
    if guarded {
        for result in comparisons.values_mut() {
            result.claim_strength = ClaimStrength::Descriptive;
            result.paired_difference.conclusive = false;
        }
    }
    Ok(guarded)
}

fn valid_family(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StatisticalPolicyError {
    #[error("statistical policy schema is unsupported")]
    UnsupportedSchema,
    #[error("minimum paired sample count is invalid")]
    InvalidMinimumSamples,
    #[error("confidence level is invalid")]
    InvalidConfidence,
    #[error("multiplicity family is invalid")]
    InvalidFamily,
    #[error("validation comparisons require multiplicity adjustment")]
    UnadjustedValidation,
    #[error("multiplicity input is invalid")]
    InvalidMultiplicityInput,
}
