mod aggregation;
mod benchmark;
mod io;
mod verification;

use std::{io as std_io, path::Path};

use hunteval_domain::RunResult;
use hunteval_reporting::{
    BenchmarkJsonRenderer, BenchmarkResultError, BenchmarkStaticHtmlRenderer, JsonRenderer,
    ReportError, ReportFormat, ReportRenderer, RunReport, StaticHtmlRenderer,
};
use thiserror::Error;

pub use verification::{ReportVerification, verify_report};

use self::io::{read_regular_bounded, write_atomic};

pub fn generate_report(root: &Path, format: ReportFormat) -> Result<(), ReportGenerationError> {
    validate_root(root)?;
    if regular_file_exists(&root.join("benchmark-definition.json"))? {
        generate_benchmark_report(root, format)
    } else {
        generate_run_report(root, format)
    }
}

fn validate_root(root: &Path) -> Result<(), ReportGenerationError> {
    let metadata = std::fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(ReportGenerationError::InvalidInput);
    }
    Ok(())
}

fn generate_run_report(root: &Path, format: ReportFormat) -> Result<(), ReportGenerationError> {
    let bytes = read_regular_bounded(&root.join("result.json"))?;
    let result: RunResult = serde_json::from_slice(&bytes)?;
    let report = RunReport::from_result(result)?;
    let (name, rendered) = match format {
        ReportFormat::Json => ("report.json", JsonRenderer.render_run(&report)?),
        ReportFormat::Html => ("report.html", StaticHtmlRenderer.render_run(&report)?),
    };
    write_atomic(root, name, &rendered)
}

fn generate_benchmark_report(
    root: &Path,
    format: ReportFormat,
) -> Result<(), ReportGenerationError> {
    let report = benchmark::build(root)?;
    let normalized = BenchmarkJsonRenderer.render(&report)?;
    write_atomic(root, "benchmark-report.json", &normalized)?;
    if format == ReportFormat::Html {
        let html = BenchmarkStaticHtmlRenderer.render(&report)?;
        write_atomic(root, "benchmark-report.html", &html)?;
    }
    Ok(())
}

fn regular_file_exists(path: &Path) -> Result<bool, ReportGenerationError> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(true),
        Ok(_) => Err(ReportGenerationError::InvalidInput),
        Err(error) if error.kind() == std_io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(ReportGenerationError::Io(error)),
    }
}

#[derive(Debug, Error)]
pub enum ReportGenerationError {
    #[error("report input must be a bounded regular file")]
    InvalidInput,
    #[error("benchmark state is unavailable")]
    MissingState,
    #[error("artifact digest mismatch: {0}")]
    DigestMismatch(std::path::PathBuf),
    #[error("report I/O failed: {0}")]
    Io(#[from] std_io::Error),
    #[error("report JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("run report is invalid: {0}")]
    Report(#[from] ReportError),
    #[error("benchmark report is invalid: {0}")]
    BenchmarkReport(#[from] BenchmarkResultError),
    #[error("benchmark service failed: {0}")]
    BenchmarkService(#[from] crate::BenchmarkServiceError),
    #[error("benchmark journal failed: {0}")]
    Journal(#[from] crate::BenchmarkJournalError),
    #[error("benchmark definition failed: {0}")]
    Definition(#[from] hunteval_domain::BenchmarkDefinitionError),
    #[error("artifact hashing failed: {0}")]
    Hashing(#[from] crate::HashingError),
    #[error("statistical calculation failed: {0}")]
    Statistics(#[from] hunteval_statistics::StatisticsError),
}
