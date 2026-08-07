//! Infrastructure-independent HuntEval domain primitives.
//!
//! This crate intentionally contains no database, process, CLI, model-provider,
//! agent-framework, or storage-adapter dependencies.

mod benchmark;
mod deployment;
mod digest;
mod episode;
mod error;
mod evidence;
mod id;
mod metrics;
mod result;
mod task;
mod timestamp;
mod version;

pub use benchmark::{
    BenchmarkCell, BenchmarkCellId, BenchmarkCellIdParseError, BenchmarkCellKey,
    BenchmarkDefinition, BenchmarkDefinitionError, ResolvedArtifact, ResolvedDeployment,
    ResolvedEpisode,
};
pub use deployment::{AgentRegistration, DeploymentArchitecture, DeploymentRegistration};
pub use digest::{DigestParseError, Sha256Digest};
pub use episode::{
    EpisodeLimits, EpisodeManifest, EpisodeObjective, GroundTruth, KnowledgeConfig, Provider,
    TelemetryConfig, TelemetryTable,
};
pub use error::{ContractValidationError, DomainError};
pub use evidence::{
    Confidence, Evidence, FinalSubmission, Finding, FindingSeverity, SubmissionStatus, TimeRange,
};
pub use id::{
    ActionId, AgentId, BenchmarkAttemptId, BenchmarkId, DeploymentId, EpisodeId, EventId,
    EvidenceId, FaultProfileId, FindingId, HypothesisId, IdValidationError, MessageId, RunId,
    ScoringProfileId, TaskId,
};
pub use metrics::{Applicability, MetricDirection, MetricRange, MetricValue};
pub use result::{
    ArtifactReferences, ConstraintViolation, MetricVector, ResourceProvenance, ResourceUsage,
    RunResult, RunStatus, SourcedCost,
};
pub use task::{TaskPriority, TaskRecord, TaskSpec, TaskState};
pub use timestamp::{TimestampError, UtcTimestamp};
pub use version::{ContractVersion, ProtocolVersion, SchemaVersion, VersionParseError};
