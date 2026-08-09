use std::{collections::BTreeMap, fs, path::Path};

use hunteval_domain::{FinalSubmission, RunId, SchemaVersion, Sha256Digest};
use hunteval_protocol::{ProtocolEnvelope, ProtocolPayload, TrajectoryRecorder};
use hunteval_runner::{RunManifest, VerificationStatus, verify_run};

#[test]
fn verifies_and_detects_tampering_without_private_inputs() -> Result<(), Box<dyn std::error::Error>>
{
    let root = tempfile::tempdir()?;
    write_completed_run(root.path())?;
    let verified = verify_run(root.path());
    assert_eq!(verified.status, VerificationStatus::Verified);
    assert_eq!(verified.private_evaluation, "not_checked");

    fs::write(root.path().join("metrics.json"), b"{}\nchanged")?;
    let tampered = verify_run(root.path());
    assert_eq!(tampered.status, VerificationStatus::Invalid);
    assert!(tampered.checks.iter().any(|check| {
        check.check == "metrics_digest" && check.reason.as_deref() == Some("digest_mismatch")
    }));
    Ok(())
}

#[test]
fn identifies_partial_runs_without_requiring_missing_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let manifest = RunManifest {
        schema_version: SchemaVersion::new(0, 4),
        run_id: RunId::new("run-partial")?,
        hashes: BTreeMap::new(),
        partial: true,
    };
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec(&manifest)?,
    )?;
    let result = verify_run(root.path());
    assert_eq!(result.status, VerificationStatus::Incomplete);
    Ok(())
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir()?;
    write_completed_run(root.path())?;
    let target = root.path().join("outside.json");
    fs::write(&target, b"{}")?;
    fs::remove_file(root.path().join("metrics.json"))?;
    symlink(target, root.path().join("metrics.json"))?;
    let result = verify_run(root.path());
    assert_eq!(result.status, VerificationStatus::Invalid);
    assert!(result.checks.iter().any(|check| {
        check.check == "metrics.json" && check.reason.as_deref() == Some("unsafe_artifact")
    }));
    Ok(())
}

fn write_completed_run(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let messages: Vec<ProtocolEnvelope> = serde_json::from_str(include_str!(
        "../../../examples/contracts/protocol-transcript.json"
    ))?;
    let mut recorder = TrajectoryRecorder::new();
    let mut submission: Option<FinalSubmission> = None;
    for message in messages {
        if let ProtocolPayload::FinalSubmission {
            submission: value, ..
        } = &message.payload
        {
            submission = Some(value.clone());
        }
        recorder.append(message)?;
    }
    let submission = submission.ok_or("fixture has no submission")?;
    let trajectory = recorder.as_bytes();
    let mut submission_bytes = serde_json::to_vec_pretty(&submission)?;
    submission_bytes.push(b'\n');
    let metrics = b"{}\n";
    fs::write(root.join("trajectory.jsonl"), trajectory)?;
    fs::write(root.join("submission.json"), &submission_bytes)?;
    fs::write(root.join("metrics.json"), metrics)?;
    let manifest = RunManifest {
        schema_version: SchemaVersion::new(0, 4),
        run_id: RunId::new("run-001")?,
        hashes: BTreeMap::from([
            (
                "trajectory".to_owned(),
                Sha256Digest::from_bytes(trajectory),
            ),
            (
                "submission".to_owned(),
                Sha256Digest::from_bytes(&submission_bytes),
            ),
            ("metrics".to_owned(), Sha256Digest::from_bytes(metrics)),
        ]),
        partial: false,
    };
    fs::write(root.join("manifest.json"), serde_json::to_vec(&manifest)?)?;
    Ok(())
}
