use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use hunteval_domain::{BottleneckAnalysis, BottleneckObservations, RunDiagnosis, Sha256Digest};
use hunteval_evaluation::{canonical_taxonomy, classifier_registry_digest};
use hunteval_reporting::DiagnosticReport;
use serde::Serialize;

use super::{
    DiagnosticBundleManifest,
    artifact_validation::{
        diagnosis_sources_resolve, report_references_match, valid_bottleneck_analysis,
        valid_run_diagnosis,
    },
};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticVerificationStatus {
    Verified,
    Invalid,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticVerificationResult {
    pub schema_version: String,
    pub status: DiagnosticVerificationStatus,
    pub checked_artifacts: usize,
    pub reasons: Vec<String>,
}

#[must_use]
pub fn verify_diagnostic_bundle(root: &Path) -> DiagnosticVerificationResult {
    let mut reasons = Vec::new();
    let Some(manifest_bytes) = read(root, "diagnostic-bundle-manifest.json", MAX_MANIFEST_BYTES)
    else {
        return invalid("manifest_unavailable");
    };
    let manifest: DiagnosticBundleManifest = match serde_json::from_slice(&manifest_bytes) {
        Ok(value) => value,
        Err(_) => return invalid("malformed_manifest"),
    };
    if manifest.schema_version != hunteval_domain::SchemaVersion::new(0, 7) {
        return DiagnosticVerificationResult {
            schema_version: "0.7".into(),
            status: DiagnosticVerificationStatus::Unsupported,
            checked_artifacts: 0,
            reasons: vec!["unsupported_schema_version".into()],
        };
    }
    if manifest.artifacts.is_empty() || manifest.artifacts.len() > 4096 {
        return invalid("invalid_artifact_inventory");
    }
    let expected_taxonomy = canonical_taxonomy()
        .ok()
        .and_then(|taxonomy| taxonomy.digest().ok());
    if expected_taxonomy != Some(manifest.taxonomy_sha256)
        || manifest.classifier_registry_sha256 != classifier_registry_digest()
    {
        reasons.push("diagnostic_registry_mismatch".into());
    }
    let mut paths = BTreeSet::new();
    let inventory: BTreeMap<_, _> = manifest
        .artifacts
        .iter()
        .map(|artifact| (artifact.path.clone(), artifact.sha256))
        .collect();
    if inventory.get("diagnostic-taxonomy.json").copied() != Some(manifest.taxonomy_sha256) {
        reasons.push("taxonomy_artifact_mismatch".into());
    }
    for artifact in &manifest.artifacts {
        if !safe_relative(&artifact.path) || !paths.insert(artifact.path.clone()) {
            reasons.push("unsafe_artifact_path".into());
            continue;
        }
        let Some(bytes) = read(root, &artifact.path, MAX_ARTIFACT_BYTES) else {
            reasons.push("artifact_unavailable".into());
            continue;
        };
        if bytes.len() as u64 != artifact.size_bytes
            || Sha256Digest::from_bytes(&bytes) != artifact.sha256
        {
            reasons.push("artifact_digest_mismatch".into());
            continue;
        }
        if !valid_media_type(&artifact.path, &artifact.media_type) {
            reasons.push("invalid_artifact_media_type".into());
        }
        if artifact.path.ends_with("diagnostic-report.json") {
            match serde_json::from_slice::<DiagnosticReport>(&bytes) {
                Ok(report)
                    if report.validate().is_ok()
                        && report_references_match(
                            root,
                            &report,
                            &manifest.artifacts,
                            &artifact.path,
                        ) => {}
                _ => reasons.push("invalid_diagnostic_report".into()),
            }
        } else if artifact.path.ends_with("run-diagnosis.json") {
            match serde_json::from_slice::<RunDiagnosis>(&bytes) {
                Ok(diagnosis)
                    if diagnosis.taxonomy_sha256 == manifest.taxonomy_sha256
                        && diagnosis.classifier_registry_sha256
                            == manifest.classifier_registry_sha256
                        && valid_run_diagnosis(&diagnosis)
                        && diagnosis_sources_resolve(root, &artifact.path, &diagnosis) => {}
                _ => reasons.push("invalid_run_diagnosis".into()),
            }
        } else if artifact.path.ends_with("bottleneck-observations.json") {
            if serde_json::from_slice::<BottleneckObservations>(&bytes).is_err() {
                reasons.push("invalid_bottleneck_observations".into());
            }
        } else if artifact.path.ends_with("bottleneck-analysis.json")
            && !serde_json::from_slice::<BottleneckAnalysis>(&bytes)
                .is_ok_and(|analysis| valid_bottleneck_analysis(&analysis))
        {
            reasons.push("invalid_bottleneck_analysis".into());
        }
    }
    if manifest.root_reports.iter().any(|path| {
        !inventory.contains_key(path)
            || !(path.ends_with("diagnostic-report.json")
                || path.ends_with("diagnostic-report.html"))
    }) || manifest.root_reports.is_empty()
        || !manifest
            .root_reports
            .iter()
            .any(|path| path.ends_with("diagnostic-report.json"))
    {
        reasons.push("invalid_root_report".into());
    }
    let expected_id = serde_json::to_vec(&manifest.artifacts)
        .ok()
        .map(|bytes| format!("diagnostic-bundle:{}", Sha256Digest::from_bytes(bytes)));
    if expected_id.as_deref() != Some(manifest.bundle_id.as_str()) {
        reasons.push("bundle_identity_mismatch".into());
    }
    reasons.sort();
    reasons.dedup();
    DiagnosticVerificationResult {
        schema_version: "0.7".into(),
        status: if reasons.is_empty() {
            DiagnosticVerificationStatus::Verified
        } else {
            DiagnosticVerificationStatus::Invalid
        },
        checked_artifacts: manifest.artifacts.len(),
        reasons,
    }
}

fn valid_media_type(path: &str, media_type: &str) -> bool {
    (path.ends_with(".json") && media_type == "application/json")
        || (path.ends_with(".html") && media_type == "text/html")
        || (path.ends_with(".jsonl") && media_type == "application/x-ndjson")
}

fn read(root: &Path, relative: &str, maximum: u64) -> Option<Vec<u8>> {
    if !safe_relative(relative) {
        return None;
    }
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return None;
    }
    fs::read(path).ok()
}
fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|item| matches!(item, Component::Normal(_)))
}
fn invalid(reason: &str) -> DiagnosticVerificationResult {
    DiagnosticVerificationResult {
        schema_version: "0.7".into(),
        status: DiagnosticVerificationStatus::Invalid,
        checked_artifacts: 0,
        reasons: vec![reason.into()],
    }
}
