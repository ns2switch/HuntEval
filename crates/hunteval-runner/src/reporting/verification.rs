use std::path::{Path, PathBuf};

use hunteval_reporting::{BenchmarkResult, RunReport};
use serde::{Deserialize, Serialize};

use super::{ReportGenerationError, io::read_regular_bounded};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportVerification {
    pub valid: bool,
    pub report_kind: String,
    pub checked_artifacts: usize,
    pub errors: Vec<String>,
}

pub fn verify_report(path: &Path) -> Result<ReportVerification, ReportGenerationError> {
    let (report_path, root) = resolve_report(path)?;
    let bytes = read_regular_bounded(&report_path)?;
    if let Ok(report) = serde_json::from_slice::<BenchmarkResult>(&bytes) {
        verify_benchmark(&root, &report)
    } else {
        let report: RunReport = serde_json::from_slice(&bytes)?;
        verify_run(&root, &report)
    }
}

fn resolve_report(path: &Path) -> Result<(PathBuf, PathBuf), ReportGenerationError> {
    if path.is_dir() {
        let metadata = std::fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(ReportGenerationError::InvalidInput);
        }
        let benchmark = path.join("benchmark-report.json");
        if benchmark.exists() {
            return Ok((benchmark, path.to_path_buf()));
        }
        return Ok((path.join("report.json"), path.to_path_buf()));
    }
    let root = path
        .parent()
        .ok_or(ReportGenerationError::InvalidInput)?
        .to_path_buf();
    Ok((path.to_path_buf(), root))
}

fn verify_benchmark(
    root: &Path,
    report: &BenchmarkResult,
) -> Result<ReportVerification, ReportGenerationError> {
    report.validate()?;
    let mut errors = Vec::new();
    for artifact in &report.artifacts {
        let path = root.join(&artifact.path);
        match read_regular_bounded(&path) {
            Ok(bytes) if hunteval_domain::Sha256Digest::from_bytes(&bytes) == artifact.sha256 => {}
            Ok(_) => errors.push(format!("digest_mismatch:{}", artifact.path)),
            Err(_) => errors.push(format!("unavailable_artifact:{}", artifact.path)),
        }
    }
    check_declared_digest(
        report,
        "benchmark-definition.json",
        report.benchmark_definition_sha256,
        &mut errors,
    );
    check_declared_digest(
        report,
        "benchmark-state.json",
        report.benchmark_state_sha256,
        &mut errors,
    );
    Ok(ReportVerification {
        valid: errors.is_empty(),
        report_kind: "benchmark".to_owned(),
        checked_artifacts: report.artifacts.len(),
        errors,
    })
}

fn check_declared_digest(
    report: &BenchmarkResult,
    path: &str,
    expected: hunteval_domain::Sha256Digest,
    errors: &mut Vec<String>,
) {
    if !report
        .artifacts
        .iter()
        .any(|artifact| artifact.path == path && artifact.sha256 == expected)
    {
        errors.push(format!("inconsistent_report_digest:{path}"));
    }
}

fn verify_run(
    root: &Path,
    report: &RunReport,
) -> Result<ReportVerification, ReportGenerationError> {
    report.validate()?;
    let mut errors = Vec::new();
    for artifact in &report.artifacts {
        if read_regular_bounded(&root.join(&artifact.path)).is_err() {
            errors.push(format!("unavailable_artifact:{}", artifact.path));
        }
    }
    Ok(ReportVerification {
        valid: errors.is_empty(),
        report_kind: "run".to_owned(),
        checked_artifacts: report.artifacts.len(),
        errors,
    })
}
