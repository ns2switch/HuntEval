//! Infrastructure-independent HuntEval domain primitives.
//!
//! This crate intentionally contains no database, process, CLI, model-provider,
//! agent-framework, or storage-adapter dependencies.

mod digest;
mod error;
mod id;
mod timestamp;
mod version;

pub use digest::{DigestParseError, Sha256Digest};
pub use error::DomainError;
pub use id::{
    ActionId, AgentId, DeploymentId, EpisodeId, EventId, EvidenceId, FindingId, HypothesisId,
    IdValidationError, MessageId, RunId, TaskId,
};
pub use timestamp::{TimestampError, UtcTimestamp};
pub use version::{ContractVersion, ProtocolVersion, SchemaVersion, VersionParseError};
