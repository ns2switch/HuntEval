use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use hunteval_domain::{FinalSubmission, ProtocolVersion, RunId, Sha256Digest, UtcTimestamp};
use serde::Serialize;
use thiserror::Error;

use crate::BudgetUsage;
use hunteval_evaluation::{AggregateScore, MetricVector};

/// Trusted settings for one deployment-neutral run.
#[derive(Debug, Clone)]
pub struct RunRequest {
    pub run_id: RunId,
    pub seed: u64,
    pub output_root: PathBuf,
    pub started_at: UtcTimestamp,
    pub protocol_version: ProtocolVersion,
    pub timeout: Duration,
    pub maximum_line_bytes: usize,
}

/// Verified artifact locations and exact-byte digests for a completed run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunArtifacts {
    pub root: PathBuf,
    pub hashes: BTreeMap<String, Sha256Digest>,
}

/// Successful result of the generic mediated application service.
#[derive(Debug, Clone)]
pub struct RunExecution {
    pub submission: FinalSubmission,
    pub metrics: MetricVector,
    pub aggregate_score: AggregateScore,
    pub usage: BudgetUsage,
    pub artifacts: RunArtifacts,
}

/// Stable failure categories suitable for journals and normalized results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunFailureKind {
    Artifact,
    BudgetExceeded,
    InvalidConfiguration,
    Evaluation,
    ManagedTool,
    ProcessCrash,
    ProtocolViolation,
    Timeout,
}

/// Terminal run failure with the retained partial artifact directory.
#[derive(Debug, Error)]
#[error("run failed ({kind:?}); partial artifacts: {partial_artifacts}")]
pub struct RunFailure {
    pub kind: RunFailureKind,
    pub partial_artifacts: PathBuf,
}
