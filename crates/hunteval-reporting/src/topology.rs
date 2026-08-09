use std::{collections::BTreeMap, fmt::Write};

use hunteval_domain::{SchemaVersion, Sha256Digest, TopologyAnalysis};
use hunteval_statistics::PolicyComparisonResult;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintFirstStatus {
    BaselinePreferred,
    CandidatePreferred,
    Tied,
    Incomparable,
}

/// Auditable controlled topology report with an authoritative raw metric vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyComparisonReport {
    pub schema_version: SchemaVersion,
    pub analysis: TopologyAnalysis,
    pub statistical_policy_sha256: Sha256Digest,
    pub scoring_profile_sha256: Sha256Digest,
    pub comparisons: BTreeMap<String, PolicyComparisonResult>,
    pub aggregate_score: Option<f64>,
    pub constraint_first_status: ConstraintFirstStatus,
    pub limitations: Vec<String>,
}

impl TopologyComparisonReport {
    pub fn validate(&self) -> Result<(), TopologyReportError> {
        self.analysis
            .validate()
            .map_err(|_| TopologyReportError::InvalidContract)?;
        if self.schema_version != SchemaVersion::new(0, 6)
            || self.analysis.baseline_topology_sha256 == self.analysis.candidate_topology_sha256
            || self.comparisons.is_empty()
            || self.comparisons.len() > 128
            || self
                .aggregate_score
                .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            || self.comparisons.iter().any(|(name, result)| {
                !valid_identifier(name) || result.policy_sha256 != self.statistical_policy_sha256
            })
            || self.limitations.is_empty()
            || self.limitations.len() > 128
            || self
                .limitations
                .iter()
                .any(|value| value.trim().is_empty() || value.len() > 4_096 || value.contains('\0'))
        {
            return Err(TopologyReportError::InvalidContract);
        }
        Ok(())
    }

    pub fn render_json(&self) -> Result<Vec<u8>, TopologyReportError> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn render_html(&self) -> Result<Vec<u8>, TopologyReportError> {
        self.validate()?;
        let mut metrics = String::new();
        for (name, value) in &self.analysis.metrics {
            write!(
                metrics,
                "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td></tr>",
                escape(name),
                escape(&format!("{:?}", value.applicability).to_ascii_lowercase()),
                value
                    .value
                    .map_or_else(|| "unavailable".to_owned(), |item| format!("{item:.6}"))
            )
            .map_err(|_| TopologyReportError::InvalidContract)?;
        }
        let mut comparisons = String::new();
        for (name, comparison) in &self.comparisons {
            let interval = comparison.paired_difference.interval.map_or_else(
                || "unavailable".to_owned(),
                |value| {
                    format!(
                        "[{:.6}, {:.6}] at {:.3}",
                        value.lower, value.upper, value.confidence
                    )
                },
            );
            write!(
                comparisons,
                "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(name),
                comparison.paired_difference.count,
                escape(&format!("{:?}", comparison.claim_strength).to_ascii_lowercase()),
                escape(&interval)
            )
            .map_err(|_| TopologyReportError::InvalidContract)?;
        }
        let limitations = self
            .limitations
            .iter()
            .map(|value| format!("<li>{}</li>", escape(value)))
            .collect::<String>();
        let html = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>HuntEval topology comparison</title></head><body><h1>Controlled topology comparison</h1><p>Results are experimental and topology-dependent. Raw metrics are authoritative; the aggregate is derived from the recorded scoring profile.</p><p>Statistical policy SHA-256: <code>{}</code>. Scoring profile SHA-256: <code>{}</code>.</p><h2>Metric vector</h2><table><thead><tr><th>Metric</th><th>Applicability</th><th>Value</th></tr></thead><tbody>{metrics}</tbody></table><h2>Policy-bound comparisons</h2><table><thead><tr><th>Metric</th><th>Paired samples</th><th>Claim strength</th><th>Interval</th></tr></thead><tbody>{comparisons}</tbody></table><h2>Limitations</h2><ul>{limitations}</ul></body></html>\n",
            self.statistical_policy_sha256, self.scoring_profile_sha256
        );
        Ok(html.into_bytes())
    }
}

fn valid_identifier(value: &str) -> bool {
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

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Error)]
pub enum TopologyReportError {
    #[error("topology comparison report is invalid")]
    InvalidContract,
    #[error("topology report JSON serialization failed")]
    Json(#[from] serde_json::Error),
}
