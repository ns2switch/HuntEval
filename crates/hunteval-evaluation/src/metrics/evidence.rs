use std::collections::BTreeMap;

use hunteval_domain::{Applicability, MetricDirection, MetricValue};

use crate::{EvaluationInput, sets};

pub(super) fn evaluate(input: &EvaluationInput, metrics: &mut BTreeMap<String, MetricValue>) {
    metrics.insert(
        "evidence_event_coverage".into(),
        sets::recall(
            &input.grounded_evidence_events,
            &input.truth_events,
            input.benign_scored_episode,
        ),
    );
    metrics.insert(
        "evidence_entity_coverage".into(),
        sets::recall(
            &input.grounded_evidence_entities,
            &input.truth_entities,
            input.benign_scored_episode,
        ),
    );
    let sufficiency = if input.minimum_evidence_items == 0 {
        sets::unavailable(
            Applicability::InsufficientEvidenceRequirements,
            MetricDirection::HigherIsBetter,
        )
    } else {
        sets::counted(
            input.submitted_grounded_evidence_items,
            input.minimum_evidence_items,
            MetricDirection::HigherIsBetter,
        )
    };
    metrics.insert("evidence_sufficiency".into(), sufficiency);
}
