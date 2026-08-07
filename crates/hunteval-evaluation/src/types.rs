use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    EventId, ExpectedTimelineWindow, MetricDirection, MetricValue, ResourceProvenance,
    SchemaVersion, SubmissionStatus, TimelineEntry,
};
use serde::{Deserialize, Serialize};

/// Trusted normalized values required by the deterministic evaluator.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluationInput {
    pub truth_events: BTreeSet<EventId>,
    pub submitted_events: BTreeSet<EventId>,
    pub truth_entities: BTreeSet<String>,
    pub submitted_entities: BTreeSet<String>,
    pub expected_attack_path: Vec<EventId>,
    pub submitted_attack_path: Vec<EventId>,
    pub expected_timeline_windows: Option<Vec<ExpectedTimelineWindow>>,
    pub submitted_timeline: Option<Vec<TimelineEntry>>,
    pub acceptable_submission_statuses: Option<BTreeSet<SubmissionStatus>>,
    pub submitted_status: SubmissionStatus,
    pub expected_attack_techniques: BTreeSet<String>,
    pub submitted_attack_techniques: BTreeSet<String>,
    pub grounded_evidence_events: BTreeSet<EventId>,
    pub grounded_evidence_entities: BTreeSet<String>,
    pub submitted_grounded_evidence_items: u64,
    pub minimum_evidence_items: u64,
    pub duplicate_tool_calls: u64,
    pub useful_messages: u64,
    pub operational_messages: u64,
    pub benign_scored_episode: bool,
    pub evidence_items: u64,
    pub grounded_evidence_items: u64,
    pub findings_submitted: u64,
    pub provenance_references: u64,
    pub valid_provenance_references: u64,
    pub tasks_created: u64,
    pub tasks_completed: u64,
    pub tool_calls_used: u64,
    pub tool_call_limit: u64,
    pub resources: EfficiencyInput,
}

/// Trusted resource measurements and configured normalization caps for one run.
#[derive(Debug, Clone, PartialEq)]
pub struct EfficiencyInput {
    pub duration_ms: u64,
    pub duration_cap_ms: u64,
    pub estimated_cost: Option<f64>,
    pub cost_provenance: ResourceProvenance,
    pub estimated_cost_cap: Option<f64>,
}

impl Default for EfficiencyInput {
    fn default() -> Self {
        Self {
            duration_ms: 0,
            duration_cap_ms: 0,
            estimated_cost: None,
            cost_provenance: ResourceProvenance::Unavailable,
            estimated_cost_cap: None,
        }
    }
}

/// Complete human-readable metric contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricDefinition {
    pub name: String,
    pub version: SchemaVersion,
    pub minimum: f64,
    pub maximum: f64,
    pub direction: MetricDirection,
    pub required_resource_provenance: Option<ResourceProvenance>,
    pub denominator: String,
    pub edge_cases: String,
}

/// Named raw metrics; deterministic ordering is part of normalized JSON.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MetricVector(pub BTreeMap<String, MetricValue>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingMetricPolicy {
    Reject,
    Renormalize,
    Zero,
}

/// Versioned metric selection supplied by a scoring profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSelection {
    pub version: SchemaVersion,
    pub weight: f64,
}

/// Exact metric contract selected by a constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricReference {
    pub name: String,
    pub version: SchemaVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdComparison {
    Minimum,
    Maximum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceProvenanceRequirement {
    None,
    Measured,
    VerifiedAdapter,
}

/// Typed profile constraint. Resource provenance is explicit on metric thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ScoringConstraint {
    ObservedViolation {
        code: String,
        disqualifying: bool,
    },
    MetricThreshold {
        code: String,
        metric: MetricReference,
        comparison: ThresholdComparison,
        threshold: f64,
        disqualifying: bool,
        required_resource_provenance: ResourceProvenanceRequirement,
    },
}

/// Normalized v0.4 profile. Direction always comes from the metric registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoringProfile {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub missing_metric_policy: MissingMetricPolicy,
    pub metrics: BTreeMap<String, MetricSelection>,
    #[serde(default)]
    pub constraints: Vec<ScoringConstraint>,
}

/// Immutable v0.3 source shape accepted only through explicit normalization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyScoringProfile {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub missing_metric_policy: MissingMetricPolicy,
    pub weights: BTreeMap<String, f64>,
    #[serde(default)]
    pub disqualifying_constraints: BTreeSet<String>,
}

/// On-disk scoring profile versions accepted by the compatibility boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScoringProfileArtifact {
    Current(ScoringProfile),
    Legacy(LegacyScoringProfile),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AggregateScore {
    pub profile_id: String,
    pub value: Option<f64>,
    pub omitted_metrics: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstraintEvaluation {
    pub code: String,
    pub status: ConstraintStatus,
    pub disqualifying: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintStatus {
    Satisfied,
    Violated,
    Unverifiable,
}
