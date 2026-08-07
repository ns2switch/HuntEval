//! Pure deterministic evaluation over stored trusted inputs.

mod diagnosis;
mod evaluator;
mod experiments;
mod input;
mod profile;
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
    ObservedRun, ObservedTask, ObservedToolOutcome, SubmittedTimelineEntry, TrustedRunInput,
    TrustedRunView, TrustedViewError,
};
pub use profile::{ProfileError, evaluate_constraints, score_profile};
pub use types::{
    AggregateScore, ConstraintEvaluation, EvaluationInput, MetricDefinition, MetricVector,
    MissingMetricPolicy, ScoringProfile,
};
