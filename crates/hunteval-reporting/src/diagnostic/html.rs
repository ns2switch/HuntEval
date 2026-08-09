use std::fmt::Write;

use super::{DiagnosticClaimStage, DiagnosticReport};
use crate::ReportError;

#[derive(Debug, Default, Clone, Copy)]
pub struct DiagnosticStaticHtmlRenderer;

impl DiagnosticStaticHtmlRenderer {
    pub fn render(&self, report: &DiagnosticReport) -> Result<Vec<u8>, ReportError> {
        report.validate()?;
        let mut claims = String::new();
        for claim in &report.claims {
            write!(
                claims,
                "<article data-stage=\"{}\"><h3>{}</h3><p>{}</p><p>Status: {}</p><ul>",
                stage_label(claim.stage),
                escape(&claim.code),
                escape(&claim.summary),
                escape(&format!("{:?}", claim.validation_status).to_lowercase())
            )
            .map_err(|_| ReportError::InvalidDiagnostic)?;
            for source in &claim.sources {
                write!(claims, "<li>{}</li>", escape(&format!("{source:?}")))
                    .map_err(|_| ReportError::InvalidDiagnostic)?;
            }
            claims.push_str("</ul></article>");
        }
        let document = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>HuntEval diagnostic report</title></head><body><header><h1>Diagnostic report {}</h1></header><main><p>Scope: {:?}</p><section aria-labelledby=\"claims\"><h2 id=\"claims\">Evidence-backed claims</h2>{claims}</section><aside><h2>Limitations</h2><ul>{}</ul></aside></main></body></html>\n",
            escape(&report.report_id),
            report.scope,
            report
                .limitations
                .iter()
                .map(|item| format!("<li>{}</li>", escape(item)))
                .collect::<String>()
        );
        Ok(document.into_bytes())
    }
}

const fn stage_label(stage: DiagnosticClaimStage) -> &'static str {
    match stage {
        DiagnosticClaimStage::Observation => "observation",
        DiagnosticClaimStage::Classification => "classification",
        DiagnosticClaimStage::Hypothesis => "hypothesis",
        DiagnosticClaimStage::ExperimentResult => "experiment_result",
        DiagnosticClaimStage::ApprovedChange => "approved_change",
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
