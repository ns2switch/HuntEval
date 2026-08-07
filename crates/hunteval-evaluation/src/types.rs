use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    EventId, ExpectedTimelineWindow, MetricDirection, MetricValue, SchemaVersion, SubmissionStatus,
    TimelineEntry,
};
use serde::{Deserialize, Serialize};

/// Trusted normalized values required by the deterministic evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}

/// Complete human-readable metric contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricDefinition {
    pub name: String,
    pub minimum: f64,
    pub maximum: f64,
    pub direction: MetricDirection,
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

/// Versioned weights supplied as data, never compiled into evaluation code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoringProfile {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub missing_metric_policy: MissingMetricPolicy,
    pub weights: BTreeMap<String, f64>,
    #[serde(default)]
    pub disqualifying_constraints: BTreeSet<String>,
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
    pub violated: bool,
    pub disqualifying: bool,
}
