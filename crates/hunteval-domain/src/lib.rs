//! Infrastructure-independent HuntEval domain primitives.
//!
//! This crate intentionally contains no database, process, CLI, model-provider,
//! agent-framework, or storage-adapter dependencies.

mod benchmark;
mod deployment;
mod diagnosis;
mod digest;
mod episode;
mod error;
mod evidence;
mod extension;
mod extension_tool;
mod id;
mod improvement;
mod metrics;
mod result;
mod science;
mod task;
mod timestamp;
mod topology;
mod topology_analysis;
mod topology_experiment;
mod version;

pub use benchmark::{
    BenchmarkCell, BenchmarkCellId, BenchmarkCellIdParseError, BenchmarkCellKey,
    BenchmarkDefinition, BenchmarkDefinitionError, ResolvedArtifact, ResolvedDeployment,
    ResolvedEpisode,
};
pub use deployment::{AgentRegistration, DeploymentArchitecture, DeploymentRegistration};
pub use diagnosis::{
    BottleneckAnalysis, BottleneckInterval, BottleneckIntervalKind, BottleneckMetric,
    BottleneckObservations, ClassificationOmission, ContributionClaimStrength,
    ContributionInterval, ContributionMetricEffect, ContributionTarget, ContributionTargetKind,
    ControlledContributionAnalysis, DiagnosticApplicability, DiagnosticClaimStrength,
    DiagnosticHypothesis, DiagnosticMetricDirection, DiagnosticMetricRange, DiagnosticMetricUnit,
    DiagnosticRecurrenceGroup, DiagnosticSourceKind, DiagnosticSourceReference, DiagnosticTaxonomy,
    EvidenceConfidence, ExcludedDiagnosticCell, FailureCategory, FailureClassification,
    FailureDefinition, HypothesisStatus, RecurrenceClaimStrength, RunDiagnosis, SourceFamily,
    TaxonomyValidationError,
};
pub use digest::{DigestParseError, Sha256Digest};
pub use episode::{
    EpisodeLimits, EpisodeManifest, EpisodeObjective, ExpectedTimelineWindow, GroundTruth,
    KnowledgeConfig, Provider, TelemetryConfig, TelemetryTable,
};
pub use error::{ContractValidationError, DomainError};
pub use evidence::{
    Confidence, Evidence, FinalSubmission, FinalSubmissionArtifact, Finding, FindingSeverity,
    SubmissionStatus, TimeRange, TimelineEntry,
};
pub use extension::{
    ExtensionCapability, ExtensionCapabilityPolicy, ExtensionConformanceResult,
    ExtensionConformanceStatus, ExtensionKind, ExtensionLimits, ExtensionManifest,
    ExtensionNetworkPolicy, ExtensionResolution, ExtensionResolutionStatus,
};
pub use extension_tool::{ManagedToolAdapterRequest, ManagedToolAdapterResponse};
pub use id::{
    ActionId, AgentId, BenchmarkAttemptId, BenchmarkId, DatasetReviewId, DeploymentId, EpisodeId,
    EventId, EvidenceId, FaultProfileId, FindingId, HypothesisId, IdValidationError, MessageId,
    ReviewerId, RunId, ScoringProfileId, StatisticalPolicyId, TaskId, TopologyExperimentId,
    TopologyId,
};
pub use improvement::*;
pub use metrics::{Applicability, MetricDirection, MetricRange, MetricValue};
pub use result::{
    ArtifactReferences, ConstraintViolation, MetricVector, ResourceProvenance, ResourceUsage,
    RunResult, RunStatus, SourcedCost,
};
pub use science::{
    DatasetReviewRecord, DatasetReviewStatus, EpisodeCapability, EpisodeClassification,
    EpisodeDifficulty, InvestigationShape,
};
pub use task::{TaskPriority, TaskRecord, TaskSpec, TaskState};
pub use timestamp::{TimestampError, UtcTimestamp};
pub use topology::{
    CoordinationMode, DeploymentTopology, ExecutionPattern, MemoryMode, ModelComposition,
    RelationshipKind, TaskAllocationPolicy, TopologyAgent, TopologyKind, TopologyRelationship,
    TopologySpecialization,
};
pub use topology_analysis::{
    TopologyAnalysis, TopologyAnalysisKind, TopologyMetricApplicability, TopologyMetricValue,
};
pub use topology_experiment::{
    ControlHashes, EquivalenceStatus, TopologyEquivalenceResult, TopologyExperiment,
};
pub use version::{ContractVersion, ProtocolVersion, SchemaVersion, VersionParseError};
