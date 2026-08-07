use std::{collections::BTreeMap, path::PathBuf};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCell, BenchmarkCellId, BenchmarkDefinition, DeploymentId,
    EpisodeId, RunId,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::benchmark::BenchmarkJournalError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkExecutionPlan {
    pub definition: BenchmarkDefinition,
    pub deployments: BTreeMap<DeploymentId, PathBuf>,
    pub episodes: BTreeMap<EpisodeId, PathBuf>,
    pub scoring_profile: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryPolicy {
    None,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkRunOptions {
    pub jobs: usize,
    pub fail_fast: bool,
    pub retry: RetryPolicy,
}

impl Default for BenchmarkRunOptions {
    fn default() -> Self {
        Self {
            jobs: 1,
            fail_fast: false,
            retry: RetryPolicy::None,
        }
    }
}

impl BenchmarkExecutionPlan {
    /// Binds exact runtime binaries and schema bytes into each configuration identity.
    pub fn bind_runtime_artifacts(
        &mut self,
        deployment_executable: &std::path::Path,
        managed_tool_executable: &std::path::Path,
        schema_contract: &std::path::Path,
        runner_executable: &std::path::Path,
    ) -> Result<(), BenchmarkServiceError> {
        let deployment_hash = crate::hash_file(deployment_executable)?;
        let managed_tool_hash = crate::hash_file(managed_tool_executable)?;
        let schema_hash = crate::hash_file(schema_contract)?;
        let runner_hash = crate::hash_file(runner_executable)?;
        let deployments = self
            .definition
            .deployments
            .iter()
            .map(|deployment| hunteval_domain::ResolvedDeployment {
                id: deployment.id.clone(),
                configuration_sha256: hunteval_domain::Sha256Digest::from_bytes(
                    format!(
                        "{}:{deployment_hash}:{managed_tool_hash}:{schema_hash}:{runner_hash}",
                        deployment.configuration_sha256
                    )
                    .as_bytes(),
                ),
            })
            .collect();
        self.definition = BenchmarkDefinition::new(
            self.definition.id.clone(),
            deployments,
            self.definition.episodes.clone(),
            self.definition.seeds.clone(),
            self.definition.scoring_profile.clone(),
            self.definition.fault_profile.clone(),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunSummary {
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub non_comparable: usize,
}

#[derive(Debug, Clone)]
pub struct CellExecution {
    pub run_id: RunId,
    pub result_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("cell execution failed ({reason_code})")]
pub struct CellExecutionFailure {
    pub reason_code: String,
}

pub trait BenchmarkCellExecutor: Send + Sync {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure>;
}

#[derive(Debug, Error)]
pub enum BenchmarkServiceError {
    #[error("benchmark execution options are invalid")]
    InvalidOptions,
    #[error("benchmark execution plan does not match stored state")]
    ConfigurationDrift,
    #[error("benchmark execution plan is incomplete")]
    IncompletePlan,
    #[error("benchmark identifier is invalid")]
    InvalidIdentifier,
    #[error("benchmark state is unavailable")]
    MissingState,
    #[error("benchmark cell result is invalid")]
    InvalidCellResult,
    #[error("benchmark storage I/O failed")]
    Io(#[source] std::io::Error),
    #[error("benchmark serialization failed")]
    Serialize(#[source] serde_json::Error),
    #[error("benchmark journal failed: {0}")]
    Journal(#[from] BenchmarkJournalError),
    #[error("benchmark definition failed: {0}")]
    Definition(#[from] hunteval_domain::BenchmarkDefinitionError),
    #[error("benchmark executable hashing failed: {0}")]
    Hashing(#[from] crate::HashingError),
}

impl CellExecutionFailure {
    pub(crate) fn validated(reason_code: impl Into<String>) -> Self {
        let reason_code = reason_code.into();
        let valid = !reason_code.is_empty()
            && reason_code.len() <= 256
            && reason_code
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
        Self {
            reason_code: if valid {
                reason_code
            } else {
                "execution_failure".to_owned()
            },
        }
    }
}

pub(crate) fn cell_state_map(
    state: &crate::benchmark::BenchmarkState,
) -> BTreeMap<BenchmarkCellId, crate::benchmark::BenchmarkCellState> {
    state
        .cells
        .iter()
        .cloned()
        .map(|cell| (cell.cell_id, cell))
        .collect()
}
