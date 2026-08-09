mod projection;

use std::{
    fs::{self, File},
    io::{Read, Take},
    path::Path,
};

use hunteval_domain::{GroundTruth, RunId, Sha256Digest};
use hunteval_evaluation::{
    EvaluationProvenance, ObservedRun, TrustedRunInput, TrustedRunView, TrustedViewError,
};
use hunteval_protocol::{ProtocolError, StoredEvent, replay_trajectory};
use thiserror::Error;

use projection::ReplayProjection;

const MAX_TRAJECTORY_BYTES: u64 = 256 * 1024 * 1024;
const MAX_SUBMISSION_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticObservedRun {
    pub observed: ObservedRun,
    pub trajectory_sha256: Sha256Digest,
    pub submission_sha256: Sha256Digest,
    pub trajectory_event_count: u64,
}

pub fn load_observed_run_for_diagnosis(
    run_root: &Path,
    expected_run_id: &RunId,
    expected_trajectory: Sha256Digest,
    expected_submission: Sha256Digest,
    maximum_line_bytes: usize,
) -> Result<DiagnosticObservedRun, StoredEvaluationError> {
    let trajectory =
        read_bounded_regular(&run_root.join("trajectory.jsonl"), MAX_TRAJECTORY_BYTES)?;
    let submission_bytes =
        read_bounded_regular(&run_root.join("submission.json"), MAX_SUBMISSION_BYTES)?;
    if Sha256Digest::from_bytes(&trajectory) != expected_trajectory
        || Sha256Digest::from_bytes(&submission_bytes) != expected_submission
    {
        return Err(StoredEvaluationError::ArtifactDigestMismatch);
    }
    let replay = replay_trajectory(&trajectory, maximum_line_bytes)?;
    let mut projection = ReplayProjection::new(expected_run_id.clone());
    for line in trajectory.split_inclusive(|byte| *byte == b'\n') {
        let Some(json) = line.strip_suffix(b"\n") else {
            return Err(StoredEvaluationError::InvalidArtifact);
        };
        let event: StoredEvent =
            serde_json::from_slice(json).map_err(|_| StoredEvaluationError::InvalidArtifact)?;
        projection.apply(event)?;
    }
    let (observed, terminal_submission) = projection.finish()?;
    let submission = serde_json::from_slice(&submission_bytes)
        .map_err(|_| StoredEvaluationError::InvalidArtifact)?;
    if terminal_submission != submission {
        return Err(StoredEvaluationError::InvalidProjection);
    }
    Ok(DiagnosticObservedRun {
        observed,
        trajectory_sha256: replay.trajectory_sha256,
        submission_sha256: expected_submission,
        trajectory_event_count: replay.event_count,
    })
}
pub fn load_trusted_run_view(
    run_root: &Path,
    expected_run_id: &RunId,
    ground_truth: GroundTruth,
    expected_hashes: StoredEvaluationHashes,
    maximum_line_bytes: usize,
    tool_call_limit: u64,
    benign_scored_episode: bool,
) -> Result<TrustedRunView, StoredEvaluationError> {
    let trajectory =
        read_bounded_regular(&run_root.join("trajectory.jsonl"), MAX_TRAJECTORY_BYTES)?;
    let submission_bytes =
        read_bounded_regular(&run_root.join("submission.json"), MAX_SUBMISSION_BYTES)?;
    let submission = serde_json::from_slice(&submission_bytes)
        .map_err(|_| StoredEvaluationError::InvalidArtifact)?;
    if Sha256Digest::from_bytes(&trajectory) != expected_hashes.trajectory
        || Sha256Digest::from_bytes(&submission_bytes) != expected_hashes.submission
    {
        return Err(StoredEvaluationError::ArtifactDigestMismatch);
    }
    let replay = replay_trajectory(&trajectory, maximum_line_bytes)?;
    let mut projection = ReplayProjection::new(expected_run_id.clone());
    for line in trajectory.split_inclusive(|byte| *byte == b'\n') {
        let event: StoredEvent = serde_json::from_slice(&line[..line.len() - 1])
            .map_err(|_| StoredEvaluationError::InvalidArtifact)?;
        projection.apply(event)?;
    }
    let (observed, terminal_submission) = projection.finish()?;
    let provenance = EvaluationProvenance {
        run_id: expected_run_id.clone(),
        trajectory_sha256: replay.trajectory_sha256,
        submission_sha256: Sha256Digest::from_bytes(&submission_bytes),
        ground_truth_sha256: expected_hashes.ground_truth,
        trajectory_event_count: replay.event_count,
    };
    TrustedRunView::reduce(TrustedRunInput {
        observed,
        submission,
        terminal_submission,
        ground_truth,
        provenance,
        tool_call_limit,
        benign_scored_episode,
    })
    .map_err(StoredEvaluationError::View)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoredEvaluationHashes {
    pub trajectory: Sha256Digest,
    pub submission: Sha256Digest,
    pub ground_truth: Sha256Digest,
}

fn read_bounded_regular(path: &Path, maximum: u64) -> Result<Vec<u8>, StoredEvaluationError> {
    let metadata = fs::symlink_metadata(path).map_err(StoredEvaluationError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return Err(StoredEvaluationError::InvalidArtifact);
    }
    let mut bytes = Vec::new();
    let file = File::open(path).map_err(StoredEvaluationError::Io)?;
    read_all(file.take(maximum + 1), &mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(StoredEvaluationError::InvalidArtifact);
    }
    Ok(bytes)
}

fn read_all(mut file: Take<File>, bytes: &mut Vec<u8>) -> Result<(), StoredEvaluationError> {
    file.read_to_end(bytes)
        .map(|_| ())
        .map_err(StoredEvaluationError::Io)
}

#[derive(Debug, Error)]
pub enum StoredEvaluationError {
    #[error("stored evaluation artifact is missing or unreadable")]
    Io(#[source] std::io::Error),
    #[error("stored evaluation artifact is malformed, oversized, or unsafe")]
    InvalidArtifact,
    #[error("stored evaluation artifact digest does not match trusted metadata")]
    ArtifactDigestMismatch,
    #[error("stored trajectory replay failed: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("stored replay projection is invalid")]
    InvalidProjection,
    #[error("trusted evaluation view is invalid: {0}")]
    View(#[from] TrustedViewError),
}
