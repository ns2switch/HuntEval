use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use hunteval_domain::{
    RecommendationEvent, RecommendationState, RecommendationStatusV08, SchemaVersion, Sha256Digest,
    UtcTimestamp, event_digest, project_recommendation,
};
use thiserror::Error;

const MAX_EVENT_BYTES: u64 = 64 * 1024;
const MAX_EVENTS: usize = 4096;

#[derive(Debug)]
pub struct RecommendationJournal {
    path: PathBuf,
    lock_path: PathBuf,
    events: Vec<RecommendationEvent>,
}

impl RecommendationJournal {
    pub fn open(root: &Path) -> Result<Self, RecommendationJournalError> {
        fs::create_dir_all(root).map_err(|_| RecommendationJournalError::Io)?;
        let lock_path = root.join("recommendation.lock");
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    RecommendationJournalError::ConcurrentWriter
                } else {
                    RecommendationJournalError::Io
                }
            })?;
        let path = root.join("recommendation-events.jsonl");
        let events = match read_events(&path) {
            Ok(events) => events,
            Err(error) => {
                let _ = fs::remove_file(&lock_path);
                return Err(error);
            }
        };
        if !events.is_empty() && project_recommendation(&events).is_err() {
            let _ = fs::remove_file(&lock_path);
            return Err(RecommendationJournalError::InvalidHistory);
        }
        Ok(Self {
            path,
            lock_path,
            events,
        })
    }

    pub fn append(
        &mut self,
        event: RecommendationEvent,
    ) -> Result<RecommendationState, RecommendationJournalError> {
        let mut candidate = self.events.clone();
        candidate.push(event.clone());
        let state = project_recommendation(&candidate)
            .map_err(|_| RecommendationJournalError::InvalidTransition)?;
        let mut bytes =
            serde_json::to_vec(&event).map_err(|_| RecommendationJournalError::Serialize)?;
        if bytes.len() as u64 > MAX_EVENT_BYTES {
            return Err(RecommendationJournalError::EventTooLarge);
        }
        bytes.push(b'\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|_| RecommendationJournalError::Io)?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| RecommendationJournalError::Io)?;
        self.events.push(event);
        Ok(state)
    }

    pub fn state(&self) -> Result<Option<RecommendationState>, RecommendationJournalError> {
        if self.events.is_empty() {
            Ok(None)
        } else {
            project_recommendation(&self.events)
                .map(Some)
                .map_err(|_| RecommendationJournalError::InvalidHistory)
        }
    }

    #[must_use]
    pub fn next_link(&self) -> (u64, Option<Sha256Digest>) {
        let sequence = u64::try_from(self.events.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let digest = self
            .events
            .last()
            .and_then(|event| event_digest(event).ok());
        (sequence, digest)
    }
}

impl Drop for RecommendationJournal {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

pub fn stale_candidate_invalidation(
    state: &RecommendationState,
    current_candidate_sha256: Sha256Digest,
    timestamp: UtcTimestamp,
    caused_by_artifact_sha256: Sha256Digest,
) -> Option<RecommendationEvent> {
    let eligible_state = matches!(
        state.status,
        RecommendationStatusV08::Testing
            | RecommendationStatusV08::Validated
            | RecommendationStatusV08::Approved
            | RecommendationStatusV08::Adopted
    );
    if !eligible_state || current_candidate_sha256 == state.candidate_artifact_sha256 {
        return None;
    }
    Some(RecommendationEvent {
        schema_version: SchemaVersion::new(0, 8),
        recommendation_id: state.recommendation_id.clone(),
        sequence: state.last_sequence.saturating_add(1),
        timestamp,
        previous_event_sha256: Some(state.last_event_sha256),
        event: RecommendationStatusV08::Invalidated,
        candidate_artifact_sha256: current_candidate_sha256,
        caused_by_artifact_sha256,
        reason_code: "candidate_artifact_changed".to_owned(),
        validation_decision_sha256: state.validation_decision_sha256,
        human_decision_sha256: state.human_decision_sha256,
        adoption_record_sha256: state.adoption_record_sha256,
    })
}

fn read_events(path: &Path) -> Result<Vec<RecommendationEvent>, RecommendationJournalError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| RecommendationJournalError::Io)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_EVENT_BYTES.saturating_mul(MAX_EVENTS as u64)
    {
        return Err(RecommendationJournalError::InvalidHistory);
    }
    let mut events = Vec::new();
    for line in
        BufReader::new(File::open(path).map_err(|_| RecommendationJournalError::Io)?).lines()
    {
        let line = line.map_err(|_| RecommendationJournalError::Io)?;
        if line.len() as u64 > MAX_EVENT_BYTES || events.len() >= MAX_EVENTS {
            return Err(RecommendationJournalError::EventTooLarge);
        }
        events.push(
            serde_json::from_str(&line).map_err(|_| RecommendationJournalError::InvalidHistory)?,
        );
    }
    Ok(events)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecommendationJournalError {
    #[error("another recommendation writer owns the journal")]
    ConcurrentWriter,
    #[error("recommendation history is malformed or its hash chain is invalid")]
    InvalidHistory,
    #[error("recommendation transition is invalid")]
    InvalidTransition,
    #[error("recommendation event exceeds its bound")]
    EventTooLarge,
    #[error("recommendation serialization failed")]
    Serialize,
    #[error("recommendation journal I/O failed")]
    Io,
}
