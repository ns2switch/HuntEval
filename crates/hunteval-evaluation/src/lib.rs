//! Pure deterministic evaluation over stored trusted inputs.

mod diagnosis;
mod evaluator;
mod experiments;
mod input;
mod metrics;
mod profile;
mod registry;
mod sets;
mod types;

pub use diagnosis::{
    DiagnosticEvidence, DiagnosticInput, FailureClassification, FailureKind, ObservableFailure,
    Recommendation, RecommendationStatus, diagnose, recommend,
};
pub use evaluator::{DeterministicEvaluator, EvaluationError, Evaluator};
pub use experiments::{
    CandidateConstraint, ExperimentError, ExperimentManifest, ExperimentObservation, Partition,
    ValidationDecision, validate_candidate, validate_experiment_manifest,
};
pub use input::{
    EvaluationProvenance, ObservedAction, ObservedEvidence, ObservedFinding, ObservedMessage,
    ObservedRun, ObservedTask, ObservedTaskTransition, ObservedToolOutcome, SubmittedTimelineEntry,
    TrustedRunInput, TrustedRunView, TrustedViewError,
};
pub use profile::{
    ConstraintInput, ProfileError, evaluate_constraints, normalize_profile, score_profile,
};
pub use registry::{MetricContract, metric_contract, metric_contracts};
pub use types::{
    AggregateScore, ConstraintEvaluation, ConstraintStatus, EfficiencyInput, EvaluationInput,
    LegacyScoringProfile, MetricDefinition, MetricReference, MetricSelection, MetricVector,
    MissingMetricPolicy, ResourceProvenanceRequirement, ScoringConstraint, ScoringProfile,
    ScoringProfileArtifact, ThresholdComparison,
};
