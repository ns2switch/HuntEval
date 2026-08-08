use super::{BenchmarkResult, BenchmarkResultError};

#[derive(Debug, Default, Clone, Copy)]
pub struct BenchmarkJsonRenderer;

impl BenchmarkJsonRenderer {
    pub fn render(&self, report: &BenchmarkResult) -> Result<Vec<u8>, BenchmarkResultError> {
        report.validate()?;
        let mut bytes = serde_json::to_vec_pretty(report)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
