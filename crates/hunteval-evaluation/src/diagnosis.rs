use std::collections::BTreeSet;

use hunteval_domain::{RunId, SchemaVersion};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    EvidenceWithoutProvenance,
    ToolBudgetExhausted,
    TaskIncomplete,
    LowEventRecall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticEvidence {
    pub run_id: RunId,
    pub event_sequences: BTreeSet<u64>,
    pub metric_references: BTreeSet<String>,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservableFailure {
    pub reason_code: String,
    pub event_sequences: BTreeSet<u64>,
    pub metric_references: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticInput {
    pub run_id: RunId,
    pub failures: Vec<ObservableFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureClassification {
    pub taxonomy_version: SchemaVersion,
    pub kind: FailureKind,
    pub evidence: DiagnosticEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatus {
    Unvalidated,
    Validated,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recommendation {
    pub id: String,
    pub classification: FailureKind,
    pub affected_runs: BTreeSet<RunId>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub proposed_change: String,
    pub status: RecommendationStatus,
    pub human_review_required: bool,
}

/// Classify only stable reason codes that already cite observable run artifacts.
#[must_use]
pub fn diagnose(input: &DiagnosticInput) -> Vec<FailureClassification> {
    input
        .failures
        .iter()
        .filter(|failure| {
            !failure.event_sequences.is_empty() || !failure.metric_references.is_empty()
        })
        .filter_map(|failure| {
            let kind = match failure.reason_code.as_str() {
                "evidence_without_provenance" => FailureKind::EvidenceWithoutProvenance,
                "tool_budget_exhausted" => FailureKind::ToolBudgetExhausted,
                "task_incomplete" => FailureKind::TaskIncomplete,
                "low_event_recall" => FailureKind::LowEventRecall,
                _ => return None,
            };
            Some(FailureClassification {
                taxonomy_version: SchemaVersion::new(0, 1),
                kind,
                evidence: DiagnosticEvidence {
                    run_id: input.run_id.clone(),
                    event_sequences: failure.event_sequences.clone(),
                    metric_references: failure.metric_references.clone(),
                    reason_code: failure.reason_code.clone(),
                },
            })
        })
        .collect()
}

/// Create deterministic hypotheses. Validation and human review remain separate gates.
#[must_use]
pub fn recommend(classifications: &[FailureClassification]) -> Vec<Recommendation> {
    classifications
        .iter()
        .enumerate()
        .map(|(index, classification)| Recommendation {
            id: format!("recommendation-{:04}", index + 1),
            classification: classification.kind,
            affected_runs: [classification.evidence.run_id.clone()]
                .into_iter()
                .collect(),
            evidence: vec![classification.evidence.clone()],
            proposed_change: recommendation_text(classification.kind).into(),
            status: RecommendationStatus::Unvalidated,
            human_review_required: true,
        })
        .collect()
}

fn recommendation_text(kind: FailureKind) -> &'static str {
    match kind {
        FailureKind::EvidenceWithoutProvenance => {
            "Require each submitted evidence item to cite its managed action and event identifiers."
        }
        FailureKind::ToolBudgetExhausted => {
            "Make the investigation plan prioritize high-value managed tool calls."
        }
        FailureKind::TaskIncomplete => {
            "Require explicit completion or reassignment for every created task."
        }
        FailureKind::LowEventRecall => {
            "Require a final coverage check against the observed investigation window."
        }
    }
}
