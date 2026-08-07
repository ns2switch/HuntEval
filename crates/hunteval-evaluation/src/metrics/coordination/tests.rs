use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    ActionId, AgentId, Confidence, EpisodeId, EventId, Evidence, EvidenceId, MessageId, RunId,
    TaskId, TaskState,
};

use crate::{
    EvaluationError, ObservedAction, ObservedEvidence, ObservedMessage, ObservedRun,
    ObservedTaskTransition, ObservedToolOutcome,
};

use super::reduce;

#[test]
fn coordination_canonical_fingerprint_distinguishes_new_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let mut observed = observed()?;
    let first = action("action-1", 1, r#"{"query":"x","limit":1}"#)?;
    let second = action("action-2", 2, r#"{"limit":1,"query":"x"}"#)?;
    let third = action("action-3", 3, r#"{"query":"x","limit":1}"#)?;
    observed.actions = [first, second, third]
        .into_iter()
        .map(|action| (action.action_id.clone(), action))
        .collect();
    observed.evidence.insert(
        EvidenceId::new("evidence-1")?,
        evidence("evidence-1", "action-1")?,
    );
    observed.evidence.insert(
        EvidenceId::new("evidence-2")?,
        evidence("evidence-2", "action-2")?,
    );
    let counts = reduce(&observed)?;
    assert_eq!(counts.duplicate_tool_calls, 1);
    assert_eq!(counts.operational_messages, 0);
    Ok(())
}

#[test]
fn coordination_rejects_invalid_tool_names() -> Result<(), Box<dyn std::error::Error>> {
    let mut observed = observed()?;
    let mut invalid = action("action-1", 1, r#"{"query":"x"}"#)?;
    invalid.tool = "DuckDB SQL".to_owned();
    observed.actions.insert(invalid.action_id.clone(), invalid);
    assert!(matches!(
        reduce(&observed),
        Err(EvaluationError::InvalidToolName)
    ));
    let mut nested = serde_json::Value::Null;
    for _ in 0..=65 {
        nested = serde_json::Value::Array(vec![nested]);
    }
    let mut oversized = action("action-2", 2, r#"{}"#)?;
    oversized.arguments = nested;
    observed.actions.clear();
    observed
        .actions
        .insert(oversized.action_id.clone(), oversized);
    assert!(matches!(
        reduce(&observed),
        Err(EvaluationError::InvalidToolArguments)
    ));
    Ok(())
}

#[test]
fn coordination_counts_only_causal_target_effects() -> Result<(), Box<dyn std::error::Error>> {
    let mut observed = observed()?;
    let investigator = AgentId::new("investigator")?;
    let replacement = AgentId::new("replacement")?;
    let task_id = TaskId::new("task-1")?;
    observed.messages = vec![
        message("message-action", 1, &investigator, &task_id)?,
        message("message-reassign", 3, &replacement, &task_id)?,
        message("message-cancel", 5, &investigator, &task_id)?,
        message("message-prose-only", 7, &investigator, &task_id)?,
        message("message-wrong-target", 8, &replacement, &task_id)?,
    ];
    let mut causal_action = action("action-1", 2, r#"{"query":"x"}"#)?;
    causal_action.caused_by_message_id = Some(MessageId::new("message-action")?);
    observed
        .actions
        .insert(causal_action.action_id.clone(), causal_action);
    let mut wrong_target = action("action-2", 9, r#"{"query":"different"}"#)?;
    wrong_target.caused_by_message_id = Some(MessageId::new("message-wrong-target")?);
    observed
        .actions
        .insert(wrong_target.action_id.clone(), wrong_target);
    observed.task_transitions = vec![
        transition(
            "transition-reassign",
            4,
            "message-reassign",
            replacement,
            &task_id,
            TaskState::Reassigned,
        )?,
        transition(
            "transition-cancel",
            6,
            "message-cancel",
            investigator,
            &task_id,
            TaskState::Cancelled,
        )?,
    ];
    let counts = reduce(&observed)?;
    assert_eq!(counts.useful_messages, 3);
    assert_eq!(counts.operational_messages, 5);
    Ok(())
}

fn observed() -> Result<ObservedRun, Box<dyn std::error::Error>> {
    Ok(ObservedRun {
        run_id: RunId::new("run-1")?,
        episode_id: EpisodeId::new("episode-1")?,
        actions: BTreeMap::new(),
        tasks: BTreeMap::new(),
        evidence: BTreeMap::new(),
        findings: BTreeMap::new(),
        messages: Vec::new(),
        task_transitions: Vec::new(),
        message_sequences: BTreeMap::new(),
        timeline: None,
    })
}

fn action(
    id: &str,
    sequence: u64,
    arguments: &str,
) -> Result<ObservedAction, Box<dyn std::error::Error>> {
    Ok(ObservedAction {
        action_id: ActionId::new(id)?,
        agent_id: AgentId::new("investigator")?,
        task_id: TaskId::new("task-1")?,
        request_message_id: MessageId::new(format!("request-{id}"))?,
        request_sequence: sequence,
        caused_by_message_id: None,
        result_message_id: MessageId::new(format!("result-{id}"))?,
        tool: "duckdb_sql".to_owned(),
        purpose: "Inspect events".to_owned(),
        arguments: serde_json::from_str(arguments)?,
        outcome: ObservedToolOutcome::Success,
        event_ids: BTreeSet::from([EventId::new(format!("event-{id}"))?]),
        result: serde_json::json!({"rows": []}),
    })
}

fn evidence(id: &str, action_id: &str) -> Result<ObservedEvidence, Box<dyn std::error::Error>> {
    Ok(ObservedEvidence {
        agent_id: AgentId::new("investigator")?,
        task_id: TaskId::new("task-1")?,
        message_id: MessageId::new(format!("message-{id}"))?,
        evidence: Evidence {
            id: EvidenceId::new(id)?,
            summary: "Grounded evidence".to_owned(),
            source_action_ids: BTreeSet::from([ActionId::new(action_id)?]),
            event_ids: BTreeSet::new(),
            entity_ids: BTreeSet::new(),
            time_range: None,
            confidence: Confidence::new(1.0)?,
        },
    })
}

fn message(
    id: &str,
    sequence: u64,
    target: &AgentId,
    task_id: &TaskId,
) -> Result<ObservedMessage, Box<dyn std::error::Error>> {
    Ok(ObservedMessage {
        message_id: MessageId::new(id)?,
        sequence,
        caused_by_message_id: None,
        agent_id: AgentId::new("supervisor")?,
        target_agent_id: target.clone(),
        task_id: Some(task_id.clone()),
        reason_code: "observable_handoff".to_owned(),
        message: "Untrusted prose is not evaluated".to_owned(),
    })
}

fn transition(
    id: &str,
    sequence: u64,
    cause: &str,
    agent_id: AgentId,
    task_id: &TaskId,
    state: TaskState,
) -> Result<ObservedTaskTransition, Box<dyn std::error::Error>> {
    Ok(ObservedTaskTransition {
        message_id: MessageId::new(id)?,
        sequence,
        caused_by_message_id: Some(MessageId::new(cause)?),
        agent_id,
        task_id: task_id.clone(),
        state,
    })
}
