use std::collections::BTreeMap;

use hunteval_domain::{Applicability, MetricDirection, MetricRange, MetricValue};
use thiserror::Error;

use crate::{
    EvaluationInput, MetricVector,
    metrics::evaluate_investigation,
    sets::{counted, precision, recall},
};

pub trait Evaluator {
    fn evaluate(&self, input: &EvaluationInput) -> Result<MetricVector, EvaluationError>;
}

#[derive(Debug, Default)]
pub struct DeterministicEvaluator;

impl Evaluator for DeterministicEvaluator {
    fn evaluate(&self, input: &EvaluationInput) -> Result<MetricVector, EvaluationError> {
        validate_counts(input)?;
        let mut metrics = BTreeMap::new();
        metrics.insert(
            "event_precision".into(),
            precision(
                &input.submitted_events,
                &input.truth_events,
                input.benign_scored_episode,
            ),
        );
        evaluate_investigation(input, &mut metrics)?;
        metrics.insert(
            "event_recall".into(),
            recall(
                &input.submitted_events,
                &input.truth_events,
                input.benign_scored_episode,
            ),
        );
        metrics.insert(
            "entity_precision".into(),
            precision(
                &input.submitted_entities,
                &input.truth_entities,
                input.benign_scored_episode,
            ),
        );
        metrics.insert(
            "entity_recall".into(),
            recall(
                &input.submitted_entities,
                &input.truth_entities,
                input.benign_scored_episode,
            ),
        );
        let grounded_denominator = if input.evidence_items == 0 && input.findings_submitted > 0 {
            1
        } else {
            input.evidence_items
        };
        metrics.insert(
            "evidence_grounding".into(),
            counted(
                input.grounded_evidence_items,
                grounded_denominator,
                MetricDirection::HigherIsBetter,
            ),
        );
        metrics.insert(
            "provenance_validity".into(),
            counted(
                input.valid_provenance_references,
                input.provenance_references,
                MetricDirection::HigherIsBetter,
            ),
        );
        metrics.insert(
            "task_completion".into(),
            counted(
                input.tasks_completed,
                input.tasks_created,
                MetricDirection::HigherIsBetter,
            ),
        );
        metrics.insert(
            "tool_call_utilization".into(),
            counted(
                input.tool_calls_used,
                input.tool_call_limit,
                MetricDirection::LowerIsBetter,
            ),
        );
        metrics.insert(
            "resilience".into(),
            MetricValue {
                value: None,
                applicability: Applicability::RequiresFaultPair,
                direction: MetricDirection::HigherIsBetter,
                range: MetricRange {
                    minimum: 0.0,
                    maximum: 1.0,
                },
                numerator: None,
                denominator: None,
            },
        );
        Ok(MetricVector(metrics))
    }
}

fn validate_counts(input: &EvaluationInput) -> Result<(), EvaluationError> {
    for (name, numerator, denominator) in [
        (
            "grounded evidence",
            input.grounded_evidence_items,
            input.evidence_items,
        ),
        (
            "valid provenance",
            input.valid_provenance_references,
            input.provenance_references,
        ),
        (
            "completed tasks",
            input.tasks_completed,
            input.tasks_created,
        ),
    ] {
        if numerator > denominator {
            return Err(EvaluationError::InvalidCount(name));
        }
    }
    if input.tool_calls_used > input.tool_call_limit {
        return Err(EvaluationError::ToolBudgetViolation);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvaluationError {
    #[error("{0} count exceeds its denominator")]
    InvalidCount(&'static str),
    #[error("tool-call usage exceeds the trusted limit")]
    ToolBudgetViolation,
    #[error("attack paths exceed the bounded exact comparison size")]
    AttackPathComparisonTooLarge,
    #[error("timeline contains duplicate event identifiers")]
    DuplicateTimelineEvent,
    #[error("acceptable submission statuses cannot be empty")]
    EmptyAcceptableStatuses,
    #[error("unsupported ATT&CK technique identifier {0}")]
    UnsupportedTechniqueIdentifier(String),
}
