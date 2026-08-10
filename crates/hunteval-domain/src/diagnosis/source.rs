use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::{RunId, Sha256Digest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSourceKind {
    TrajectoryEvent,
    Agent,
    Action,
    Task,
    Evidence,
    Finding,
    OperationalMessage,
    Metric,
    BenchmarkCell,
    StatisticalComparison,
    Topology,
    TopologyExperiment,
    TopologyEquivalence,
    TopologyAnalysis,
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFamily {
    Trajectory,
    Deployment,
    ManagedTool,
    InvestigationArtifact,
    Metric,
    Benchmark,
    TopologyExperiment,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiagnosticSourceReference {
    TrajectoryEvent {
        run_id: RunId,
        event_sequence: u64,
        artifact_sha256: Sha256Digest,
    },
    Agent {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    Action {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    Task {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    Evidence {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    Finding {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    OperationalMessage {
        run_id: RunId,
        entity_id: String,
        artifact_sha256: Sha256Digest,
    },
    Metric {
        #[serde(skip_serializing_if = "Option::is_none")]
        run_id: Option<RunId>,
        metric_name: String,
        metric_version: SchemaVersionText,
        artifact_sha256: Sha256Digest,
    },
    BenchmarkCell {
        cell_id: String,
        artifact_sha256: Sha256Digest,
    },
    StatisticalComparison {
        comparison_id: String,
        artifact_sha256: Sha256Digest,
    },
    Topology {
        artifact_id: String,
        artifact_sha256: Sha256Digest,
    },
    TopologyExperiment {
        artifact_id: String,
        artifact_sha256: Sha256Digest,
    },
    TopologyEquivalence {
        artifact_id: String,
        artifact_sha256: Sha256Digest,
    },
    TopologyAnalysis {
        artifact_id: String,
        artifact_sha256: Sha256Digest,
    },
    Artifact {
        path: String,
        artifact_sha256: Sha256Digest,
        #[serde(skip_serializing_if = "Option::is_none")]
        pointer: Option<String>,
    },
}

pub type SchemaVersionText = crate::SchemaVersion;

impl DiagnosticSourceReference {
    #[must_use]
    pub const fn kind(&self) -> DiagnosticSourceKind {
        match self {
            Self::TrajectoryEvent { .. } => DiagnosticSourceKind::TrajectoryEvent,
            Self::Agent { .. } => DiagnosticSourceKind::Agent,
            Self::Action { .. } => DiagnosticSourceKind::Action,
            Self::Task { .. } => DiagnosticSourceKind::Task,
            Self::Evidence { .. } => DiagnosticSourceKind::Evidence,
            Self::Finding { .. } => DiagnosticSourceKind::Finding,
            Self::OperationalMessage { .. } => DiagnosticSourceKind::OperationalMessage,
            Self::Metric { .. } => DiagnosticSourceKind::Metric,
            Self::BenchmarkCell { .. } => DiagnosticSourceKind::BenchmarkCell,
            Self::StatisticalComparison { .. } => DiagnosticSourceKind::StatisticalComparison,
            Self::Topology { .. } => DiagnosticSourceKind::Topology,
            Self::TopologyExperiment { .. } => DiagnosticSourceKind::TopologyExperiment,
            Self::TopologyEquivalence { .. } => DiagnosticSourceKind::TopologyEquivalence,
            Self::TopologyAnalysis { .. } => DiagnosticSourceKind::TopologyAnalysis,
            Self::Artifact { .. } => DiagnosticSourceKind::Artifact,
        }
    }

    #[must_use]
    pub const fn family(&self) -> SourceFamily {
        match self {
            Self::TrajectoryEvent { .. } => SourceFamily::Trajectory,
            Self::Agent { .. } | Self::Artifact { .. } => SourceFamily::Deployment,
            Self::Action { .. } => SourceFamily::ManagedTool,
            Self::Task { .. }
            | Self::Evidence { .. }
            | Self::Finding { .. }
            | Self::OperationalMessage { .. } => SourceFamily::InvestigationArtifact,
            Self::Metric { .. } => SourceFamily::Metric,
            Self::BenchmarkCell { .. } | Self::StatisticalComparison { .. } => {
                SourceFamily::Benchmark
            }
            Self::Topology { .. }
            | Self::TopologyExperiment { .. }
            | Self::TopologyEquivalence { .. }
            | Self::TopologyAnalysis { .. } => SourceFamily::TopologyExperiment,
        }
    }

    #[must_use]
    pub fn has_safe_shape(&self) -> bool {
        match self {
            Self::TrajectoryEvent { event_sequence, .. } => *event_sequence > 0,
            Self::Agent { entity_id, .. }
            | Self::Action { entity_id, .. }
            | Self::Task { entity_id, .. }
            | Self::Evidence { entity_id, .. }
            | Self::Finding { entity_id, .. }
            | Self::OperationalMessage { entity_id, .. } => bounded_identifier(entity_id),
            Self::Metric { metric_name, .. } => bounded_identifier(metric_name),
            Self::BenchmarkCell { cell_id, .. } => {
                cell_id.parse::<crate::BenchmarkCellId>().is_ok()
            }
            Self::StatisticalComparison { comparison_id, .. } => bounded_identifier(comparison_id),
            Self::Topology { artifact_id, .. }
            | Self::TopologyExperiment { artifact_id, .. }
            | Self::TopologyEquivalence { artifact_id, .. }
            | Self::TopologyAnalysis { artifact_id, .. } => bounded_identifier(artifact_id),
            Self::Artifact { path, pointer, .. } => {
                safe_relative(path) && pointer.as_deref().is_none_or(valid_pointer)
            }
        }
    }
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_pointer(value: &str) -> bool {
    value.starts_with('/') && value.len() <= 1024 && !value.chars().any(char::is_control)
}
