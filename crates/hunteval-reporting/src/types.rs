use std::{collections::BTreeMap, path::Path};

use hunteval_domain::{RunResult, RunStatus, SchemaVersion};
use hunteval_statistics::{PairedDifference, StatisticalSummary};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactLink {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportClaim {
    pub label: String,
    pub value: Option<f64>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    pub schema_version: SchemaVersion,
    pub status_label: String,
    pub result: RunResult,
    pub artifacts: Vec<ArtifactLink>,
    pub claims: Vec<ReportClaim>,
}

impl RunReport {
    pub fn from_result(result: RunResult) -> Result<Self, ReportError> {
        result.validate().map_err(|_| ReportError::InvalidResult)?;
        let artifacts = [
            ("Trajectory", result.artifacts.trajectory.as_str()),
            ("Submission", result.artifacts.submission.as_str()),
            ("Metrics", result.artifacts.metrics.as_str()),
        ]
        .into_iter()
        .map(|(label, path)| ArtifactLink {
            label: label.into(),
            path: path.into(),
        })
        .collect();
        let claims = result
            .raw_metrics
            .iter()
            .map(|(name, metric)| ReportClaim {
                label: name.clone(),
                value: metric.value,
                source: format!("result.json#/raw_metrics/{name}"),
            })
            .collect();
        Ok(Self {
            schema_version: SchemaVersion::new(0, 1),
            status_label: status_label(result.status).into(),
            result,
            artifacts,
            claims,
        })
    }

    pub fn validate(&self) -> Result<(), ReportError> {
        self.result
            .validate()
            .map_err(|_| ReportError::InvalidResult)?;
        if self.artifacts.iter().any(|link| !safe_relative(&link.path))
            || self.claims.iter().any(|claim| claim.source.is_empty())
        {
            return Err(ReportError::InvalidReference);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReport {
    pub schema_version: SchemaVersion,
    pub label: String,
    pub summaries: BTreeMap<String, StatisticalSummary>,
    pub comparisons: BTreeMap<String, PairedDifference>,
    pub incomplete_cells: usize,
    pub claims: Vec<ReportClaim>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyDiagnosticFinding {
    pub classification: String,
    pub affected_runs: Vec<String>,
    pub observable_sources: Vec<String>,
    pub recommendation: String,
    pub validation_status: LegacyDiagnosticValidationStatus,
    pub validation_source: Option<String>,
    pub human_review_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyDiagnosticValidationStatus {
    Unvalidated,
    Validated,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyDiagnosticReport {
    pub schema_version: SchemaVersion,
    pub findings: Vec<LegacyDiagnosticFinding>,
}

impl LegacyDiagnosticReport {
    pub fn validate(&self) -> Result<(), ReportError> {
        if self.findings.iter().any(|finding| {
            finding.classification.trim().is_empty()
                || finding.affected_runs.is_empty()
                || finding.observable_sources.is_empty()
                || finding
                    .observable_sources
                    .iter()
                    .any(|source| source.trim().is_empty())
                || finding.recommendation.trim().is_empty()
                || match finding.validation_status {
                    LegacyDiagnosticValidationStatus::Unvalidated => {
                        finding.validation_source.is_some()
                    }
                    LegacyDiagnosticValidationStatus::Validated
                    | LegacyDiagnosticValidationStatus::Rejected => finding
                        .validation_source
                        .as_ref()
                        .is_none_or(|source| source.trim().is_empty()),
                }
                || !finding.human_review_required
        }) {
            return Err(ReportError::InvalidDiagnostic);
        }
        Ok(())
    }
}

pub trait ReportRenderer {
    fn render_run(&self, report: &RunReport) -> Result<Vec<u8>, ReportError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct JsonRenderer;

impl ReportRenderer for JsonRenderer {
    fn render_run(&self, report: &RunReport) -> Result<Vec<u8>, ReportError> {
        report.validate()?;
        let mut bytes = serde_json::to_vec_pretty(report).map_err(ReportError::Serialize)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

impl JsonRenderer {
    pub fn render_legacy_diagnostic(
        &self,
        report: &LegacyDiagnosticReport,
    ) -> Result<Vec<u8>, ReportError> {
        report.validate()?;
        let mut bytes = serde_json::to_vec_pretty(report).map_err(ReportError::Serialize)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

fn status_label(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Completed => "complete",
        RunStatus::Incomplete => "incomplete",
        RunStatus::Failed => "failed",
        RunStatus::BudgetExceeded => "budget exceeded",
        RunStatus::PolicyViolation => "policy violation",
    }
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && !Path::new(value)
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("run result is invalid")]
    InvalidResult,
    #[error("report contains an invalid artifact or evidence reference")]
    InvalidReference,
    #[error("diagnostic conclusions require observable sources and a controlled review gate")]
    InvalidDiagnostic,
    #[error("report serialization failed: {0}")]
    Serialize(serde_json::Error),
}
