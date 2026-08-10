//! Pure deterministic evaluation over stored trusted inputs.

mod diagnosis;
mod diagnostics;
mod evaluator;
mod improvement;
mod input;
mod metrics;
mod profile;
mod registry;
mod sets;
mod topology_metrics;
mod types;

pub use diagnosis::{
    DiagnosticEvidence, DiagnosticInput, FailureClassification, FailureKind, ObservableFailure,
    Recommendation, RecommendationStatus, diagnose, recommend,
};
pub use diagnostics::{
    BottleneckError, ClassificationCandidate, ClassifierRule, ComparableDiagnosticCell,
    ContributionError, ControlledContributionInput, DiagnosticArtifactSet, DiagnosticInputV07,
    DiagnosticRegistryError, DiagnosticResolutionError, EvidenceSufficiency, RecurrenceError,
    ResolvedDiagnosticSource, canonical_taxonomy, classifier_registry_digest, classify_verified,
    evaluate_bottlenecks, evaluate_sufficiency, reduce_controlled_contribution, reduce_recurrence,
    resolve_sources, validate_registry,
};
pub use evaluator::{DeterministicEvaluator, EvaluationError, Evaluator};
pub use improvement::*;
pub use input::{
    EvaluationProvenance, ObservedAction, ObservedEvidence, ObservedFinding, ObservedMessage,
    ObservedRun, ObservedTask, ObservedTaskTransition, ObservedToolOutcome, SubmittedTimelineEntry,
    TrustedRunInput, TrustedRunView, TrustedViewError,
};
pub use profile::{
    ConstraintInput, ProfileError, evaluate_constraints, normalize_profile, score_profile,
};
pub use registry::{MetricContract, metric_contract, metric_contracts};
pub use topology_metrics::{TopologyMetricError, TopologyMetricInput, evaluate_topology_metrics};
pub use types::{
    AggregateScore, ConstraintEvaluation, ConstraintStatus, EfficiencyInput, EvaluationInput,
    LegacyScoringProfile, MetricDefinition, MetricReference, MetricSelection, MetricVector,
    MissingMetricPolicy, ResourceProvenanceRequirement, ScoringConstraint, ScoringProfile,
    ScoringProfileArtifact, ThresholdComparison,
};
