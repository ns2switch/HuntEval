//! Trusted HuntEval orchestration components.
//!
//! Outbound adapters are kept behind small policy, process, and artifact
//! boundaries so orchestration remains testable without external services.

mod artifacts;
mod benchmark;
mod budget;
mod episode_loader;
mod faults;
mod hashing;
mod knowledge;
mod managed_tool;
mod orchestrator;
mod policy;
mod process;
mod reporting;
mod run;
mod scheduling;
mod sql_tool;
mod vertical_slice;

pub use artifacts::{ArtifactError, ArtifactWriter, RunManifest};
pub use benchmark::{
    AuthoredBenchmarkManifest, AuthoredRunCell, BenchmarkCellExecutor, BenchmarkCellState,
    BenchmarkCellStatus, BenchmarkError, BenchmarkEvent, BenchmarkEventKind,
    BenchmarkExecutionPlan, BenchmarkJournal, BenchmarkJournalError, BenchmarkManifest,
    BenchmarkMetricGroup, BenchmarkMetrics, BenchmarkRunOptions, BenchmarkRunSummary,
    BenchmarkService, BenchmarkServiceError, BenchmarkState, CellExecution, CellExecutionFailure,
    ComparisonEligibility, ComparisonReason, ComparisonStatus, ProductionCellExecutor, RetryPolicy,
    RunCell, load_benchmark, load_stored_definition, resolve_benchmark, resolve_execution_plan,
};
pub use budget::{BudgetError, BudgetLedger, BudgetLimits, BudgetUsage};
pub use episode_loader::{ArtifactDigests, EpisodeLoadError, EpisodePackage, PublicEpisodePackage};
pub use faults::FaultController;
pub use hashing::{HashingError, hash_file};
pub use hunteval_reporting::ReportFormat;
pub use knowledge::{KnowledgeController, KnowledgeControllerError};
pub use managed_tool::{ManagedTool, ManagedToolError, ManagedToolOutput};
pub use orchestrator::{OrchestratorError, RunConfig, RunOrchestrator, RunTerminalStatus};
pub use policy::{IsolationPolicy, PolicyError};
pub use process::{DeploymentProcess, LinuxSandbox, ProcessError, ProcessOutput, ProcessSpec};
pub use reporting::{ReportGenerationError, ReportVerification, generate_report, verify_report};
pub use run::{
    ResolvedRunInputs, RunArtifacts, RunExecution, RunExecutor, RunFailure, RunFailureKind,
    RunInputError, RunRequest, StoredEvaluationError, StoredEvaluationHashes, load_scoring_profile,
    load_trusted_run_view,
};
pub use scheduling::{ScheduledTask, deterministic_schedule};
pub use sql_tool::DuckDbManagedTool;
pub use vertical_slice::run_vertical_slice;

/// Replays a stored trajectory and returns its event count and exact-byte hash.
pub fn inspect_trajectory(
    bytes: &[u8],
) -> Result<(u64, hunteval_domain::Sha256Digest), hunteval_protocol::ProtocolError> {
    let outcome = hunteval_protocol::replay_trajectory(bytes, 128 * 1024)?;
    Ok((outcome.event_count, outcome.trajectory_sha256))
}
