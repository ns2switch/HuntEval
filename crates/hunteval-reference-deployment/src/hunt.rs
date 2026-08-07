use std::{
    collections::BTreeSet,
    io::{BufRead, Write},
};

use hunteval_domain::{
    ActionId, AgentId, Confidence, Evidence, EvidenceId, FinalSubmission, Finding, FindingId,
    FindingSeverity, SubmissionStatus, TaskId, TaskPriority, TaskSpec,
};
use hunteval_protocol::{ProtocolPayload, ToolOutcome};

use crate::{ReferenceError, ReferenceTopology, peer::Peer};

pub(super) fn execute<R: BufRead, W: Write>(
    topology: ReferenceTopology,
    tables: &BTreeSet<String>,
    peer: &mut Peer<R, W>,
) -> Result<(), ReferenceError> {
    let coordinator = topology.coordinator()?;
    let investigator = topology.investigator()?;
    let task_id = TaskId::new(format!("task-{}", peer.seed))?;
    let action_id = ActionId::new(format!("action-{}", peer.seed))?;

    peer.send(
        None,
        ProtocolPayload::TaskCreated {
            agent_id: coordinator.clone(),
            task: TaskSpec {
                id: task_id.clone(),
                objective: "Identify activity associated with the observable suspicious source."
                    .to_owned(),
                priority: TaskPriority::High,
                dependencies: BTreeSet::new(),
                required_capabilities: BTreeSet::from(["iam_analysis".to_owned()]),
                parent_task_id: None,
            },
        },
    )?;
    peer.send(
        None,
        ProtocolPayload::TaskDelegated {
            agent_id: coordinator.clone(),
            task_id: task_id.clone(),
            target_agent_id: investigator.clone(),
            reason_code: "observable_capability_match".to_owned(),
        },
    )?;
    peer.send(
        None,
        ProtocolPayload::TaskStarted {
            agent_id: investigator.clone(),
            task_id: task_id.clone(),
        },
    )?;

    let table = select_table(tables)?;
    let query = format!(
        "SELECT event_id, principal, event_time, event_name FROM {table} WHERE source_ip = ? ORDER BY event_time, event_id"
    );
    let request_message = peer.send(
        None,
        ProtocolPayload::ToolRequest {
            agent_id: investigator.clone(),
            task_id: task_id.clone(),
            action_id: action_id.clone(),
            tool: "duckdb_sql".to_owned(),
            purpose: "Inspect events sharing an observable suspicious source address.".to_owned(),
            arguments: serde_json::json!({
                "query": query,
                "parameters": [{"type": "string", "value": "203.0.113.77"}]
            }),
        },
    )?;
    let tool_result = peer.receive()?;
    let (outcome, event_ids, result) = match tool_result.payload {
        ProtocolPayload::ToolResult {
            action_id: returned_action,
            outcome,
            event_ids,
            result,
            ..
        } if returned_action == action_id
            && tool_result.caused_by_message_id.as_ref() == Some(&request_message) =>
        {
            (outcome, event_ids, result)
        }
        _ => return Err(ReferenceError::InvalidRunnerMessage),
    };

    match outcome {
        ToolOutcome::Success => complete_successful_hunt(
            coordinator,
            investigator,
            task_id,
            action_id,
            event_ids,
            result,
            peer,
        ),
        ToolOutcome::Error => complete_failed_hunt(coordinator, investigator, task_id, peer),
    }
}

fn complete_successful_hunt<R: BufRead, W: Write>(
    coordinator: AgentId,
    investigator: AgentId,
    task_id: TaskId,
    action_id: ActionId,
    event_ids: BTreeSet<hunteval_domain::EventId>,
    result: serde_json::Value,
    peer: &mut Peer<R, W>,
) -> Result<(), ReferenceError> {
    let entity_ids = extract_principals(&result)?;
    let (status, finding_ids) = if event_ids.is_empty() {
        (SubmissionStatus::NoMaliciousActivity, BTreeSet::new())
    } else {
        let evidence_id = EvidenceId::new(format!("evidence-{}", peer.seed))?;
        let finding_id = FindingId::new(format!("finding-{}", peer.seed))?;
        peer.send(
            None,
            ProtocolPayload::EvidenceShared {
                agent_id: investigator.clone(),
                task_id: task_id.clone(),
                evidence: Evidence {
                    id: evidence_id.clone(),
                    summary: "Managed SQL returned events associated with the suspicious source."
                        .to_owned(),
                    source_action_ids: BTreeSet::from([action_id]),
                    event_ids: event_ids.clone(),
                    entity_ids: entity_ids.clone(),
                    time_range: None,
                    confidence: Confidence::new(0.9)?,
                },
            },
        )?;
        peer.send(
            None,
            ProtocolPayload::FindingProposed {
                agent_id: investigator.clone(),
                task_id: task_id.clone(),
                finding: Finding {
                    id: finding_id.clone(),
                    title: "Suspicious identity activity from a shared source".to_owned(),
                    severity: FindingSeverity::High,
                    evidence_ids: BTreeSet::from([evidence_id]),
                    event_ids: event_ids.clone(),
                    entity_ids: entity_ids.clone(),
                    attack_techniques: BTreeSet::from(["T1078".to_owned()]),
                    benign_alternatives: vec!["Authorized administrative activity".to_owned()],
                    confidence: Confidence::new(0.9)?,
                },
            },
        )?;
        (
            SubmissionStatus::ConfirmedMaliciousActivity,
            BTreeSet::from([finding_id]),
        )
    };
    let attack_techniques = if event_ids.is_empty() {
        BTreeSet::new()
    } else {
        BTreeSet::from(["T1078".to_owned()])
    };
    peer.send(
        None,
        ProtocolPayload::TaskCompleted {
            agent_id: investigator,
            task_id,
        },
    )?;
    peer.send(
        None,
        ProtocolPayload::FinalSubmission {
            agent_id: coordinator,
            submission: FinalSubmission {
                status,
                summary: "The reference deployment classified the observable managed-tool results."
                    .to_owned(),
                finding_ids,
                malicious_event_ids: event_ids.clone(),
                malicious_entity_ids: entity_ids,
                attack_path: event_ids.into_iter().collect(),
                attack_techniques,
                confidence: Confidence::new(0.9)?,
                limitations: Vec::new(),
            },
        },
    )?;
    Ok(())
}

fn complete_failed_hunt<R: BufRead, W: Write>(
    coordinator: AgentId,
    investigator: AgentId,
    task_id: TaskId,
    peer: &mut Peer<R, W>,
) -> Result<(), ReferenceError> {
    peer.send(
        None,
        ProtocolPayload::TaskFailed {
            agent_id: investigator,
            task_id,
            reason_code: "managed_tool_error".to_owned(),
        },
    )?;
    peer.send(
        None,
        ProtocolPayload::FinalSubmission {
            agent_id: coordinator,
            submission: FinalSubmission {
                status: SubmissionStatus::Inconclusive,
                summary: "The managed tool failed, so the reference deployment is inconclusive."
                    .to_owned(),
                finding_ids: BTreeSet::new(),
                malicious_event_ids: BTreeSet::new(),
                malicious_entity_ids: BTreeSet::new(),
                attack_path: Vec::new(),
                attack_techniques: BTreeSet::new(),
                confidence: Confidence::new(0.0)?,
                limitations: vec!["Managed tool result was unavailable.".to_owned()],
            },
        },
    )?;
    Ok(())
}

fn select_table(tables: &BTreeSet<String>) -> Result<&str, ReferenceError> {
    let table = tables
        .iter()
        .next()
        .ok_or(ReferenceError::InvalidRunnerMessage)?;
    let mut bytes = table.bytes();
    let first = bytes.next().ok_or(ReferenceError::InvalidRunnerMessage)?;
    if !(first.is_ascii_lowercase() || first == b'_')
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ReferenceError::InvalidRunnerMessage);
    }
    Ok(table)
}

fn extract_principals(result: &serde_json::Value) -> Result<BTreeSet<String>, ReferenceError> {
    let columns = result
        .get("columns")
        .and_then(serde_json::Value::as_array)
        .ok_or(ReferenceError::InvalidToolResult)?;
    let principal_index = columns
        .iter()
        .position(|column| column.as_str() == Some("principal"))
        .ok_or(ReferenceError::InvalidToolResult)?;
    let rows = result
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .ok_or(ReferenceError::InvalidToolResult)?;
    let mut principals = BTreeSet::new();
    for row in rows {
        let value = row
            .as_array()
            .and_then(|values| values.get(principal_index))
            .and_then(|value| value.get("value"))
            .and_then(serde_json::Value::as_str)
            .ok_or(ReferenceError::InvalidToolResult)?;
        if value.is_empty() || value.len() > 4096 {
            return Err(ReferenceError::InvalidToolResult);
        }
        principals.insert(value.to_owned());
    }
    Ok(principals)
}
