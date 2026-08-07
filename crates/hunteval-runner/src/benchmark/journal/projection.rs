use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCellId, BenchmarkId, SchemaVersion, Sha256Digest,
};

use super::model::{
    BenchmarkCellState, BenchmarkCellStatus, BenchmarkEvent, BenchmarkEventKind, BenchmarkState,
};
use super::storage::BenchmarkJournalError;

#[derive(Debug, Clone)]
pub(super) struct Projection {
    benchmark_id: BenchmarkId,
    started: bool,
    completed: bool,
    attempts: BTreeSet<BenchmarkAttemptId>,
    cells: BTreeMap<BenchmarkCellId, BenchmarkCellState>,
    last_sequence: u64,
    last_event_sha256: Option<Sha256Digest>,
}

impl Projection {
    pub(super) fn new(benchmark_id: BenchmarkId) -> Self {
        Self {
            benchmark_id,
            started: false,
            completed: false,
            attempts: BTreeSet::new(),
            cells: BTreeMap::new(),
            last_sequence: 0,
            last_event_sha256: None,
        }
    }

    pub(super) fn apply(
        &mut self,
        event: &BenchmarkEvent,
        line_sha256: Sha256Digest,
    ) -> Result<(), BenchmarkJournalError> {
        if event.schema_version != SchemaVersion::new(0, 4)
            || event.benchmark_id != self.benchmark_id
            || event.sequence
                != self
                    .last_sequence
                    .checked_add(1)
                    .ok_or(BenchmarkJournalError::InvalidTransition)?
            || event.previous_event_sha256 != self.last_event_sha256
            || self.completed
        {
            return Err(BenchmarkJournalError::InvalidTransition);
        }
        self.apply_kind(&event.kind)?;
        self.last_sequence = event.sequence;
        self.last_event_sha256 = Some(line_sha256);
        Ok(())
    }

    fn apply_kind(&mut self, kind: &BenchmarkEventKind) -> Result<(), BenchmarkJournalError> {
        match kind {
            BenchmarkEventKind::BenchmarkStarted if !self.started => self.started = true,
            BenchmarkEventKind::CellQueued { cell_id } if self.started => {
                if self.cells.contains_key(cell_id) {
                    return Err(BenchmarkJournalError::InvalidTransition);
                }
                self.cells.insert(*cell_id, pending(*cell_id));
            }
            BenchmarkEventKind::AttemptStarted {
                cell_id,
                attempt_id,
            } => {
                if !self.attempts.insert(attempt_id.clone()) {
                    return Err(BenchmarkJournalError::InvalidTransition);
                }
                let cell = self.cell_mut(cell_id)?;
                if !matches!(
                    cell.status,
                    BenchmarkCellStatus::Pending | BenchmarkCellStatus::Failed
                ) {
                    return Err(BenchmarkJournalError::InvalidTransition);
                }
                cell.status = BenchmarkCellStatus::Running;
                cell.attempt_ids.push(attempt_id.clone());
                clear_outcome(cell);
            }
            BenchmarkEventKind::AttemptInterrupted {
                cell_id,
                attempt_id,
                reason_code,
            }
            | BenchmarkEventKind::AttemptFailed {
                cell_id,
                attempt_id,
                reason_code,
            } => {
                require_reason(reason_code)?;
                let cell = self.running_attempt(cell_id, attempt_id)?;
                cell.status = BenchmarkCellStatus::Failed;
                cell.reason_code = Some(reason_code.clone());
            }
            BenchmarkEventKind::AttemptCompleted {
                cell_id,
                attempt_id,
                run_id,
                result_sha256,
            } => {
                let cell = self.running_attempt(cell_id, attempt_id)?;
                cell.status = BenchmarkCellStatus::Completed;
                cell.run_id = Some(run_id.clone());
                cell.result_sha256 = Some(*result_sha256);
                cell.reason_code = None;
            }
            BenchmarkEventKind::CellNonComparable {
                cell_id,
                reason_code,
            } => {
                require_reason(reason_code)?;
                let cell = self.cell_mut(cell_id)?;
                if cell.status == BenchmarkCellStatus::Running {
                    return Err(BenchmarkJournalError::InvalidTransition);
                }
                cell.status = BenchmarkCellStatus::NonComparable;
                cell.reason_code = Some(reason_code.clone());
            }
            BenchmarkEventKind::BenchmarkCompleted
                if self.started
                    && !self.cells.is_empty()
                    && self.cells.values().all(|cell| {
                        matches!(
                            cell.status,
                            BenchmarkCellStatus::Completed
                                | BenchmarkCellStatus::Failed
                                | BenchmarkCellStatus::NonComparable
                        )
                    }) =>
            {
                self.completed = true;
            }
            _ => return Err(BenchmarkJournalError::InvalidTransition),
        }
        Ok(())
    }

    fn cell_mut(
        &mut self,
        cell_id: &BenchmarkCellId,
    ) -> Result<&mut BenchmarkCellState, BenchmarkJournalError> {
        self.cells
            .get_mut(cell_id)
            .ok_or(BenchmarkJournalError::InvalidTransition)
    }

    fn running_attempt(
        &mut self,
        cell_id: &BenchmarkCellId,
        attempt_id: &BenchmarkAttemptId,
    ) -> Result<&mut BenchmarkCellState, BenchmarkJournalError> {
        let cell = self.cell_mut(cell_id)?;
        if cell.status != BenchmarkCellStatus::Running
            || cell.attempt_ids.last() != Some(attempt_id)
        {
            return Err(BenchmarkJournalError::InvalidTransition);
        }
        Ok(cell)
    }

    pub(super) fn state(&self) -> Option<BenchmarkState> {
        Some(BenchmarkState {
            schema_version: SchemaVersion::new(0, 4),
            benchmark_id: self.benchmark_id.clone(),
            last_sequence: self.last_sequence,
            last_event_sha256: self.last_event_sha256?,
            cells: self.cells.values().cloned().collect(),
        })
    }

    pub(super) fn running_attempts(&self) -> Vec<(BenchmarkCellId, BenchmarkAttemptId)> {
        self.cells
            .values()
            .filter(|cell| cell.status == BenchmarkCellStatus::Running)
            .filter_map(|cell| {
                cell.attempt_ids
                    .last()
                    .cloned()
                    .map(|attempt| (cell.cell_id, attempt))
            })
            .collect()
    }

    pub(super) fn next_sequence(&self) -> Result<u64, BenchmarkJournalError> {
        self.last_sequence
            .checked_add(1)
            .ok_or(BenchmarkJournalError::InvalidTransition)
    }

    pub(super) const fn previous_digest(&self) -> Option<Sha256Digest> {
        self.last_event_sha256
    }
}

fn pending(cell_id: BenchmarkCellId) -> BenchmarkCellState {
    BenchmarkCellState {
        cell_id,
        status: BenchmarkCellStatus::Pending,
        attempt_ids: Vec::new(),
        run_id: None,
        result_sha256: None,
        reason_code: None,
    }
}

fn clear_outcome(cell: &mut BenchmarkCellState) {
    cell.run_id = None;
    cell.result_sha256 = None;
    cell.reason_code = None;
}

fn require_reason(reason: &str) -> Result<(), BenchmarkJournalError> {
    if reason.is_empty()
        || reason.len() > 256
        || !reason
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(BenchmarkJournalError::InvalidTransition);
    }
    Ok(())
}
