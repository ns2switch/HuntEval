use std::{fs::OpenOptions, io::Write, path::Path, time::Duration};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCell, ProtocolVersion, RunId, SchemaVersion, UtcTimestamp,
};
use serde::Serialize;
use time::OffsetDateTime;

use crate::{DuckDbManagedTool, ResolvedRunInputs, RunExecutor, RunFailureKind, RunRequest};

use super::{BenchmarkCellExecutor, BenchmarkExecutionPlan, CellExecution, CellExecutionFailure};

#[derive(Debug, Clone)]
pub struct ProductionCellExecutor {
    plan: BenchmarkExecutionPlan,
    deployment_executable: std::path::PathBuf,
    duckdb_worker: std::path::PathBuf,
    schema_contract: std::path::PathBuf,
}

impl ProductionCellExecutor {
    #[must_use]
    pub fn new(
        plan: BenchmarkExecutionPlan,
        deployment_executable: std::path::PathBuf,
        duckdb_worker: std::path::PathBuf,
        schema_contract: std::path::PathBuf,
    ) -> Self {
        Self {
            plan,
            deployment_executable,
            duckdb_worker,
            schema_contract,
        }
    }
}

impl BenchmarkCellExecutor for ProductionCellExecutor {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        let deployment = self
            .plan
            .deployments
            .get(&cell.key.deployment.id)
            .ok_or_else(|| CellExecutionFailure::validated("missing_deployment"))?;
        let episode = self
            .plan
            .episodes
            .get(&cell.key.episode.id)
            .ok_or_else(|| CellExecutionFailure::validated("missing_episode"))?;
        let mut inputs = ResolvedRunInputs::resolve_with_executable(
            episode,
            deployment,
            &self.plan.scoring_profile,
            &self.schema_contract,
            Some(&self.deployment_executable),
        )
        .map_err(|_| CellExecutionFailure::validated("invalid_configuration"))?;
        inputs.hashes.insert(
            "managed_tool_binary".to_owned(),
            crate::hash_file(&self.duckdb_worker)
                .map_err(|_| CellExecutionFailure::validated("invalid_configuration"))?,
        );
        let timeout =
            Duration::from_secs(inputs.episode.public().manifest.limits.max_duration_seconds);
        let tool = DuckDbManagedTool::new(&self.duckdb_worker, inputs.episode.public());
        let request = RunRequest {
            run_id: run_id.clone(),
            seed: cell.key.seed,
            output_root: output_root.to_path_buf(),
            started_at: now()?,
            protocol_version: ProtocolVersion::new(0, 3),
            timeout,
            maximum_line_bytes: 128 * 1024,
        };
        let execution = RunExecutor
            .execute(&request, &inputs, &tool)
            .map_err(|failure| CellExecutionFailure::validated(failure_reason(failure.kind)))?;
        let result_path = execution.artifacts.root.join("result.json");
        let result = NormalizedCellResult {
            schema_version: SchemaVersion::new(0, 4),
            cell_id: cell.cell_id,
            run_id: run_id.clone(),
            cell: cell.clone(),
            metrics: execution.metrics,
            aggregate_score: execution.aggregate_score,
            usage: execution.usage,
            artifact_hashes: execution.artifacts.hashes,
        };
        write_result(&result_path, &result)?;
        Ok(CellExecution {
            run_id: run_id.clone(),
            result_path,
        })
    }
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct NormalizedCellResult {
    schema_version: SchemaVersion,
    cell_id: hunteval_domain::BenchmarkCellId,
    run_id: RunId,
    cell: BenchmarkCell,
    metrics: hunteval_evaluation::MetricVector,
    aggregate_score: hunteval_evaluation::AggregateScore,
    usage: crate::BudgetUsage,
    artifact_hashes: std::collections::BTreeMap<String, hunteval_domain::Sha256Digest>,
}

fn write_result(path: &Path, result: &NormalizedCellResult) -> Result<(), CellExecutionFailure> {
    let mut bytes = serde_json::to_vec_pretty(result)
        .map_err(|_| CellExecutionFailure::validated("result_serialization_failed"))?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|_| CellExecutionFailure::validated("result_write_failed"))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| CellExecutionFailure::validated("result_write_failed"))
}

fn now() -> Result<UtcTimestamp, CellExecutionFailure> {
    UtcTimestamp::new(OffsetDateTime::now_utc())
        .map_err(|_| CellExecutionFailure::validated("clock_failure"))
}

const fn failure_reason(kind: RunFailureKind) -> &'static str {
    match kind {
        RunFailureKind::Artifact => "artifact_failure",
        RunFailureKind::BudgetExceeded => "budget_exceeded",
        RunFailureKind::InvalidConfiguration => "invalid_configuration",
        RunFailureKind::Evaluation => "evaluation_failure",
        RunFailureKind::ManagedTool => "managed_tool_failure",
        RunFailureKind::ProcessCrash => "process_crash",
        RunFailureKind::ProtocolViolation => "protocol_violation",
        RunFailureKind::Timeout => "timeout",
    }
}
