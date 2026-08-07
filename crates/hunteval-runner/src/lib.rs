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

pub use artifacts::{ArtifactError, ArtifactWriter, RunManifest};
pub use budget::{BudgetError, BudgetLedger, BudgetLimits, BudgetUsage};
pub use episode_loader::{ArtifactDigests, EpisodeLoadError, EpisodePackage, PublicEpisodePackage};
pub use hashing::{HashingError, hash_file};
pub use managed_tool::{ManagedTool, ManagedToolError};
pub use orchestrator::{OrchestratorError, RunConfig, RunOrchestrator, RunTerminalStatus};
pub use policy::{IsolationPolicy, PolicyError};
pub use process::{DeploymentProcess, LinuxSandbox, ProcessError, ProcessOutput, ProcessSpec};
