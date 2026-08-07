use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{ActionId, Applicability, EvidenceId, MetricDirection, MetricValue};

use crate::{EvaluationError, EvaluationInput, ObservedRun, sets};

use super::fingerprint::canonical_tool_fingerprint;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct CoordinationCounts {
    pub duplicate_tool_calls: u64,
    pub useful_messages: u64,
    pub operational_messages: u64,
}

pub(crate) fn reduce(observed: &ObservedRun) -> Result<CoordinationCounts, EvaluationError> {
    let mut actions: Vec<_> = observed.actions.values().collect();
    actions.sort_unstable_by(|left, right| {
        left.request_sequence
            .cmp(&right.request_sequence)
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    let evidence_by_action = evidence_by_action(observed);
    let mut fingerprints = BTreeMap::new();
    let mut duplicate_tool_calls = 0;
    for action in actions {
        let fingerprint = canonical_tool_fingerprint(&action.tool, &action.arguments)?;
        let contributed = evidence_by_action
            .get(&action.action_id)
            .cloned()
            .unwrap_or_default();
        match fingerprints.entry(fingerprint) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(contributed);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                if contributed.is_subset(entry.get()) {
                    duplicate_tool_calls += 1;
                }
                entry.get_mut().extend(contributed);
            }
        }
    }
    Ok(CoordinationCounts {
        duplicate_tool_calls,
        useful_messages: observed
            .messages
            .iter()
            .filter(|message| message_is_useful(message, observed))
            .count() as u64,
        operational_messages: observed.messages.len() as u64,
    })
}

pub(super) fn evaluate(input: &EvaluationInput, metrics: &mut BTreeMap<String, MetricValue>) {
    metrics.insert(
        "duplicate_tool_work".into(),
        if input.tool_calls_used == 0 {
            sets::unavailable(
                Applicability::ZeroDenominator,
                MetricDirection::LowerIsBetter,
            )
        } else {
            sets::counted(
                input.duplicate_tool_calls,
                input.tool_calls_used,
                MetricDirection::LowerIsBetter,
            )
        },
    );
    metrics.insert(
        "useful_communication".into(),
        sets::counted(
            input.useful_messages,
            input.operational_messages,
            MetricDirection::HigherIsBetter,
        ),
    );
}

fn evidence_by_action(observed: &ObservedRun) -> BTreeMap<ActionId, BTreeSet<EvidenceId>> {
    let mut by_action = BTreeMap::<ActionId, BTreeSet<EvidenceId>>::new();
    for item in observed.evidence.values() {
        for action_id in &item.evidence.source_action_ids {
            by_action
                .entry(action_id.clone())
                .or_default()
                .insert(item.evidence.id.clone());
        }
    }
    by_action
}

fn message_is_useful(message: &&crate::ObservedMessage, observed: &ObservedRun) -> bool {
    let action = observed.actions.values().any(|action| {
        action.caused_by_message_id.as_ref() == Some(&message.message_id)
            && action.request_sequence > message.sequence
            && action.agent_id == message.target_agent_id
            && message
                .task_id
                .as_ref()
                .is_none_or(|task| task == &action.task_id)
    });
    action
        || observed.task_transitions.iter().any(|transition| {
            transition.caused_by_message_id.as_ref() == Some(&message.message_id)
                && transition.sequence > message.sequence
                && transition.agent_id == message.target_agent_id
                && message
                    .task_id
                    .as_ref()
                    .is_none_or(|task| task == &transition.task_id)
        })
}

#[cfg(test)]
mod tests;
