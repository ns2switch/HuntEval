use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCellId, BenchmarkId, RunId, SchemaVersion, Sha256Digest,
    UtcTimestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkEvent {
    pub schema_version: SchemaVersion,
    pub benchmark_id: BenchmarkId,
    pub sequence: u64,
    pub previous_event_sha256: Option<Sha256Digest>,
    pub timestamp: UtcTimestamp,
    #[serde(flatten)]
    pub kind: BenchmarkEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BenchmarkEventKind {
    BenchmarkStarted,
    CellQueued {
        cell_id: BenchmarkCellId,
    },
    AttemptStarted {
        cell_id: BenchmarkCellId,
        attempt_id: BenchmarkAttemptId,
    },
    AttemptInterrupted {
        cell_id: BenchmarkCellId,
        attempt_id: BenchmarkAttemptId,
        reason_code: String,
    },
    AttemptCompleted {
        cell_id: BenchmarkCellId,
        attempt_id: BenchmarkAttemptId,
        run_id: RunId,
        result_sha256: Sha256Digest,
    },
    AttemptFailed {
        cell_id: BenchmarkCellId,
        attempt_id: BenchmarkAttemptId,
        reason_code: String,
    },
    CellNonComparable {
        cell_id: BenchmarkCellId,
        reason_code: String,
    },
    BenchmarkCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCellStatus {
    Pending,
    Running,
    Completed,
    Failed,
    NonComparable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCellState {
    pub cell_id: BenchmarkCellId,
    pub status: BenchmarkCellStatus,
    pub attempt_ids: Vec<BenchmarkAttemptId>,
    pub run_id: Option<RunId>,
    pub result_sha256: Option<Sha256Digest>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkState {
    pub schema_version: SchemaVersion,
    pub benchmark_id: BenchmarkId,
    pub last_sequence: u64,
    pub last_event_sha256: Sha256Digest,
    pub cells: Vec<BenchmarkCellState>,
}
