use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCellId, BenchmarkId, RunId, SchemaVersion, Sha256Digest,
    UtcTimestamp,
};
use thiserror::Error;

use super::{
    lock::{JournalLock, prepare_root, recover_temporary_state, reject_symlink_if_present},
    model::{BenchmarkEvent, BenchmarkEventKind, BenchmarkState},
    projection::Projection,
    verifier::verify_result,
};

const JOURNAL_NAME: &str = "benchmark-events.jsonl";
const STATE_NAME: &str = "benchmark-state.json";
const TEMP_STATE_NAME: &str = ".benchmark-state.tmp";
const MAX_EVENT_BYTES: usize = 128 * 1024;

#[derive(Debug)]
pub struct BenchmarkJournal {
    root: PathBuf,
    events: File,
    projection: Projection,
    benchmark_id: BenchmarkId,
    _lock: JournalLock,
}

impl BenchmarkJournal {
    pub fn open(root: &Path, benchmark_id: BenchmarkId) -> Result<Self, BenchmarkJournalError> {
        let root = prepare_root(root)?;
        let lock = JournalLock::acquire(&root)?;
        recover_temporary_state(&root, TEMP_STATE_NAME)?;
        let journal_path = root.join(JOURNAL_NAME);
        reject_symlink_if_present(&journal_path)?;
        let events = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&journal_path)
            .map_err(BenchmarkJournalError::Io)?;
        let mut projection = Projection::new(benchmark_id.clone());
        replay(&events, &mut projection)?;
        let journal = Self {
            root,
            events,
            projection,
            benchmark_id,
            _lock: lock,
        };
        journal.write_state()?;
        Ok(journal)
    }

    pub fn append(
        &mut self,
        timestamp: UtcTimestamp,
        kind: BenchmarkEventKind,
    ) -> Result<BenchmarkEvent, BenchmarkJournalError> {
        if matches!(kind, BenchmarkEventKind::AttemptCompleted { .. }) {
            return Err(BenchmarkJournalError::UnverifiedCompletion);
        }
        self.append_verified(timestamp, kind)
    }

    pub fn complete_attempt(
        &mut self,
        timestamp: UtcTimestamp,
        cell_id: BenchmarkCellId,
        attempt_id: BenchmarkAttemptId,
        run_id: RunId,
        result_path: &Path,
    ) -> Result<BenchmarkEvent, BenchmarkJournalError> {
        let result_sha256 = verify_result(result_path, cell_id, &run_id)?;
        self.append_verified(
            timestamp,
            BenchmarkEventKind::AttemptCompleted {
                cell_id,
                attempt_id,
                run_id,
                result_sha256,
            },
        )
    }

    pub fn interrupt_running(
        &mut self,
        timestamp: UtcTimestamp,
    ) -> Result<Vec<BenchmarkEvent>, BenchmarkJournalError> {
        let attempts = self.projection.running_attempts();
        let mut events = Vec::with_capacity(attempts.len());
        for (cell_id, attempt_id) in attempts {
            events.push(self.append_verified(
                timestamp,
                BenchmarkEventKind::AttemptInterrupted {
                    cell_id,
                    attempt_id,
                    reason_code: "controller_interrupted".to_owned(),
                },
            )?);
        }
        Ok(events)
    }

    #[must_use]
    pub fn state(&self) -> Option<BenchmarkState> {
        self.projection.state()
    }

    fn append_verified(
        &mut self,
        timestamp: UtcTimestamp,
        kind: BenchmarkEventKind,
    ) -> Result<BenchmarkEvent, BenchmarkJournalError> {
        let event = BenchmarkEvent {
            schema_version: SchemaVersion::new(0, 4),
            benchmark_id: self.benchmark_id.clone(),
            sequence: self.projection.next_sequence()?,
            previous_event_sha256: self.projection.previous_digest(),
            timestamp,
            kind,
        };
        let mut line = serde_json::to_vec(&event).map_err(BenchmarkJournalError::Serialize)?;
        line.push(b'\n');
        if line.len() > MAX_EVENT_BYTES {
            return Err(BenchmarkJournalError::EventTooLarge);
        }
        let digest = Sha256Digest::from_bytes(&line);
        let mut candidate = self.projection.clone();
        candidate.apply(&event, digest)?;
        self.events
            .write_all(&line)
            .map_err(BenchmarkJournalError::Io)?;
        self.events.sync_data().map_err(BenchmarkJournalError::Io)?;
        self.projection = candidate;
        self.write_state()?;
        Ok(event)
    }

    fn write_state(&self) -> Result<(), BenchmarkJournalError> {
        let Some(state) = self.projection.state() else {
            return Ok(());
        };
        let temporary = self.root.join(TEMP_STATE_NAME);
        reject_symlink_if_present(&self.root.join(STATE_NAME))?;
        reject_symlink_if_present(&temporary)?;
        if temporary.exists() {
            fs::remove_file(&temporary).map_err(BenchmarkJournalError::Io)?;
        }
        let mut bytes =
            serde_json::to_vec_pretty(&state).map_err(BenchmarkJournalError::Serialize)?;
        bytes.push(b'\n');
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(BenchmarkJournalError::Io)?;
        file.write_all(&bytes).map_err(BenchmarkJournalError::Io)?;
        file.sync_all().map_err(BenchmarkJournalError::Io)?;
        fs::rename(temporary, self.root.join(STATE_NAME)).map_err(BenchmarkJournalError::Io)?;
        File::open(&self.root)
            .and_then(|directory| directory.sync_all())
            .map_err(BenchmarkJournalError::Io)
    }
}

fn replay(events: &File, projection: &mut Projection) -> Result<(), BenchmarkJournalError> {
    let mut reader = BufReader::new(events.try_clone().map_err(BenchmarkJournalError::Io)?);
    loop {
        let mut line = Vec::new();
        let read = reader
            .by_ref()
            .take((MAX_EVENT_BYTES + 1) as u64)
            .read_until(b'\n', &mut line)
            .map_err(BenchmarkJournalError::Io)?;
        if read == 0 {
            break;
        }
        if line.len() > MAX_EVENT_BYTES || !line.ends_with(b"\n") {
            return Err(BenchmarkJournalError::MalformedJournal);
        }
        let bytes = &line[..line.len() - 1];
        validate_event_fields(bytes)?;
        let event: BenchmarkEvent =
            serde_json::from_slice(bytes).map_err(|_| BenchmarkJournalError::MalformedJournal)?;
        projection.apply(&event, Sha256Digest::from_bytes(&line))?;
    }
    Ok(())
}

fn validate_event_fields(bytes: &[u8]) -> Result<(), BenchmarkJournalError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| BenchmarkJournalError::MalformedJournal)?;
    let object = value
        .as_object()
        .ok_or(BenchmarkJournalError::MalformedJournal)?;
    let event_type = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or(BenchmarkJournalError::MalformedJournal)?;
    let variant_fields: &[&str] = match event_type {
        "benchmark_started" | "benchmark_completed" => &[],
        "cell_queued" => &["cell_id"],
        "attempt_started" => &["cell_id", "attempt_id"],
        "attempt_interrupted" | "attempt_failed" => &["cell_id", "attempt_id", "reason_code"],
        "attempt_completed" => &["cell_id", "attempt_id", "run_id", "result_sha256"],
        "cell_non_comparable" => &["cell_id", "reason_code"],
        _ => return Err(BenchmarkJournalError::MalformedJournal),
    };
    const COMMON: &[&str] = &[
        "schema_version",
        "benchmark_id",
        "sequence",
        "previous_event_sha256",
        "timestamp",
        "type",
    ];
    if object
        .keys()
        .any(|key| !COMMON.contains(&key.as_str()) && !variant_fields.contains(&key.as_str()))
    {
        return Err(BenchmarkJournalError::MalformedJournal);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum BenchmarkJournalError {
    #[error("benchmark journal transition is invalid")]
    InvalidTransition,
    #[error("benchmark completion requires a verified result artifact")]
    UnverifiedCompletion,
    #[error("benchmark result artifact is invalid or has a mismatched identity")]
    InvalidResult,
    #[error("benchmark journal is malformed or truncated")]
    MalformedJournal,
    #[error("benchmark journal event exceeds its byte limit")]
    EventTooLarge,
    #[error("benchmark journal path is unsafe")]
    UnsafePath,
    #[error("benchmark journal is owned by another controller")]
    Locked,
    #[error("benchmark journal I/O failed")]
    Io(#[source] std::io::Error),
    #[error("benchmark journal serialization failed")]
    Serialize(#[source] serde_json::Error),
}
