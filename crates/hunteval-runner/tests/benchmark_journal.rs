use std::{fs, io::Write, path::Path, str::FromStr};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCellId, BenchmarkId, RunId, Sha256Digest, UtcTimestamp,
};
use hunteval_runner::{
    BenchmarkCellStatus, BenchmarkEventKind, BenchmarkJournal, BenchmarkJournalError,
};

fn timestamp() -> Result<UtcTimestamp, serde_json::Error> {
    serde_json::from_str("\"2026-08-07T00:00:00Z\"")
}

fn cell(byte: char) -> Result<BenchmarkCellId, Box<dyn std::error::Error>> {
    Ok(BenchmarkCellId::from_str(&format!(
        "cell:{}",
        byte.to_string().repeat(64)
    ))?)
}

#[test]
fn benchmark_journal_replays_to_identical_projection() -> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().join("journal");
    let benchmark_id = BenchmarkId::new("benchmark-journal")?;
    let cell_id = cell('a')?;
    let attempt_id = BenchmarkAttemptId::new("attempt-001")?;
    let run_id = RunId::new("run-001")?;
    let mut journal = BenchmarkJournal::open(&root, benchmark_id.clone())?;
    assert!(journal.state().is_none());
    journal.append(timestamp()?, BenchmarkEventKind::BenchmarkStarted)?;
    journal.append(timestamp()?, BenchmarkEventKind::CellQueued { cell_id })?;
    journal.append(
        timestamp()?,
        BenchmarkEventKind::AttemptStarted {
            cell_id,
            attempt_id: attempt_id.clone(),
        },
    )?;
    let result = root.join("result.json");
    fs::write(
        &result,
        serde_json::to_vec(&serde_json::json!({
            "cell_id": cell_id,
            "run_id": run_id,
            "status": "completed"
        }))?,
    )?;
    let completed = journal.complete_attempt(
        timestamp()?,
        cell_id,
        attempt_id,
        RunId::new("run-001")?,
        &result,
    )?;
    let expected_digest = Sha256Digest::from_bytes(fs::read(&result)?);
    assert!(matches!(
        completed.kind,
        BenchmarkEventKind::AttemptCompleted { result_sha256, .. }
            if result_sha256 == expected_digest
    ));
    journal.append(timestamp()?, BenchmarkEventKind::BenchmarkCompleted)?;
    let first_state = fs::read(root.join("benchmark-state.json"))?;
    let state = journal
        .state()
        .ok_or_else(|| std::io::Error::other("projection is unavailable"))?;
    assert_eq!(state.cells[0].status, BenchmarkCellStatus::Completed);
    drop(journal);

    let replayed = BenchmarkJournal::open(&root, benchmark_id)?;
    assert_eq!(first_state, fs::read(root.join("benchmark-state.json"))?);
    assert_eq!(replayed.state(), Some(state));
    assert!(matches!(
        replayed.state().and_then(|value| value.cells.first().cloned()),
        Some(cell) if cell.result_sha256 == Some(expected_digest)
    ));
    Ok(())
}

#[test]
fn benchmark_journal_enforces_attempt_transitions_and_resume()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().join("resume");
    let benchmark_id = BenchmarkId::new("benchmark-resume")?;
    let cell_id = cell('b')?;
    let first_attempt = BenchmarkAttemptId::new("attempt-first")?;
    let mut journal = BenchmarkJournal::open(&root, benchmark_id.clone())?;
    journal.append(timestamp()?, BenchmarkEventKind::BenchmarkStarted)?;
    journal.append(timestamp()?, BenchmarkEventKind::CellQueued { cell_id })?;
    journal.append(
        timestamp()?,
        BenchmarkEventKind::AttemptStarted {
            cell_id,
            attempt_id: first_attempt.clone(),
        },
    )?;
    assert!(matches!(
        journal.append(
            timestamp()?,
            BenchmarkEventKind::AttemptFailed {
                cell_id,
                attempt_id: BenchmarkAttemptId::new("wrong-attempt")?,
                reason_code: "process_failure".to_owned(),
            }
        ),
        Err(BenchmarkJournalError::InvalidTransition)
    ));
    drop(journal);

    let mut resumed = BenchmarkJournal::open(&root, benchmark_id)?;
    let interrupted = resumed.interrupt_running(timestamp()?)?;
    assert_eq!(interrupted.len(), 1);
    let second_attempt = BenchmarkAttemptId::new("attempt-second")?;
    resumed.append(
        timestamp()?,
        BenchmarkEventKind::AttemptStarted {
            cell_id,
            attempt_id: second_attempt.clone(),
        },
    )?;
    resumed.append(
        timestamp()?,
        BenchmarkEventKind::AttemptFailed {
            cell_id,
            attempt_id: second_attempt.clone(),
            reason_code: "process_failure".to_owned(),
        },
    )?;
    assert!(matches!(
        resumed.append(
            timestamp()?,
            BenchmarkEventKind::AttemptFailed {
                cell_id,
                attempt_id: second_attempt,
                reason_code: "duplicate_terminal".to_owned(),
            }
        ),
        Err(BenchmarkJournalError::InvalidTransition)
    ));
    let state = resumed
        .state()
        .ok_or_else(|| std::io::Error::other("projection is unavailable"))?;
    assert_eq!(state.cells[0].attempt_ids.len(), 2);
    assert_eq!(state.cells[0].status, BenchmarkCellStatus::Failed);
    Ok(())
}

#[test]
fn benchmark_journal_rejects_untrusted_storage_and_unverified_results()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().join("untrusted");
    let benchmark_id = BenchmarkId::new("benchmark-untrusted")?;
    let cell_id = cell('c')?;
    let attempt_id = BenchmarkAttemptId::new("attempt-untrusted")?;
    let mut journal = BenchmarkJournal::open(&root, benchmark_id.clone())?;
    journal.append(timestamp()?, BenchmarkEventKind::BenchmarkStarted)?;
    journal.append(timestamp()?, BenchmarkEventKind::CellQueued { cell_id })?;
    journal.append(
        timestamp()?,
        BenchmarkEventKind::AttemptStarted {
            cell_id,
            attempt_id: attempt_id.clone(),
        },
    )?;
    assert!(matches!(
        journal.append(
            timestamp()?,
            BenchmarkEventKind::AttemptCompleted {
                cell_id,
                attempt_id: attempt_id.clone(),
                run_id: RunId::new("run-forged")?,
                result_sha256: Sha256Digest::from_bytes(b"forged"),
            }
        ),
        Err(BenchmarkJournalError::UnverifiedCompletion)
    ));
    let mismatched = root.join("mismatched.json");
    fs::write(
        &mismatched,
        serde_json::to_vec(&serde_json::json!({
            "cell_id": cell('d')?,
            "run_id": "run-forged"
        }))?,
    )?;
    assert!(matches!(
        journal.complete_attempt(
            timestamp()?,
            cell_id,
            attempt_id,
            RunId::new("run-forged")?,
            &mismatched
        ),
        Err(BenchmarkJournalError::InvalidResult)
    ));
    let second = BenchmarkJournal::open(&root, benchmark_id.clone());
    assert!(matches!(second, Err(BenchmarkJournalError::Locked)));
    drop(journal);

    let mut events = OpenAppend::new(&root.join("benchmark-events.jsonl"))?;
    events.write_all(b"{")?;
    events.sync_all()?;
    assert!(matches!(
        BenchmarkJournal::open(&root, benchmark_id),
        Err(BenchmarkJournalError::MalformedJournal)
    ));
    Ok(())
}

#[test]
fn benchmark_journal_recovers_stale_lock_and_snapshot_temp()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().join("recovery");
    fs::create_dir_all(&root)?;
    fs::write(root.join(".benchmark.lock"), b"4294967294\n")?;
    fs::write(root.join(".benchmark-state.tmp"), b"partial")?;
    let mut journal = BenchmarkJournal::open(&root, BenchmarkId::new("benchmark-recovery")?)?;
    journal.append(timestamp()?, BenchmarkEventKind::BenchmarkStarted)?;
    assert!(!root.join(".benchmark-state.tmp").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn benchmark_journal_rejects_symlinked_storage() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let temporary = tempfile::tempdir()?;
    let root = temporary.path().join("symlinked");
    fs::create_dir_all(&root)?;
    symlink("/etc/passwd", root.join("benchmark-events.jsonl"))?;
    assert!(matches!(
        BenchmarkJournal::open(&root, BenchmarkId::new("benchmark-symlink")?),
        Err(BenchmarkJournalError::UnsafePath)
    ));
    Ok(())
}

struct OpenAppend(fs::File);

impl OpenAppend {
    fn new(path: &Path) -> Result<Self, std::io::Error> {
        fs::OpenOptions::new().append(true).open(path).map(Self)
    }
}

impl Write for OpenAppend {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, std::io::Error> {
        self.0.write(buffer)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.0.flush()
    }
}

impl OpenAppend {
    fn sync_all(&self) -> Result<(), std::io::Error> {
        self.0.sync_all()
    }
}
