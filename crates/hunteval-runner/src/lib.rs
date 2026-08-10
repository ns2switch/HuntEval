//! Trusted HuntEval orchestration components.
//!
//! Outbound adapters are kept behind small policy, process, and artifact
//! boundaries so orchestration remains testable without external services.

mod artifacts;
mod benchmark;
mod budget;
mod dataset_review;
mod diagnostics;
mod episode_loader;
mod faults;
mod hashing;
mod improvement;
mod knowledge;
mod managed_tool;
mod orchestrator;
mod policy;
mod process;
mod reporting;
mod run;
mod scheduling;
mod sql_tool;
mod topology_ablation;
mod topology_control;
mod topology_report;
mod vertical_slice;

pub use artifacts::{ArtifactError, ArtifactWriter, RunManifest};
pub use benchmark::{
    AuthoredBenchmarkManifest, AuthoredRunCell, BenchmarkCellExecutor, BenchmarkCellState,
    BenchmarkCellStatus, BenchmarkError, BenchmarkEvent, BenchmarkEventKind,
    BenchmarkExecutionPlan, BenchmarkJournal, BenchmarkJournalError, BenchmarkManifest,
    BenchmarkMetricGroup, BenchmarkMetrics, BenchmarkRunOptions, BenchmarkRunSummary,
    BenchmarkService, BenchmarkServiceError, BenchmarkState, CellExecution, CellExecutionFailure,
    ComparisonEligibility, ComparisonReason, ComparisonStatus, ProductionCellExecutor,
    ResolvedTopology, RetryPolicy, RunCell, load_benchmark, load_deployment_topology,
    load_stored_definition, resolve_benchmark, resolve_execution_plan,
};
pub use budget::{BudgetError, BudgetLedger, BudgetLimits, BudgetUsage};
pub use dataset_review::{
    DatasetReviewError, DatasetReviewValidation, DatasetReviewValidationStatus,
    create_approved_dataset_review, hash_public_package, validate_dataset_review,
};
pub use diagnostics::{
    DiagnosticBundleArtifact, DiagnosticBundleManifest, DiagnosticGenerationError,
    DiagnosticVerificationResult, DiagnosticVerificationStatus, generate_benchmark_diagnosis,
    generate_run_diagnosis, verify_diagnostic_bundle,
};
pub use episode_loader::{ArtifactDigests, EpisodeLoadError, EpisodePackage, PublicEpisodePackage};
pub use faults::FaultController;
pub use hashing::{HashingError, hash_file};
pub use hunteval_reporting::ReportFormat;
pub use hunteval_sandbox::{ResolvedExecutionPolicy, SandboxCapabilityReport, probe_linux_sandbox};
pub use hunteval_sandbox::{SecretScanPolicy, SecretScanResult, SecretScanStatus, scan_paths};
pub use improvement::*;
pub use knowledge::{KnowledgeController, KnowledgeControllerError};
pub use managed_tool::{ManagedTool, ManagedToolError, ManagedToolOutput};
pub use orchestrator::{OrchestratorError, RunConfig, RunOrchestrator, RunTerminalStatus};
pub use policy::{IsolationPolicy, PolicyError};
pub use process::{DeploymentProcess, LinuxSandbox, ProcessError, ProcessOutput, ProcessSpec};
pub use reporting::{ReportGenerationError, ReportVerification, generate_report, verify_report};
pub use run::{
    ConformanceResult, ConformanceStatus, DiagnosticObservedRun, ResolvedRunInputs, RunArtifacts,
    RunExecution, RunExecutor, RunFailure, RunFailureKind, RunInputError, RunRequest,
    RunVerificationResult, StoredEvaluationError, StoredEvaluationHashes, VerificationCheck,
    VerificationStatus, load_observed_run_for_diagnosis, load_scoring_profile,
    load_trusted_run_view, run_conformance, verify_run,
};
pub use scheduling::{ScheduledTask, deterministic_schedule};
pub use sql_tool::DuckDbManagedTool;
pub use topology_ablation::{
    ControlledTopologyAblation, TopologyAblationObservations, execute_controlled_topology_ablation,
};
pub use topology_control::{
    TopologyControlError, build_controlled_topology_analysis, evaluate_topology_equivalence,
    registration_conforms_to_topology,
};
pub use topology_report::{
    ControlledTopologyReportError, ControlledTopologyReportInput, render_controlled_topology_report,
};
pub use vertical_slice::run_vertical_slice;

/// Replays a stored trajectory and returns its event count and exact-byte hash.
pub fn inspect_trajectory(
    bytes: &[u8],
) -> Result<(u64, hunteval_domain::Sha256Digest), hunteval_protocol::ProtocolError> {
    let outcome = hunteval_protocol::replay_trajectory(bytes, 128 * 1024)?;
    Ok((outcome.event_count, outcome.trajectory_sha256))
}
