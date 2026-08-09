use super::DiagnosticReport;
use crate::ReportError;

#[derive(Debug, Default, Clone, Copy)]
pub struct DiagnosticJsonRenderer;

impl DiagnosticJsonRenderer {
    pub fn render(&self, report: &DiagnosticReport) -> Result<Vec<u8>, ReportError> {
        report.validate()?;
        let mut bytes = serde_json::to_vec_pretty(report).map_err(ReportError::Serialize)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
