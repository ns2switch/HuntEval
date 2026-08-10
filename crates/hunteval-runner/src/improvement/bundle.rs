use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use hunteval_domain::{SchemaVersion, Sha256Digest};
use hunteval_reporting::{
    ImprovementJsonRenderer, ImprovementReport, ImprovementStaticHtmlRenderer,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_ARTIFACTS: usize = 128;
const MAX_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImprovementBundleInput {
    pub kind: String,
    pub relative_path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementBundleArtifact {
    pub kind: String,
    pub path: String,
    pub sha256: Sha256Digest,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementBundleManifest {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub report_sha256: Sha256Digest,
    pub artifacts: Vec<ImprovementBundleArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementVerificationStatus {
    Verified,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementVerificationResult {
    pub schema_version: SchemaVersion,
    pub status: ImprovementVerificationStatus,
    pub checked_artifacts: usize,
    pub reason_codes: Vec<String>,
}

pub fn generate_improvement_bundle(
    output: &Path,
    report: &ImprovementReport,
    inputs: &[ImprovementBundleInput],
) -> Result<PathBuf, ImprovementBundleError> {
    if output.exists() || inputs.len() > MAX_ARTIFACTS.saturating_sub(2) {
        return Err(ImprovementBundleError::UnsafeOutput);
    }
    let partial = output.with_extension("partial");
    if partial.exists() {
        return Err(ImprovementBundleError::UnsafeOutput);
    }
    fs::create_dir(&partial).map_err(|_| ImprovementBundleError::Io)?;
    let result = write_bundle(&partial, report, inputs);
    if result.is_err() {
        return result.map(|_| output.to_path_buf());
    }
    fs::rename(&partial, output).map_err(|_| ImprovementBundleError::Io)?;
    Ok(output.to_path_buf())
}

fn write_bundle(
    root: &Path,
    report: &ImprovementReport,
    inputs: &[ImprovementBundleInput],
) -> Result<(), ImprovementBundleError> {
    let json = ImprovementJsonRenderer
        .render(report)
        .map_err(|_| ImprovementBundleError::InvalidReport)?;
    let html = ImprovementStaticHtmlRenderer
        .render(report)
        .map_err(|_| ImprovementBundleError::InvalidReport)?;
    let mut artifacts = Vec::new();
    write_artifact(
        root,
        "report",
        Path::new("improvement-report.json"),
        &json,
        &mut artifacts,
    )?;
    write_artifact(
        root,
        "report_html",
        Path::new("improvement-report.html"),
        &html,
        &mut artifacts,
    )?;
    for input in inputs {
        write_artifact(
            root,
            &input.kind,
            &input.relative_path,
            &input.bytes,
            &mut artifacts,
        )?;
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let report_sha256 = Sha256Digest::from_bytes(&json);
    let manifest = ImprovementBundleManifest {
        schema_version: SchemaVersion::new(0, 8),
        id: report.id.clone(),
        report_sha256,
        artifacts,
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest).map_err(|_| ImprovementBundleError::Io)?;
    bytes.push(b'\n');
    write_new(&root.join("improvement-bundle-manifest.json"), &bytes)
}

fn write_artifact(
    root: &Path,
    kind: &str,
    relative: &Path,
    bytes: &[u8],
    artifacts: &mut Vec<ImprovementBundleArtifact>,
) -> Result<(), ImprovementBundleError> {
    validate_relative(relative)?;
    if kind.is_empty()
        || kind.len() > 128
        || bytes.is_empty()
        || bytes.len() > MAX_ARTIFACT_BYTES
        || artifacts
            .iter()
            .any(|artifact| artifact.path == relative.to_string_lossy())
    {
        return Err(ImprovementBundleError::UnsafeArtifact);
    }
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| ImprovementBundleError::Io)?;
    }
    write_new(&path, bytes)?;
    artifacts.push(ImprovementBundleArtifact {
        kind: kind.to_owned(),
        path: relative.to_string_lossy().into_owned(),
        sha256: Sha256Digest::from_bytes(bytes),
        size_bytes: u64::try_from(bytes.len())
            .map_err(|_| ImprovementBundleError::UnsafeArtifact)?,
    });
    Ok(())
}

pub fn verify_improvement_bundle(root: &Path) -> ImprovementVerificationResult {
    let mut reasons = Vec::new();
    let manifest_path = root.join("improvement-bundle-manifest.json");
    let manifest = safe_read(&manifest_path).and_then(|bytes| {
        serde_json::from_slice::<ImprovementBundleManifest>(&bytes)
            .map_err(|_| ImprovementBundleError::UnsafeArtifact)
    });
    let mut checked = 0;
    match manifest {
        Ok(manifest) if manifest.schema_version == SchemaVersion::new(0, 8) => {
            if manifest.artifacts.is_empty() || manifest.artifacts.len() > MAX_ARTIFACTS {
                reasons.push("invalid_manifest".to_owned());
            }
            let mut report = None;
            let mut report_html = None;
            for artifact in &manifest.artifacts {
                let relative = Path::new(&artifact.path);
                let result = validate_relative(relative)
                    .and_then(|()| safe_read(&root.join(relative)))
                    .and_then(|bytes| {
                        if Sha256Digest::from_bytes(&bytes) == artifact.sha256
                            && u64::try_from(bytes.len()).ok() == Some(artifact.size_bytes)
                        {
                            Ok(bytes)
                        } else {
                            Err(ImprovementBundleError::UnsafeArtifact)
                        }
                    });
                match result {
                    Ok(bytes) => {
                        checked += 1;
                        if artifact.path == "improvement-report.json" {
                            if Sha256Digest::from_bytes(&bytes) != manifest.report_sha256 {
                                reasons.push("report_digest_mismatch".to_owned());
                            }
                            report = Some(bytes);
                        } else if artifact.path == "improvement-report.html" {
                            report_html = Some(bytes);
                        }
                    }
                    Err(_) => reasons.push("artifact_verification_failed".to_owned()),
                }
            }
            verify_report_projection(report, report_html, &mut reasons);
        }
        _ => reasons.push("invalid_manifest".to_owned()),
    }
    reasons.sort();
    reasons.dedup();
    ImprovementVerificationResult {
        schema_version: SchemaVersion::new(0, 8),
        status: if reasons.is_empty() {
            ImprovementVerificationStatus::Verified
        } else {
            ImprovementVerificationStatus::Rejected
        },
        checked_artifacts: checked,
        reason_codes: reasons,
    }
}

fn verify_report_projection(
    report_bytes: Option<Vec<u8>>,
    html_bytes: Option<Vec<u8>>,
    reasons: &mut Vec<String>,
) {
    let Some(report_bytes) = report_bytes else {
        reasons.push("missing_report".to_owned());
        return;
    };
    let Ok(report) = serde_json::from_slice::<ImprovementReport>(&report_bytes) else {
        reasons.push("invalid_report".to_owned());
        return;
    };
    let expected_json = ImprovementJsonRenderer.render(&report);
    let expected_html = ImprovementStaticHtmlRenderer.render(&report);
    if expected_json.as_deref() != Ok(report_bytes.as_slice()) {
        reasons.push("report_projection_mismatch".to_owned());
    }
    if !matches!(
        (expected_html.as_deref(), html_bytes.as_deref()),
        (Ok(expected), Some(actual)) if expected == actual
    ) {
        reasons.push("html_projection_mismatch".to_owned());
    }
}

fn safe_read(path: &Path) -> Result<Vec<u8>, ImprovementBundleError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| ImprovementBundleError::UnsafeArtifact)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_ARTIFACT_BYTES as u64
    {
        return Err(ImprovementBundleError::UnsafeArtifact);
    }
    fs::read(path).map_err(|_| ImprovementBundleError::Io)
}

fn validate_relative(path: &Path) -> Result<(), ImprovementBundleError> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ImprovementBundleError::UnsafeArtifact);
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), ImprovementBundleError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| ImprovementBundleError::Io)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| ImprovementBundleError::Io)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ImprovementBundleError {
    #[error("improvement output path is unsafe or already exists")]
    UnsafeOutput,
    #[error("improvement bundle artifact is unsafe, malformed, or oversized")]
    UnsafeArtifact,
    #[error("improvement report is invalid")]
    InvalidReport,
    #[error("improvement bundle I/O failed")]
    Io,
}
