use std::path::Path;

use hunteval_domain::{FinalSubmission, RunResult, Sha256Digest};
use hunteval_protocol::{ProtocolPayload, StoredEvent, replay_trajectory};
use serde::Serialize;

use crate::RunManifest;

mod reader;

use reader::Verifier;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_TRAJECTORY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Verified,
    Incomplete,
    Invalid,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationCheck {
    pub check: String,
    pub passed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunVerificationResult {
    pub schema_version: String,
    pub status: VerificationStatus,
    pub private_evaluation: String,
    pub checked_artifacts: usize,
    pub checks: Vec<VerificationCheck>,
}

#[must_use]
pub fn verify_run(root: &Path) -> RunVerificationResult {
    let mut verifier = Verifier::new();
    let Some(manifest_bytes) = verifier.read(root, "manifest.json", MAX_MANIFEST_BYTES) else {
        return verifier.finish(VerificationStatus::Invalid);
    };
    let manifest: RunManifest = match serde_json::from_slice(&manifest_bytes) {
        Ok(value) => value,
        Err(_) => {
            verifier.failed("manifest", "malformed_manifest");
            return verifier.finish(VerificationStatus::Invalid);
        }
    };
    let version = (
        manifest.schema_version.major(),
        manifest.schema_version.minor(),
    );
    if !matches!(version, (0, 3..=5)) {
        verifier.failed("manifest", "unsupported_schema_version");
        return verifier.finish(VerificationStatus::Unsupported);
    }
    verifier.passed("manifest");
    if manifest.partial {
        verifier.failed("completion", "partial_run");
        return verifier.finish(VerificationStatus::Incomplete);
    }
    verifier.passed("completion");

    let Some(trajectory) = verifier.read(root, "trajectory.jsonl", MAX_TRAJECTORY_BYTES) else {
        return verifier.finish(VerificationStatus::Invalid);
    };
    let Some(submission) = verifier.read(root, "submission.json", MAX_ARTIFACT_BYTES) else {
        return verifier.finish(VerificationStatus::Invalid);
    };
    let Some(metrics) = verifier.read(root, "metrics.json", MAX_ARTIFACT_BYTES) else {
        return verifier.finish(VerificationStatus::Invalid);
    };

    verify_digest(&mut verifier, &manifest, "trajectory", &trajectory);
    verify_digest(&mut verifier, &manifest, "submission", &submission);
    verify_digest(&mut verifier, &manifest, "metrics", &metrics);
    verify_trajectory(&mut verifier, &trajectory, &submission);
    verify_json(&mut verifier, "metrics_json", &metrics);
    verify_execution_policy(&mut verifier, root, &manifest);
    verify_optional_json(
        &mut verifier,
        root,
        &manifest,
        "aggregate_score",
        "aggregate-score.json",
    );
    verify_optional_result(&mut verifier, root, &manifest);

    let status = if verifier.checks.iter().all(|check| check.passed) {
        VerificationStatus::Verified
    } else {
        VerificationStatus::Invalid
    };
    verifier.finish(status)
}

fn verify_execution_policy(verifier: &mut Verifier, root: &Path, manifest: &RunManifest) {
    if manifest.schema_version.minor() < 5 {
        verifier.checks.push(VerificationCheck {
            check: "execution_policy".to_owned(),
            passed: true,
            reason: Some("not_applicable_legacy".to_owned()),
        });
        return;
    }
    let Some(bytes) = verifier.read(root, "execution-policy.json", MAX_ARTIFACT_BYTES) else {
        return;
    };
    verify_digest(verifier, manifest, "execution_policy", &bytes);
    match serde_json::from_slice::<hunteval_sandbox::ResolvedExecutionPolicy>(&bytes) {
        Ok(policy) if policy.validate().is_ok() => verifier.passed("execution_policy"),
        _ => verifier.failed("execution_policy", "invalid_execution_policy"),
    }
}

fn verify_optional_json(
    verifier: &mut Verifier,
    root: &Path,
    manifest: &RunManifest,
    hash_name: &str,
    file_name: &str,
) {
    if !root.join(file_name).exists() {
        return;
    }
    let Some(bytes) = verifier.read(root, file_name, MAX_ARTIFACT_BYTES) else {
        return;
    };
    verify_digest(verifier, manifest, hash_name, &bytes);
    verify_json(verifier, &format!("{hash_name}_json"), &bytes);
}

fn verify_digest(verifier: &mut Verifier, manifest: &RunManifest, name: &str, bytes: &[u8]) {
    let check = format!("{name}_digest");
    match manifest.hashes.get(name) {
        Some(expected) if *expected == Sha256Digest::from_bytes(bytes) => verifier.passed(&check),
        Some(_) => verifier.failed(&check, "digest_mismatch"),
        None => verifier.failed(&check, "missing_digest"),
    }
}

fn verify_trajectory(verifier: &mut Verifier, trajectory: &[u8], submission: &[u8]) {
    if replay_trajectory(trajectory, 128 * 1024).is_err() {
        verifier.failed("trajectory_replay", "invalid_trajectory");
        return;
    }
    verifier.passed("trajectory_replay");
    let stored: FinalSubmission = match serde_json::from_slice(submission) {
        Ok(value) => value,
        Err(_) => {
            verifier.failed("submission_equivalence", "malformed_submission");
            return;
        }
    };
    let terminal = trajectory
        .split_inclusive(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<StoredEvent>(&line[..line.len() - 1]).ok())
        .find_map(|event| match event.envelope.payload {
            ProtocolPayload::FinalSubmission { submission, .. } => Some(submission),
            _ => None,
        });
    if terminal.as_ref() == Some(&stored) {
        verifier.passed("submission_equivalence");
    } else {
        verifier.failed("submission_equivalence", "submission_mismatch");
    }
}

fn verify_json(verifier: &mut Verifier, check: &str, bytes: &[u8]) {
    if serde_json::from_slice::<serde_json::Value>(bytes).is_ok() {
        verifier.passed(check);
    } else {
        verifier.failed(check, "malformed_json");
    }
}

fn verify_optional_result(verifier: &mut Verifier, root: &Path, manifest: &RunManifest) {
    let path = root.join("result.json");
    if !path.exists() {
        return;
    }
    let Some(bytes) = verifier.read(root, "result.json", MAX_ARTIFACT_BYTES) else {
        return;
    };
    verify_digest(verifier, manifest, "result", &bytes);
    if result_is_consistent(&bytes, manifest) {
        verifier.passed("result_consistency");
    } else {
        verifier.failed("result_consistency", "invalid_result");
    }
}

fn result_is_consistent(bytes: &[u8], manifest: &RunManifest) -> bool {
    if let Ok(result) = serde_json::from_slice::<RunResult>(bytes) {
        return result.run_id == manifest.run_id && result.validate().is_ok();
    }
    let Ok(result) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    if result
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("0.4")
        || result.get("run_id").and_then(serde_json::Value::as_str)
            != Some(manifest.run_id.as_str())
        || result
            .get("cell_id")
            .and_then(serde_json::Value::as_str)
            .is_none()
    {
        return false;
    }
    let Some(hashes) = result
        .get("artifact_hashes")
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    ["trajectory", "submission", "metrics", "aggregate_score"]
        .into_iter()
        .all(|name| {
            hashes.get(name).and_then(serde_json::Value::as_str)
                == manifest
                    .hashes
                    .get(name)
                    .map(ToString::to_string)
                    .as_deref()
        })
}
