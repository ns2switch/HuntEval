use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::DiagnosticSourceReference;
use crate::{RunId, SchemaVersion, Sha256Digest, UtcTimestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BottleneckIntervalKind {
    TaskQueue,
    TaskExecution,
    ManagedToolWait,
    AgentActive,
    AgentIdle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticApplicability {
    Available,
    Unavailable,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BottleneckInterval {
    pub kind: BottleneckIntervalKind,
    pub subject_id: String,
    pub start_event_sequence: Option<u64>,
    pub end_event_sequence: Option<u64>,
    pub start_time: Option<UtcTimestamp>,
    pub end_time: Option<UtcTimestamp>,
    pub duration_ms: Option<u64>,
    pub applicability: DiagnosticApplicability,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BottleneckObservations {
    pub schema_version: SchemaVersion,
    pub run_id: RunId,
    pub trajectory_sha256: Sha256Digest,
    pub intervals: Vec<BottleneckInterval>,
    pub reassignment_count: u64,
    pub duplicate_work_count: u64,
    pub tool_error_count: u64,
    pub tool_timeout_count: u64,
    pub limitations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticMetricUnit {
    Count,
    Ratio,
    Milliseconds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticMetricDirection {
    HigherIsBetter,
    LowerIsBetter,
    Descriptive,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticMetricRange {
    pub minimum: f64,
    pub maximum: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BottleneckMetric {
    pub name: String,
    pub version: SchemaVersion,
    pub direction: DiagnosticMetricDirection,
    pub unit: DiagnosticMetricUnit,
    pub range: DiagnosticMetricRange,
    pub applicability: DiagnosticApplicability,
    pub value: Option<f64>,
    pub numerator: Option<f64>,
    pub denominator: Option<f64>,
    pub reason_code: Option<String>,
    pub sources: BTreeSet<DiagnosticSourceReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BottleneckAnalysis {
    pub schema_version: SchemaVersion,
    pub run_id: RunId,
    pub observations_sha256: Sha256Digest,
    pub metrics: Vec<BottleneckMetric>,
    pub limitations: BTreeSet<String>,
}
