use std::{fs::OpenOptions, io::Write, path::Path, time::Duration};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCell, ProtocolVersion, RunId, SchemaVersion, UtcTimestamp,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    DuckDbManagedTool, ResolvedRunInputs, RunExecutor, RunFailureKind, RunManifest, RunRequest,
};

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
        let result = BenchmarkCellResult {
            schema_version: SchemaVersion::new(0, 4),
            cell_id: cell.cell_id,
            run_id: run_id.clone(),
            cell: cell.clone(),
            metrics: execution.metrics,
            aggregate_score: execution.aggregate_score,
            constraints: execution.constraints,
            usage: execution.usage,
            resource_usage: execution.resource_usage,
            submission: execution.submission,
            artifact_hashes: execution.artifacts.hashes,
        };
        let result_digest = write_result(&result_path, &result)?;
        bind_result_digest(&execution.artifacts.root, result_digest)?;
        Ok(CellExecution {
            run_id: run_id.clone(),
            result_path,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchmarkCellResult {
    pub(crate) schema_version: SchemaVersion,
    pub(crate) cell_id: hunteval_domain::BenchmarkCellId,
    pub(crate) run_id: RunId,
    pub(crate) cell: BenchmarkCell,
    pub(crate) metrics: hunteval_evaluation::MetricVector,
    pub(crate) aggregate_score: hunteval_evaluation::AggregateScore,
    pub(crate) constraints: Vec<hunteval_evaluation::ConstraintEvaluation>,
    pub(crate) usage: crate::BudgetUsage,
    pub(crate) resource_usage: hunteval_domain::ResourceUsage,
    pub(crate) submission: hunteval_domain::FinalSubmission,
    pub(crate) artifact_hashes: std::collections::BTreeMap<String, hunteval_domain::Sha256Digest>,
}

fn write_result(
    path: &Path,
    result: &BenchmarkCellResult,
) -> Result<hunteval_domain::Sha256Digest, CellExecutionFailure> {
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
        .map_err(|_| CellExecutionFailure::validated("result_write_failed"))?;
    Ok(hunteval_domain::Sha256Digest::from_bytes(&bytes))
}

fn bind_result_digest(
    run_root: &Path,
    digest: hunteval_domain::Sha256Digest,
) -> Result<(), CellExecutionFailure> {
    let manifest_path = run_root.join("manifest.json");
    let bytes = std::fs::read(&manifest_path)
        .map_err(|_| CellExecutionFailure::validated("manifest_read_failed"))?;
    let mut manifest: RunManifest = serde_json::from_slice(&bytes)
        .map_err(|_| CellExecutionFailure::validated("manifest_read_failed"))?;
    manifest.hashes.insert("result".to_owned(), digest);
    let mut normalized = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| CellExecutionFailure::validated("result_serialization_failed"))?;
    normalized.push(b'\n');
    let temporary = run_root.join("manifest.json.tmp");
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|_| CellExecutionFailure::validated("manifest_write_failed"))?;
    file.write_all(&normalized)
        .and_then(|_| file.sync_all())
        .map_err(|_| CellExecutionFailure::validated("manifest_write_failed"))?;
    std::fs::rename(temporary, manifest_path)
        .map_err(|_| CellExecutionFailure::validated("manifest_write_failed"))
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
        RunFailureKind::ResourceLimit => "resource_limit",
        RunFailureKind::Timeout => "timeout",
    }
}
