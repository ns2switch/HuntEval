use std::fmt::Write;

use crate::{ReportError, ReportRenderer, RunReport};

#[derive(Debug, Default, Clone, Copy)]
pub struct StaticHtmlRenderer;

impl ReportRenderer for StaticHtmlRenderer {
    fn render_run(&self, report: &RunReport) -> Result<Vec<u8>, ReportError> {
        report.validate()?;
        let mut body = String::new();
        for claim in &report.claims {
            let value = claim
                .value
                .map_or_else(|| "not applicable".into(), |value| value.to_string());
            write!(
                body,
                "<li>{}: {} <a href=\"{}\">source</a></li>",
                escape(&claim.label),
                escape(&value),
                escape(&claim.source)
            )
            .map_err(|_| ReportError::InvalidReference)?;
        }
        let mut links = String::new();
        for link in &report.artifacts {
            write!(
                links,
                "<li><a href=\"{}\">{}</a></li>",
                escape(&link.path),
                escape(&link.label)
            )
            .map_err(|_| ReportError::InvalidReference)?;
        }
        let document = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>HuntEval run report</title></head><body><h1>Run {}</h1><p>Status: {}</p><h2>Metrics</h2><ul>{body}</ul><h2>Artifacts</h2><ul>{links}</ul></body></html>\n",
            escape(report.result.run_id.as_str()),
            escape(&report.status_label)
        );
        Ok(document.into_bytes())
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
