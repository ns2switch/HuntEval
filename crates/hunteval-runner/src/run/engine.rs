use std::{collections::BTreeSet, path::Path};

use crate::{ArtifactWriter, BudgetLimits, ManagedTool, RunConfig, RunOrchestrator};
use hunteval_domain::{FinalSubmission, MessageId, ProtocolVersion};
use hunteval_protocol::{
    MessageOrigin, ProtocolEnvelope, ProtocolPayload, ToolOutcome, replay_trajectory,
};

use super::{
    ResolvedRunInputs,
    completion::{finalize_success, preserve_failure},
    error::{EngineError, RunnerMessageIds},
    evaluation::evaluate_validated_run,
    transport::ProtocolProcess,
    types::{RunExecution, RunFailure, RunFailureKind, RunRequest},
};

/// Provider-neutral application service for one mediated deployment run.
#[derive(Debug, Default)]
pub struct RunExecutor;

impl RunExecutor {
    pub fn execute(
        &self,
        request: &RunRequest,
        inputs: &ResolvedRunInputs,
        managed_tool: &dyn ManagedTool,
    ) -> Result<RunExecution, RunFailure> {
        let writer =
            ArtifactWriter::create(&request.output_root, &request.run_id).map_err(|_| {
                RunFailure {
                    kind: RunFailureKind::Artifact,
                    partial_artifacts: request
                        .output_root
                        .join(format!("{}.partial", request.run_id.as_str())),
                }
            })?;
        match execute_inner(request, inputs, managed_tool, &writer) {
            Ok(success) => finalize_success(success, writer, inputs, &request.run_id),
            Err(error) => preserve_failure(error.kind(), writer, request, inputs),
        }
    }
}

pub(super) struct PendingSuccess {
    pub(super) submission: FinalSubmission,
    pub(super) metrics: hunteval_evaluation::MetricVector,
    pub(super) aggregate_score: hunteval_evaluation::AggregateScore,
    pub(super) usage: crate::BudgetUsage,
}

fn execute_inner(
    request: &RunRequest,
    inputs: &ResolvedRunInputs,
    managed_tool: &dyn ManagedTool,
    writer: &ArtifactWriter,
) -> Result<PendingSuccess, EngineError> {
    if request.protocol_version != ProtocolVersion::new(0, 3)
        || request.maximum_line_bytes == 0
        || request.timeout.is_zero()
    {
        return Err(EngineError::InvalidConfiguration);
    }
    let manifest = &inputs.episode.public().manifest;
    let policy = crate::IsolationPolicy::new(
        inputs.episode.public().public_root.clone(),
        inputs.environment.clone(),
    )?;
    let expected_executable = inputs
        .hashes
        .get("deployment_executable")
        .ok_or(EngineError::InvalidConfiguration)?;
    if &crate::hash_file(&inputs.executable).map_err(|_| EngineError::InvalidConfiguration)?
        != expected_executable
    {
        return Err(EngineError::InvalidConfiguration);
    }
    let mut process = ProtocolProcess::spawn(
        &inputs.executable,
        &inputs.arguments,
        policy.public_root(),
        policy.environment(),
        request.timeout,
        request.maximum_line_bytes,
    )?;
    let mut orchestrator = RunOrchestrator::new(RunConfig {
        budgets: BudgetLimits {
            messages: u64::from(manifest.limits.max_messages),
            tool_calls: u64::from(manifest.limits.max_tool_calls),
            tokens: manifest.limits.max_tokens,
        },
        maximum_trajectory_line_bytes: request.maximum_line_bytes,
    });
    let mut ids = RunnerMessageIds::new(&request.run_id);
    let started = runner_message(
        request,
        ids.next()?,
        None,
        ProtocolPayload::RunStarted {
            supported_minimum: request.protocol_version,
            supported_maximum: request.protocol_version,
            episode_id: manifest.id.clone(),
            objective: manifest.objective.primary.clone(),
            tables: manifest
                .telemetry
                .tables
                .iter()
                .map(|table| table.name.clone())
                .collect(),
            limits: manifest.limits.clone(),
            seed: request.seed,
        },
    );
    send_runner(&mut orchestrator, &mut process, writer, started)?;

    let mut submission = None;
    loop {
        let incoming = process.receive()?;
        if incoming.payload.origin() != MessageOrigin::Deployment {
            return Err(EngineError::ProtocolViolation);
        }
        let incoming_id = incoming.message_id.clone();
        let response = match &incoming.payload {
            ProtocolPayload::RegisterDeployment {
                selected_protocol_version,
                ..
            } => Some(ProtocolPayload::RegistrationAccepted {
                selected_protocol_version: *selected_protocol_version,
            }),
            ProtocolPayload::ToolRequest {
                action_id,
                tool,
                arguments,
                ..
            } => {
                let (outcome, event_ids, result) = match managed_tool.execute(tool, arguments) {
                    Ok(output) => (ToolOutcome::Success, output.event_ids, output.result),
                    Err(error) => (
                        ToolOutcome::Error,
                        BTreeSet::new(),
                        serde_json::json!({"error": error.to_string()}),
                    ),
                };
                Some(ProtocolPayload::ToolResult {
                    action_id: action_id.clone(),
                    tool: tool.clone(),
                    outcome,
                    event_ids,
                    result,
                })
            }
            ProtocolPayload::FinalSubmission {
                submission: value, ..
            } => {
                submission = Some(value.clone());
                Some(ProtocolPayload::RunTerminated {
                    status: "completed".to_owned(),
                })
            }
            _ => None,
        };
        accept_deployment(&mut orchestrator, writer, incoming)?;
        if let Some(payload) = response {
            let terminal = matches!(payload, ProtocolPayload::RunTerminated { .. });
            let outgoing = runner_message(request, ids.next()?, Some(incoming_id), payload);
            send_runner(&mut orchestrator, &mut process, writer, outgoing)?;
            if terminal {
                break;
            }
        }
    }
    process.finish()?;
    let submission = submission.ok_or(EngineError::ProtocolViolation)?;
    replay_trajectory(orchestrator.trajectory(), request.maximum_line_bytes)?;
    let usage = orchestrator.usage();
    let evaluated = evaluate_validated_run(inputs, orchestrator.trajectory(), &submission, usage)?;
    writer.write_json(Path::new("submission.json"), &submission)?;
    writer.write_json(Path::new("metrics.json"), &evaluated.metrics)?;
    writer.write_json(
        Path::new("aggregate-score.json"),
        &evaluated.aggregate_score,
    )?;
    Ok(PendingSuccess {
        submission,
        metrics: evaluated.metrics,
        aggregate_score: evaluated.aggregate_score,
        usage,
    })
}

fn send_runner(
    orchestrator: &mut RunOrchestrator,
    process: &mut ProtocolProcess,
    writer: &ArtifactWriter,
    message: ProtocolEnvelope,
) -> Result<(), EngineError> {
    let previous_length = orchestrator.trajectory().len();
    orchestrator.accept_runner(message.clone())?;
    writer.append(
        Path::new("trajectory.jsonl"),
        &orchestrator.trajectory()[previous_length..],
    )?;
    process.send(&message)?;
    Ok(())
}

fn accept_deployment(
    orchestrator: &mut RunOrchestrator,
    writer: &ArtifactWriter,
    message: ProtocolEnvelope,
) -> Result<(), EngineError> {
    let previous_length = orchestrator.trajectory().len();
    orchestrator.accept_deployment(message)?;
    writer.append(
        Path::new("trajectory.jsonl"),
        &orchestrator.trajectory()[previous_length..],
    )?;
    Ok(())
}

fn runner_message(
    request: &RunRequest,
    message_id: MessageId,
    caused_by_message_id: Option<MessageId>,
    payload: ProtocolPayload,
) -> ProtocolEnvelope {
    ProtocolEnvelope {
        protocol_version: request.protocol_version,
        message_id,
        run_id: request.run_id.clone(),
        timestamp: request.started_at,
        caused_by_message_id,
        payload,
    }
}
