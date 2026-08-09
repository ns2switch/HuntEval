use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use hunteval_domain::{RunDiagnosis, SchemaVersion, Sha256Digest};
use hunteval_evaluation::{
    canonical_taxonomy, classifier_registry_digest, classify_verified, evaluate_bottlenecks,
};
use hunteval_reporting::{DiagnosticJsonRenderer, DiagnosticStaticHtmlRenderer};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    bottleneck_projection::bottleneck_observations,
    projection::diagnostic_input,
    report_projection::{ReportInputs, build_report, hypotheses},
};
use crate::{RunManifest, VerificationStatus, load_observed_run_for_diagnosis, verify_run};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_METRICS_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TRAJECTORY_BYTES: u64 = 128 * 1024 * 1024;
const MAX_SUBMISSION_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticBundleArtifact {
    pub path: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticBundleManifest {
    pub schema_version: SchemaVersion,
    pub bundle_id: String,
    pub source_revision: Sha256Digest,
    pub taxonomy_sha256: Sha256Digest,
    pub classifier_registry_sha256: Sha256Digest,
    pub artifacts: Vec<DiagnosticBundleArtifact>,
    pub root_reports: BTreeSet<String>,
    pub limitations: BTreeSet<String>,
}

pub fn generate_run_diagnosis(
    run_root: &Path,
    output: &Path,
) -> Result<PathBuf, DiagnosticGenerationError> {
    validate_roots(run_root, output)?;
    let verification = verify_run(run_root);
    if verification.status != VerificationStatus::Verified {
        return Err(DiagnosticGenerationError::UnverifiedRun);
    }
    let manifest_bytes = read_regular(&run_root.join("manifest.json"), MAX_MANIFEST_BYTES)?;
    let manifest: RunManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|_| DiagnosticGenerationError::MalformedSource)?;
    let trajectory = manifest
        .hashes
        .get("trajectory")
        .copied()
        .ok_or(DiagnosticGenerationError::MalformedSource)?;
    let submission = manifest
        .hashes
        .get("submission")
        .copied()
        .ok_or(DiagnosticGenerationError::MalformedSource)?;
    let observed = load_observed_run_for_diagnosis(
        run_root,
        &manifest.run_id,
        trajectory,
        submission,
        128 * 1024,
    )?;
    let metrics_bytes = read_regular(&run_root.join("metrics.json"), MAX_METRICS_BYTES)?;
    let trajectory_bytes = read_regular(&run_root.join("trajectory.jsonl"), MAX_TRAJECTORY_BYTES)?;
    let submission_bytes = read_regular(&run_root.join("submission.json"), MAX_SUBMISSION_BYTES)?;
    let metric_names = metric_names(&metrics_bytes)?;
    let input = diagnostic_input(
        &observed.observed,
        trajectory,
        Sha256Digest::from_bytes(&manifest_bytes),
        metric_names,
    );
    let (classifications, omissions) =
        classify_verified(&input).map_err(|_| DiagnosticGenerationError::Classification)?;
    let taxonomy = canonical_taxonomy().map_err(|_| DiagnosticGenerationError::Classification)?;
    let taxonomy_sha256 = taxonomy
        .digest()
        .map_err(|_| DiagnosticGenerationError::Classification)?;
    let taxonomy_bytes =
        serde_json::to_vec(&taxonomy).map_err(DiagnosticGenerationError::Serialize)?;
    let registry_sha256 = classifier_registry_digest();
    let hypotheses = hypotheses(&classifications);
    let run_diagnosis = RunDiagnosis {
        schema_version: SchemaVersion::new(0, 7),
        run_id: manifest.run_id.clone(),
        run_manifest_sha256: Sha256Digest::from_bytes(&manifest_bytes),
        taxonomy_sha256,
        classifier_registry_sha256: registry_sha256,
        classifications,
        omissions,
        recommendation_hypotheses: hypotheses,
        limitations: [
            "registered_rules_only".into(),
            "absence_is_not_proof".into(),
        ]
        .into_iter()
        .collect(),
    };
    let observations = bottleneck_observations(&observed.observed, trajectory);
    let duration = measured_duration(&observed.observed.event_timestamps);
    let agents = input.artifacts.agent_ids.len() as u64;
    let bottlenecks = evaluate_bottlenecks(
        &observations,
        observed.observed.tasks.len() as u64,
        agents,
        observed.observed.actions.len() as u64,
        duration,
    )?;
    write_bundle(BundleInputs {
        source: run_root,
        output,
        source_manifest: &manifest,
        source_manifest_bytes: &manifest_bytes,
        metrics_bytes: &metrics_bytes,
        trajectory_bytes: &trajectory_bytes,
        submission_bytes: &submission_bytes,
        taxonomy_bytes: &taxonomy_bytes,
        diagnosis: &run_diagnosis,
        observations: &observations,
        bottlenecks: &bottlenecks,
    })
}

struct BundleInputs<'a> {
    source: &'a Path,
    output: &'a Path,
    source_manifest: &'a RunManifest,
    source_manifest_bytes: &'a [u8],
    metrics_bytes: &'a [u8],
    trajectory_bytes: &'a [u8],
    submission_bytes: &'a [u8],
    taxonomy_bytes: &'a [u8],
    diagnosis: &'a RunDiagnosis,
    observations: &'a hunteval_domain::BottleneckObservations,
    bottlenecks: &'a hunteval_domain::BottleneckAnalysis,
}

fn write_bundle(input: BundleInputs<'_>) -> Result<PathBuf, DiagnosticGenerationError> {
    let parent = input
        .output
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(DiagnosticGenerationError::Io)?;
    validate_materialized_destination(input.source, parent)?;
    let name = input
        .output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(DiagnosticGenerationError::UnsafeOutput)?;
    let partial = parent.join(format!(".{name}.partial"));
    if input.output.exists() || partial.exists() {
        return Err(DiagnosticGenerationError::AlreadyExists);
    }
    fs::create_dir(&partial).map_err(DiagnosticGenerationError::Io)?;
    let diagnosis_bytes = json_bytes(input.diagnosis)?;
    let observations_bytes = json_bytes(input.observations)?;
    let bottleneck_bytes = json_bytes(input.bottlenecks)?;
    let report = build_report(ReportInputs {
        manifest: input.source_manifest,
        manifest_bytes: input.source_manifest_bytes,
        metrics_bytes: input.metrics_bytes,
        diagnosis: input.diagnosis,
        diagnosis_bytes: &diagnosis_bytes,
        observation_bytes: &observations_bytes,
        bottleneck_bytes: &bottleneck_bytes,
        bottlenecks: input.bottlenecks,
    });
    let report_json = DiagnosticJsonRenderer.render(&report)?;
    let report_html = DiagnosticStaticHtmlRenderer.render(&report)?;
    let files = [
        ("run-diagnosis.json", "application/json", diagnosis_bytes),
        (
            "bottleneck-observations.json",
            "application/json",
            observations_bytes,
        ),
        (
            "bottleneck-analysis.json",
            "application/json",
            bottleneck_bytes,
        ),
        (
            "manifest.json",
            "application/json",
            input.source_manifest_bytes.to_vec(),
        ),
        (
            "metrics.json",
            "application/json",
            input.metrics_bytes.to_vec(),
        ),
        (
            "trajectory.jsonl",
            "application/x-ndjson",
            input.trajectory_bytes.to_vec(),
        ),
        (
            "submission.json",
            "application/json",
            input.submission_bytes.to_vec(),
        ),
        (
            "diagnostic-taxonomy.json",
            "application/json",
            input.taxonomy_bytes.to_vec(),
        ),
        ("diagnostic-report.json", "application/json", report_json),
        ("diagnostic-report.html", "text/html", report_html),
    ];
    let mut artifacts = Vec::new();
    for (path, media_type, bytes) in files {
        write_new(&partial.join(path), &bytes)?;
        artifacts.push(DiagnosticBundleArtifact {
            path: path.into(),
            media_type: media_type.into(),
            size_bytes: bytes.len() as u64,
            sha256: Sha256Digest::from_bytes(&bytes),
        });
    }
    let bundle_id = format!(
        "diagnostic-bundle:{}",
        Sha256Digest::from_bytes(
            serde_json::to_vec(&artifacts).map_err(DiagnosticGenerationError::Serialize)?
        )
    );
    let manifest = DiagnosticBundleManifest {
        schema_version: SchemaVersion::new(0, 7),
        bundle_id,
        source_revision: Sha256Digest::from_bytes(env!("CARGO_PKG_VERSION")),
        taxonomy_sha256: input.diagnosis.taxonomy_sha256,
        classifier_registry_sha256: input.diagnosis.classifier_registry_sha256,
        artifacts,
        root_reports: [
            "diagnostic-report.json".into(),
            "diagnostic-report.html".into(),
        ]
        .into_iter()
        .collect(),
        limitations: input.diagnosis.limitations.clone(),
    };
    write_new(
        &partial.join("diagnostic-bundle-manifest.json"),
        &json_bytes(&manifest)?,
    )?;
    fs::rename(&partial, input.output).map_err(DiagnosticGenerationError::Io)?;
    Ok(input.output.to_path_buf())
}

pub(super) fn metric_names(bytes: &[u8]) -> Result<BTreeSet<String>, DiagnosticGenerationError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| DiagnosticGenerationError::MalformedSource)?;
    let object = value
        .get("raw_metrics")
        .and_then(serde_json::Value::as_object)
        .or_else(|| value.as_object())
        .ok_or(DiagnosticGenerationError::MalformedSource)?;
    Ok(object.keys().cloned().collect())
}

fn measured_duration(timestamps: &BTreeMap<u64, hunteval_domain::UtcTimestamp>) -> Option<u64> {
    let start = timestamps.first_key_value()?.1.as_offset_date_time();
    let end = timestamps.last_key_value()?.1.as_offset_date_time();
    u64::try_from((end - start).whole_milliseconds()).ok()
}

fn validate_roots(source: &Path, output: &Path) -> Result<(), DiagnosticGenerationError> {
    if output.as_os_str().is_empty() || output == Path::new("/") || output.starts_with(source) {
        return Err(DiagnosticGenerationError::UnsafeOutput);
    }
    Ok(())
}

pub(super) fn validate_materialized_destination(
    source: &Path,
    parent: &Path,
) -> Result<(), DiagnosticGenerationError> {
    let metadata = fs::symlink_metadata(parent).map_err(DiagnosticGenerationError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(DiagnosticGenerationError::UnsafeOutput);
    }
    let source = fs::canonicalize(source).map_err(DiagnosticGenerationError::Io)?;
    let parent = fs::canonicalize(parent).map_err(DiagnosticGenerationError::Io)?;
    if parent.starts_with(source) {
        return Err(DiagnosticGenerationError::UnsafeOutput);
    }
    Ok(())
}

fn read_regular(path: &Path, maximum: u64) -> Result<Vec<u8>, DiagnosticGenerationError> {
    let metadata = fs::symlink_metadata(path).map_err(DiagnosticGenerationError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(DiagnosticGenerationError::MalformedSource);
    }
    fs::read(path).map_err(DiagnosticGenerationError::Io)
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, DiagnosticGenerationError> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(DiagnosticGenerationError::Serialize)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), DiagnosticGenerationError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(DiagnosticGenerationError::Io)?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(DiagnosticGenerationError::Io)
}

#[derive(Debug, Error)]
pub enum DiagnosticGenerationError {
    #[error("diagnostic source run is not independently verified")]
    UnverifiedRun,
    #[error("diagnostic source artifact is malformed, unsafe, or oversized")]
    MalformedSource,
    #[error("diagnostic classification failed")]
    Classification,
    #[error("diagnostic output path is unsafe or mutates the source run")]
    UnsafeOutput,
    #[error("diagnostic output already exists")]
    AlreadyExists,
    #[error("diagnostic I/O failed: {0}")]
    Io(std::io::Error),
    #[error("diagnostic serialization failed: {0}")]
    Serialize(serde_json::Error),
    #[error("stored diagnostic input failed validation: {0}")]
    Stored(#[from] crate::StoredEvaluationError),
    #[error("bottleneck analysis failed: {0}")]
    Bottleneck(#[from] hunteval_evaluation::BottleneckError),
    #[error("diagnostic report failed validation: {0}")]
    Report(#[from] hunteval_reporting::ReportError),
}
