use std::collections::BTreeSet;

use hunteval_domain::EventId;
use thiserror::Error;

use super::types::{ObservedToolOutcome, TrustedRunInput};

pub(super) fn validate_input(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    input
        .ground_truth
        .validate()
        .map_err(|_| TrustedViewError::InvalidGroundTruth)?;
    input
        .submission
        .validate()
        .map_err(|_| TrustedViewError::InvalidSubmission)?;
    if input.observed.run_id != input.provenance.run_id {
        return Err(TrustedViewError::CrossRunReference);
    }
    if input.observed.episode_id != input.ground_truth.episode_id {
        return Err(TrustedViewError::EpisodeMismatch);
    }
    if input.submission != input.terminal_submission {
        return Err(TrustedViewError::SubmissionMismatch);
    }
    if input.observed.timeline != input.submission.timeline {
        return Err(TrustedViewError::SubmissionMismatch);
    }
    if input.observed.actions.len() as u64 > input.tool_call_limit {
        return Err(TrustedViewError::ToolBudgetExceeded);
    }
    validate_actions_and_evidence(input)?;
    validate_findings(input)?;
    validate_causality(input)?;
    validate_submission(input)
}

fn validate_causality(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    let sequences = &input.observed.message_sequences;
    let mut unique_sequences = BTreeSet::new();
    if sequences
        .values()
        .any(|sequence| *sequence == 0 || !unique_sequences.insert(*sequence))
    {
        return Err(TrustedViewError::InvalidCausalReference);
    }
    for action in input.observed.actions.values() {
        validate_sequence(
            sequences,
            &action.request_message_id,
            action.request_sequence,
            action.caused_by_message_id.as_ref(),
        )?;
    }
    for message in &input.observed.messages {
        if message
            .task_id
            .as_ref()
            .is_some_and(|task| !input.observed.tasks.contains_key(task))
        {
            return Err(TrustedViewError::UnknownTask);
        }
        validate_sequence(
            sequences,
            &message.message_id,
            message.sequence,
            message.caused_by_message_id.as_ref(),
        )?;
    }
    for transition in &input.observed.task_transitions {
        if !input.observed.tasks.contains_key(&transition.task_id) {
            return Err(TrustedViewError::UnknownTask);
        }
        validate_sequence(
            sequences,
            &transition.message_id,
            transition.sequence,
            transition.caused_by_message_id.as_ref(),
        )?;
    }
    Ok(())
}

fn validate_sequence(
    sequences: &std::collections::BTreeMap<hunteval_domain::MessageId, u64>,
    message_id: &hunteval_domain::MessageId,
    sequence: u64,
    caused_by: Option<&hunteval_domain::MessageId>,
) -> Result<(), TrustedViewError> {
    if sequences.get(message_id) != Some(&sequence)
        || caused_by.is_some_and(|cause| {
            sequences
                .get(cause)
                .is_none_or(|cause_sequence| *cause_sequence >= sequence)
        })
    {
        return Err(TrustedViewError::InvalidCausalReference);
    }
    Ok(())
}

fn validate_actions_and_evidence(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    for item in input.observed.evidence.values() {
        let _task = input
            .observed
            .tasks
            .get(&item.task_id)
            .ok_or(TrustedViewError::UnknownTask)?;
        let mut issued_events = BTreeSet::new();
        for action_id in &item.evidence.source_action_ids {
            let action = input
                .observed
                .actions
                .get(action_id)
                .ok_or(TrustedViewError::UnknownAction)?;
            if action.agent_id != item.agent_id || action.task_id != item.task_id {
                return Err(TrustedViewError::WrongAgentOwnership);
            }
            if action.outcome != ObservedToolOutcome::Success {
                return Err(TrustedViewError::UnissuedEventReference);
            }
            issued_events.extend(action.event_ids.iter().cloned());
        }
        if !item.evidence.event_ids.is_subset(&issued_events) {
            return Err(TrustedViewError::UnissuedEventReference);
        }
    }
    Ok(())
}

fn validate_findings(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    for item in input.observed.findings.values() {
        let mut evidence_events = BTreeSet::<EventId>::new();
        let mut evidence_entities = BTreeSet::new();
        for evidence_id in &item.finding.evidence_ids {
            let evidence = input
                .observed
                .evidence
                .get(evidence_id)
                .ok_or(TrustedViewError::UnknownEvidence)?;
            evidence_events.extend(evidence.evidence.event_ids.iter().cloned());
            evidence_entities.extend(evidence.evidence.entity_ids.iter().cloned());
        }
        if !item.finding.event_ids.is_subset(&evidence_events)
            || !item.finding.entity_ids.is_subset(&evidence_entities)
        {
            return Err(TrustedViewError::UngroundedFinding);
        }
    }
    Ok(())
}

fn validate_submission(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    let mut finding_events = BTreeSet::<EventId>::new();
    let mut finding_entities = BTreeSet::new();
    let mut finding_techniques = BTreeSet::new();
    for finding_id in &input.submission.finding_ids {
        let finding = input
            .observed
            .findings
            .get(finding_id)
            .ok_or(TrustedViewError::UnknownFinding)?;
        finding_events.extend(finding.finding.event_ids.iter().cloned());
        finding_entities.extend(finding.finding.entity_ids.iter().cloned());
        finding_techniques.extend(finding.finding.attack_techniques.iter().cloned());
    }
    if !input
        .submission
        .malicious_event_ids
        .is_subset(&finding_events)
        || !input
            .submission
            .malicious_entity_ids
            .is_subset(&finding_entities)
    {
        return Err(TrustedViewError::UngroundedSubmission);
    }
    if !input
        .submission
        .attack_techniques
        .is_subset(&finding_techniques)
    {
        return Err(TrustedViewError::UngroundedSubmission);
    }
    if !input
        .submission
        .attack_path
        .iter()
        .all(|event| finding_events.contains(event))
    {
        return Err(TrustedViewError::UngroundedSubmission);
    }
    validate_timeline_references(input)
}

fn validate_timeline_references(input: &TrustedRunInput) -> Result<(), TrustedViewError> {
    for entry in input.observed.timeline.iter().flatten() {
        if !input
            .submission
            .malicious_event_ids
            .contains(&entry.event_id)
            || !entry.evidence_ids.iter().all(|id| {
                input
                    .observed
                    .evidence
                    .get(id)
                    .is_some_and(|evidence| evidence.evidence.event_ids.contains(&entry.event_id))
            })
        {
            return Err(TrustedViewError::InvalidTimelineReference);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TrustedViewError {
    #[error("trusted view ground truth is invalid")]
    InvalidGroundTruth,
    #[error("trusted view submission is invalid")]
    InvalidSubmission,
    #[error("trusted view contains a cross-run reference")]
    CrossRunReference,
    #[error("trusted view episode does not match ground truth")]
    EpisodeMismatch,
    #[error("stored and terminal submissions do not match")]
    SubmissionMismatch,
    #[error("observed tool calls exceed the trusted limit")]
    ToolBudgetExceeded,
    #[error("trusted view references an unknown task")]
    UnknownTask,
    #[error("trusted view references an unknown action")]
    UnknownAction,
    #[error("trusted view references unknown evidence")]
    UnknownEvidence,
    #[error("trusted view references an unknown finding")]
    UnknownFinding,
    #[error("an observed item has the wrong agent or task owner")]
    WrongAgentOwnership,
    #[error("evidence references an event not issued by its actions")]
    UnissuedEventReference,
    #[error("finding references events or entities absent from its evidence")]
    UngroundedFinding,
    #[error("submission references events or entities absent from its findings")]
    UngroundedSubmission,
    #[error("timeline entry has invalid evidence or event provenance")]
    InvalidTimelineReference,
    #[error("observable causality references an unknown or non-prior message")]
    InvalidCausalReference,
    #[error("observable coordination input is not canonical or bounded")]
    InvalidCoordinationInput,
}
