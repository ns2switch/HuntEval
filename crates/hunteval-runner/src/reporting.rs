use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
};

use hunteval_domain::RunResult;
use hunteval_reporting::{
    JsonRenderer, ReportError, ReportFormat, ReportRenderer, RunReport, StaticHtmlRenderer,
};
use thiserror::Error;

const MAX_RESULT_BYTES: u64 = 16 * 1024 * 1024;

pub fn generate_report(run_root: &Path, format: ReportFormat) -> Result<(), ReportGenerationError> {
    let result_path = run_root.join("result.json");
    let metadata = fs::symlink_metadata(&result_path).map_err(ReportGenerationError::Io)?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_RESULT_BYTES {
        return Err(ReportGenerationError::InvalidInput);
    }
    let bytes = fs::read(result_path).map_err(ReportGenerationError::Io)?;
    let result: RunResult = serde_json::from_slice(&bytes).map_err(ReportGenerationError::Json)?;
    let report = RunReport::from_result(result).map_err(ReportGenerationError::Report)?;
    let (name, rendered) = match format {
        ReportFormat::Json => (
            "report.json",
            JsonRenderer
                .render_run(&report)
                .map_err(ReportGenerationError::Report)?,
        ),
        ReportFormat::Html => (
            "report.html",
            StaticHtmlRenderer
                .render_run(&report)
                .map_err(ReportGenerationError::Report)?,
        ),
    };
    write_atomic(run_root, name, &rendered)
}

fn write_atomic(root: &Path, name: &str, bytes: &[u8]) -> Result<(), ReportGenerationError> {
    let temporary = root.join(format!(".{name}.partial"));
    let destination = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(ReportGenerationError::Io)?;
    let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
    if let Err(error) = write_result {
        let _ignored = fs::remove_file(&temporary);
        return Err(ReportGenerationError::Io(error));
    }
    fs::rename(temporary, destination).map_err(ReportGenerationError::Io)
}

#[derive(Debug, Error)]
pub enum ReportGenerationError {
    #[error("run result must be a regular file within the size limit")]
    InvalidInput,
    #[error("report I/O failed: {0}")]
    Io(io::Error),
    #[error("run result JSON is invalid: {0}")]
    Json(serde_json::Error),
    #[error("report is invalid: {0}")]
    Report(ReportError),
}
