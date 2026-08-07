use std::{fs, path::Path};

use hunteval_domain::{AgentId, FinalSubmission, GroundTruth, RunId, Sha256Digest};
use hunteval_evaluation::TrustedViewError;
use hunteval_protocol::{ProtocolEnvelope, ProtocolPayload, TrajectoryRecorder};
use hunteval_runner::{StoredEvaluationError, StoredEvaluationHashes, load_trusted_run_view};

#[test]
fn evaluation_input_reduces_verified_stored_artifacts_deterministically()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let messages = transcript()?;
    let ground_truth = ground_truth()?;
    write_run(temporary.path(), &messages)?;
    let first = load(
        temporary.path(),
        ground_truth.clone(),
        Sha256Digest::from_bytes(b"truth"),
    )?;
    let second = load(
        temporary.path(),
        ground_truth,
        Sha256Digest::from_bytes(b"truth"),
    )?;
    assert_eq!(first.observed(), second.observed());
    assert_eq!(first.evaluation_input(), second.evaluation_input());
    assert_eq!(first.evaluation_input().grounded_evidence_items, 1);
    assert_eq!(first.evaluation_input().valid_provenance_references, 1);
    assert_eq!(first.provenance().trajectory_event_count, 13);
    Ok(())
}

#[test]
fn evaluation_input_rejects_wrong_owner_and_stored_submission_mismatch()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let mut messages = transcript()?;
    for message in &mut messages {
        if let ProtocolPayload::EvidenceShared { agent_id, .. } = &mut message.payload {
            *agent_id = AgentId::new("supervisor")?;
        }
    }
    write_run(temporary.path(), &messages)?;
    assert!(matches!(
        load(
            temporary.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::View(
            TrustedViewError::WrongAgentOwnership
        ))
    ));

    let mismatch_root = tempfile::tempdir()?;
    let messages = transcript()?;
    write_run(mismatch_root.path(), &messages)?;
    let mut submission = terminal_submission(&messages)?;
    submission.summary = "tampered stored submission".to_owned();
    fs::write(
        mismatch_root.path().join("submission.json"),
        serde_json::to_vec(&submission)?,
    )?;
    assert!(matches!(
        load(
            mismatch_root.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::View(
            TrustedViewError::SubmissionMismatch
        ))
    ));
    Ok(())
}

#[test]
fn evaluation_input_rejects_cross_run_duplicates_incomplete_and_missing_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let cross_run = tempfile::tempdir()?;
    let mut messages = transcript()?;
    messages[5].run_id = RunId::new("run-other")?;
    write_run(cross_run.path(), &messages)?;
    assert!(matches!(
        load(
            cross_run.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::Protocol(_))
    ));

    let future_reference = tempfile::tempdir()?;
    let mut messages = transcript()?;
    let result_index = messages
        .iter()
        .position(|message| matches!(message.payload, ProtocolPayload::ToolResult { .. }))
        .ok_or_else(|| std::io::Error::other("tool result missing"))?;
    let evidence_index = messages
        .iter()
        .position(|message| matches!(message.payload, ProtocolPayload::EvidenceShared { .. }))
        .ok_or_else(|| std::io::Error::other("evidence missing"))?;
    messages.swap(result_index, evidence_index);
    write_run(future_reference.path(), &messages)?;
    assert!(matches!(
        load(
            future_reference.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::Protocol(_))
    ));

    let duplicate = tempfile::tempdir()?;
    let mut messages = transcript()?;
    messages.insert(4, messages[3].clone());
    write_run(duplicate.path(), &messages)?;
    assert!(matches!(
        load(
            duplicate.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::Protocol(_))
    ));

    let incomplete = tempfile::tempdir()?;
    let mut messages = transcript()?;
    messages.pop();
    write_run(incomplete.path(), &messages)?;
    assert!(matches!(
        load(
            incomplete.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::Protocol(_))
    ));

    let missing = tempfile::tempdir()?;
    assert!(matches!(
        load(
            missing.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::Io(_))
    ));
    Ok(())
}

#[test]
fn evaluation_input_rejects_artifact_digest_mismatch() -> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let messages = transcript()?;
    write_run(temporary.path(), &messages)?;
    let trajectory = fs::read(temporary.path().join("trajectory.jsonl"))?;
    let submission_path = temporary.path().join("submission.json");
    let submission = fs::read(&submission_path)?;
    let hashes = StoredEvaluationHashes {
        trajectory: Sha256Digest::from_bytes(trajectory),
        submission: Sha256Digest::from_bytes(&submission),
        ground_truth: Sha256Digest::from_bytes(b"truth"),
    };
    let mut tampered = submission;
    tampered.push(b' ');
    fs::write(submission_path, tampered)?;
    assert!(matches!(
        load_trusted_run_view(
            temporary.path(),
            &RunId::new("run-001")?,
            ground_truth()?,
            hashes,
            128 * 1024,
            10,
            false,
        ),
        Err(StoredEvaluationError::ArtifactDigestMismatch)
    ));
    Ok(())
}

#[cfg(unix)]
#[test]
fn evaluation_input_rejects_symlinked_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let temporary = tempfile::tempdir()?;
    let target = temporary.path().join("outside.jsonl");
    fs::write(&target, b"outside")?;
    symlink(&target, temporary.path().join("trajectory.jsonl"))?;
    fs::write(temporary.path().join("submission.json"), b"{}")?;
    assert!(matches!(
        load(
            temporary.path(),
            ground_truth()?,
            Sha256Digest::from_bytes(b"truth")
        ),
        Err(StoredEvaluationError::InvalidArtifact)
    ));
    Ok(())
}

fn load(
    root: &Path,
    ground_truth: GroundTruth,
    digest: Sha256Digest,
) -> Result<hunteval_evaluation::TrustedRunView, StoredEvaluationError> {
    let trajectory = fs::read(root.join("trajectory.jsonl")).map_err(StoredEvaluationError::Io)?;
    let submission = fs::read(root.join("submission.json")).map_err(StoredEvaluationError::Io)?;
    load_trusted_run_view(
        root,
        &RunId::new("run-001").map_err(|_| StoredEvaluationError::InvalidProjection)?,
        ground_truth,
        StoredEvaluationHashes {
            trajectory: Sha256Digest::from_bytes(trajectory),
            submission: Sha256Digest::from_bytes(submission),
            ground_truth: digest,
        },
        128 * 1024,
        10,
        false,
    )
}

fn write_run(root: &Path, messages: &[ProtocolEnvelope]) -> Result<(), Box<dyn std::error::Error>> {
    let mut recorder = TrajectoryRecorder::new();
    for message in messages {
        recorder.append(message.clone())?;
    }
    fs::write(root.join("trajectory.jsonl"), recorder.as_bytes())?;
    fs::write(
        root.join("submission.json"),
        serde_json::to_vec(&terminal_submission(messages)?)?,
    )?;
    Ok(())
}

fn terminal_submission(
    messages: &[ProtocolEnvelope],
) -> Result<FinalSubmission, Box<dyn std::error::Error>> {
    messages
        .iter()
        .find_map(|message| match &message.payload {
            ProtocolPayload::FinalSubmission { submission, .. } => Some(submission.clone()),
            _ => None,
        })
        .ok_or_else(|| std::io::Error::other("terminal submission missing").into())
}

fn transcript() -> Result<Vec<ProtocolEnvelope>, serde_json::Error> {
    serde_json::from_str(include_str!(
        "../../../examples/contracts/protocol-transcript.json"
    ))
}

fn ground_truth() -> Result<GroundTruth, serde_json::Error> {
    serde_json::from_str(include_str!(
        "../../../examples/contracts/ground-truth.json"
    ))
}
