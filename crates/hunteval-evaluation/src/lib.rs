//! Pure deterministic evaluation over stored trusted inputs.

mod diagnosis;
mod evaluator;
mod experiments;
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
pub use profile::{ProfileError, evaluate_constraints, score_profile};
pub use types::{
    AggregateScore, ConstraintEvaluation, EvaluationInput, MetricDefinition, MetricVector,
    MissingMetricPolicy, ScoringProfile,
};
