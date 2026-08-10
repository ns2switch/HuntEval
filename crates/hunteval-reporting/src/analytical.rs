use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::{ReportError, html::escape};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalReportMatch {
    pub source_id: String,
    pub source_kind: String,
    pub artifact_sha256: String,
    pub field: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyticalReport {
    pub schema_version: String,
    pub query_sha256: String,
    pub index_sha256: String,
    pub matches: Vec<AnalyticalReportMatch>,
    pub limitations: Vec<String>,
}

impl AnalyticalReport {
    pub fn render_html(&self) -> Result<Vec<u8>, ReportError> {
        let mut matches = String::new();
        for item in &self.matches {
            write!(
                matches,
                "<li><strong>{}</strong> [{}] {} <code>{}</code><p>{}</p></li>",
                escape(&item.source_id),
                escape(&item.source_kind),
                escape(&item.field),
                escape(&item.artifact_sha256),
                escape(&item.excerpt),
            )
            .map_err(|_| ReportError::InvalidReference)?;
        }
        let mut limitations = String::new();
        for limitation in &self.limitations {
            write!(limitations, "<li>{}</li>", escape(limitation))
                .map_err(|_| ReportError::InvalidReference)?;
        }
        Ok(format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>HuntEval analytical result</title></head><body><h1>Analytical result</h1><p>Query: <code>{}</code></p><p>Index: <code>{}</code></p><h2>Matches</h2><ul>{matches}</ul><h2>Limitations</h2><ul>{limitations}</ul></body></html>\n",
            escape(&self.query_sha256),
            escape(&self.index_sha256),
        )
        .into_bytes())
    }
}
