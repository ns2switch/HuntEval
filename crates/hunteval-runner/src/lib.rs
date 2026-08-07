//! Trusted HuntEval orchestration components.
//!
//! Outbound adapters are kept behind small policy, process, and artifact
//! boundaries so orchestration remains testable without external services.

mod artifacts;
mod budget;
mod episode_loader;
mod hashing;
mod managed_tool;
mod orchestrator;
mod policy;
mod process;
mod scheduling;
mod vertical_slice;

pub use artifacts::{ArtifactError, ArtifactWriter, RunManifest};
pub use budget::{BudgetError, BudgetLedger, BudgetLimits, BudgetUsage};
pub use episode_loader::{ArtifactDigests, EpisodeLoadError, EpisodePackage, PublicEpisodePackage};
pub use hashing::{HashingError, hash_file};
pub use managed_tool::{ManagedTool, ManagedToolError};
pub use orchestrator::{OrchestratorError, RunConfig, RunOrchestrator, RunTerminalStatus};
pub use policy::{IsolationPolicy, PolicyError};
pub use process::{DeploymentProcess, LinuxSandbox, ProcessError, ProcessOutput, ProcessSpec};
pub use scheduling::{ScheduledTask, deterministic_schedule};
pub use vertical_slice::run_vertical_slice;

/// Replays a stored trajectory and returns its event count and exact-byte hash.
pub fn inspect_trajectory(
    bytes: &[u8],
) -> Result<(u64, hunteval_domain::Sha256Digest), hunteval_protocol::ProtocolError> {
    let outcome = hunteval_protocol::replay_trajectory(bytes, 128 * 1024)?;
    Ok((outcome.event_count, outcome.trajectory_sha256))
}
