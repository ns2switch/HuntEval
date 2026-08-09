use std::{collections::BTreeSet, path::Path, time::Duration};

use hunteval_domain::{EpisodeId, EpisodeLimits, MessageId, ProtocolVersion, RunId, UtcTimestamp};
use hunteval_protocol::{
    MessageOrigin, ProtocolEnvelope, ProtocolPayload, ProtocolSession, ToolOutcome,
    TrajectoryRecorder,
};
use serde::Serialize;

use super::transport::{ProtocolProcess, execution_policy};

const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::new(0, 3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceStatus {
    Conformant,
    NonConformant,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceResult {
    pub schema_version: String,
    pub protocol_version: String,
    pub status: ConformanceStatus,
    pub checks: Vec<String>,
    pub transcript_sha256: String,
}

#[must_use]
pub fn run_conformance(executable: &Path, arguments: &[String]) -> ConformanceResult {
    execute(executable, arguments).unwrap_or_else(|check| ConformanceResult {
        schema_version: "0.5".to_owned(),
        protocol_version: "0.3".to_owned(),
        status: if check == "unsupported_protocol" {
            ConformanceStatus::Unsupported
        } else {
            ConformanceStatus::NonConformant
        },
        checks: vec![check.to_owned()],
        transcript_sha256: "0".repeat(64),
    })
}

fn execute(executable: &Path, arguments: &[String]) -> Result<ConformanceResult, &'static str> {
    let public_root = executable
        .parent()
        .filter(|path| path.is_dir())
        .ok_or("unsafe_deployment")?;
    let policy =
        execution_policy(Duration::from_secs(10), 128 * 1024).map_err(|_| "execution_policy")?;
    let mut process = ProtocolProcess::spawn(
        executable,
        arguments,
        public_root,
        &Default::default(),
        policy,
        128 * 1024,
    )
    .map_err(|_| "sandbox_launch")?;
    let run_id = RunId::new("conformance-run").map_err(|_| "contract_setup")?;
    let timestamp: UtcTimestamp = "2026-01-01T00:00:00Z"
        .parse()
        .map_err(|_| "contract_setup")?;
    let mut session = ProtocolSession::new();
    let mut trajectory = TrajectoryRecorder::new();
    let mut runner_sequence = 1_u16;
    let started = envelope(
        &run_id,
        timestamp,
        next_id(&mut runner_sequence)?,
        None,
        ProtocolPayload::RunStarted {
            supported_minimum: PROTOCOL_VERSION,
            supported_maximum: PROTOCOL_VERSION,
            episode_id: EpisodeId::new("conformance-episode").map_err(|_| "contract_setup")?,
            objective: "Validate the public HuntEval protocol contract.".to_owned(),
            tables: BTreeSet::from(["synthetic_events".to_owned()]),
            limits: limits(),
            seed: 1,
        },
    );
    accept_and_send(&mut session, &mut trajectory, &mut process, started)?;

    let registration = receive(&mut session, &mut trajectory, &mut process)?;
    let selected = match &registration.payload {
        ProtocolPayload::RegisterDeployment {
            selected_protocol_version,
            ..
        } if *selected_protocol_version == PROTOCOL_VERSION => *selected_protocol_version,
        ProtocolPayload::RegisterDeployment { .. } => return Err("unsupported_protocol"),
        _ => return Err("registration"),
    };
    let accepted = envelope(
        &run_id,
        timestamp,
        next_id(&mut runner_sequence)?,
        Some(registration.message_id),
        ProtocolPayload::RegistrationAccepted {
            selected_protocol_version: selected,
        },
    );
    accept_and_send(&mut session, &mut trajectory, &mut process, accepted)?;

    let mut saw_tool_request = false;
    let mut saw_submission = false;
    for _ in 0..64 {
        let message = receive(&mut session, &mut trajectory, &mut process)?;
        match message.payload {
            ProtocolPayload::ToolRequest {
                action_id, tool, ..
            } => {
                saw_tool_request = true;
                let response = envelope(
                    &run_id,
                    timestamp,
                    next_id(&mut runner_sequence)?,
                    Some(message.message_id),
                    ProtocolPayload::ToolResult {
                        action_id,
                        tool,
                        outcome: ToolOutcome::Error,
                        event_ids: BTreeSet::new(),
                        result: serde_json::json!({"error": "synthetic_conformance_result"}),
                    },
                );
                accept_and_send(&mut session, &mut trajectory, &mut process, response)?;
            }
            ProtocolPayload::FinalSubmission { .. } => {
                saw_submission = true;
                let response = envelope(
                    &run_id,
                    timestamp,
                    next_id(&mut runner_sequence)?,
                    Some(message.message_id),
                    ProtocolPayload::RunTerminated {
                        status: "completed".to_owned(),
                    },
                );
                accept_and_send(&mut session, &mut trajectory, &mut process, response)?;
                break;
            }
            _ => {}
        }
    }
    process.finish().map_err(|_| "process_termination")?;
    session.finish().map_err(|_| "terminal_state")?;
    if !saw_tool_request || !saw_submission {
        return Err("required_flow");
    }
    Ok(ConformanceResult {
        schema_version: "0.5".to_owned(),
        protocol_version: "0.3".to_owned(),
        status: ConformanceStatus::Conformant,
        checks: vec![
            "registration".to_owned(),
            "managed_tool_mediation".to_owned(),
            "terminal_submission".to_owned(),
        ],
        transcript_sha256: trajectory.digest().to_string(),
    })
}

fn receive(
    session: &mut ProtocolSession,
    trajectory: &mut TrajectoryRecorder,
    process: &mut ProtocolProcess,
) -> Result<ProtocolEnvelope, &'static str> {
    let message = process.receive().map_err(|_| "protocol_receive")?;
    if message.payload.origin() != MessageOrigin::Deployment {
        return Err("message_origin");
    }
    session.accept(&message).map_err(|_| "protocol_state")?;
    trajectory
        .append(message.clone())
        .map_err(|_| "trajectory")?;
    Ok(message)
}

fn accept_and_send(
    session: &mut ProtocolSession,
    trajectory: &mut TrajectoryRecorder,
    process: &mut ProtocolProcess,
    message: ProtocolEnvelope,
) -> Result<(), &'static str> {
    session.accept(&message).map_err(|_| "protocol_state")?;
    trajectory
        .append(message.clone())
        .map_err(|_| "trajectory")?;
    process.send(&message).map_err(|_| "protocol_send")
}

fn envelope(
    run_id: &RunId,
    timestamp: UtcTimestamp,
    message_id: MessageId,
    caused_by_message_id: Option<MessageId>,
    payload: ProtocolPayload,
) -> ProtocolEnvelope {
    ProtocolEnvelope {
        protocol_version: PROTOCOL_VERSION,
        message_id,
        run_id: run_id.clone(),
        timestamp,
        caused_by_message_id,
        payload,
    }
}

fn next_id(sequence: &mut u16) -> Result<MessageId, &'static str> {
    let id = MessageId::new(format!("conformance-runner-{sequence:03}"))
        .map_err(|_| "contract_setup")?;
    *sequence = sequence.checked_add(1).ok_or("contract_setup")?;
    Ok(id)
}

fn limits() -> EpisodeLimits {
    EpisodeLimits {
        max_agents: 16,
        max_parallel_agents: 16,
        max_parallel_tool_calls: 4,
        max_outstanding_tasks: 64,
        max_delegation_depth: 8,
        max_tool_calls: 16,
        max_sql_queries: 16,
        max_retrieved_documents: 0,
        max_messages: 64,
        max_duration_seconds: 10,
        max_tokens: 100_000,
        max_estimated_cost: None,
    }
}
