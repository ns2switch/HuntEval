use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{ContractValidationError, SchemaVersion, Sha256Digest};

const SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 6);
const MAX_METRICS: usize = 128;
const MAX_LIMITATIONS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyAnalysisKind {
    Observational,
    ControlledAblation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyMetricApplicability {
    Applicable,
    Unavailable,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyMetricValue {
    pub applicability: TopologyMetricApplicability,
    pub value: Option<f64>,
    pub reason_code: Option<String>,
}

impl TopologyMetricValue {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        let applicable = self.applicability == TopologyMetricApplicability::Applicable;
        if applicable
            != self
                .value
                .is_some_and(|value| value.is_finite() && (-1.0..=1.0).contains(&value))
        {
            return Err(invalid("metrics.value", "metric value is invalid"));
        }
        if applicable == self.reason_code.is_some()
            || self
                .reason_code
                .as_deref()
                .is_some_and(|reason| !valid_reason(reason))
        {
            return Err(invalid(
                "metrics.reason_code",
                "metric applicability reason is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyAnalysis {
    pub schema_version: SchemaVersion,
    pub baseline_topology_sha256: Sha256Digest,
    pub candidate_topology_sha256: Sha256Digest,
    pub experiment_sha256: Option<Sha256Digest>,
    pub analysis_kind: TopologyAnalysisKind,
    pub topology_dependent: bool,
    pub metrics: BTreeMap<String, TopologyMetricValue>,
    pub limitations: BTreeSet<String>,
}

impl TopologyAnalysis {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(invalid("schema_version", "schema version is unsupported"));
        }
        if self.metrics.is_empty() || self.metrics.len() > MAX_METRICS {
            return Err(invalid(
                "metrics",
                "metric count is outside the supported bound",
            ));
        }
        if self.limitations.len() > MAX_LIMITATIONS
            || self.limitations.iter().any(|value| !valid_reason(value))
            || self
                .metrics
                .iter()
                .any(|(name, value)| !valid_reason(name) || value.validate().is_err())
        {
            return Err(invalid("analysis", "analysis values are malformed"));
        }
        match self.analysis_kind {
            TopologyAnalysisKind::Observational if self.experiment_sha256.is_some() => {
                Err(invalid(
                    "experiment_sha256",
                    "observational analysis cannot cite a controlled experiment",
                ))
            }
            TopologyAnalysisKind::ControlledAblation
                if self.experiment_sha256.is_none()
                    || !self.topology_dependent
                    || !self.limitations.contains("experimental_topology_dependent") =>
            {
                Err(invalid(
                    "analysis_kind",
                    "controlled analysis requires experimental provenance",
                ))
            }
            _ => Ok(()),
        }
    }
}

fn valid_reason(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn invalid(field: &'static str, reason: &'static str) -> ContractValidationError {
    ContractValidationError::new(field, reason)
}
