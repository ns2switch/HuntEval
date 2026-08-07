//! Pure deterministic evaluation over stored trusted inputs.

mod evaluator;
mod profile;
mod sets;
mod types;

pub use evaluator::{DeterministicEvaluator, EvaluationError, Evaluator};
pub use profile::{ProfileError, evaluate_constraints, score_profile};
pub use types::{
    AggregateScore, ConstraintEvaluation, EvaluationInput, MetricDefinition, MetricVector,
    MissingMetricPolicy, ScoringProfile,
};
