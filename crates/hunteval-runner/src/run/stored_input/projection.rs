mod action;
mod task;

use std::collections::BTreeMap;

use hunteval_domain::{ActionId, FinalSubmission, RunId, TaskId, TaskState};
use hunteval_evaluation::{
    ObservedAction, ObservedEvidence, ObservedFinding, ObservedMessage, ObservedRun, ObservedTask,
    ObservedTaskTransition,
};
use hunteval_protocol::{ProtocolPayload, StoredEvent};

use super::StoredEvaluationError;
use task::ProjectedTaskTransition;

#[derive(Debug)]
struct PendingAction {
    agent_id: hunteval_domain::AgentId,
    task_id: TaskId,
    request_message_id: hunteval_domain::MessageId,
    request_sequence: u64,
    caused_by_message_id: Option<hunteval_domain::MessageId>,
    tool: String,
    purpose: String,
    arguments: serde_json::Value,
}

#[derive(Debug)]
pub(super) struct ReplayProjection {
    expected_run_id: RunId,
    episode_id: Option<hunteval_domain::EpisodeId>,
    actions: BTreeMap<ActionId, ObservedAction>,
    pending_actions: BTreeMap<ActionId, PendingAction>,
    tasks: BTreeMap<TaskId, ObservedTask>,
    evidence: BTreeMap<hunteval_domain::EvidenceId, ObservedEvidence>,
    findings: BTreeMap<hunteval_domain::FindingId, ObservedFinding>,
    messages: Vec<ObservedMessage>,
    task_transitions: Vec<ObservedTaskTransition>,
    message_sequences: BTreeMap<hunteval_domain::MessageId, u64>,
    event_timestamps: BTreeMap<u64, hunteval_domain::UtcTimestamp>,
    submission: Option<FinalSubmission>,
    completed_termination: bool,
}

impl ReplayProjection {
    pub(super) fn new(expected_run_id: RunId) -> Self {
        Self {
            expected_run_id,
            episode_id: None,
            actions: BTreeMap::new(),
            pending_actions: BTreeMap::new(),
            tasks: BTreeMap::new(),
            evidence: BTreeMap::new(),
            findings: BTreeMap::new(),
            messages: Vec::new(),
            task_transitions: Vec::new(),
            message_sequences: BTreeMap::new(),
            event_timestamps: BTreeMap::new(),
            submission: None,
            completed_termination: false,
        }
    }

    pub(super) fn apply(&mut self, event: StoredEvent) -> Result<(), StoredEvaluationError> {
        let sequence = event.sequence;
        let envelope = event.envelope;
        if envelope.run_id != self.expected_run_id {
            return Err(StoredEvaluationError::InvalidProjection);
        }
        self.message_sequences
            .insert(envelope.message_id.clone(), sequence);
        self.event_timestamps.insert(sequence, envelope.timestamp);
        let caused_by_message_id = envelope.caused_by_message_id.clone();
        match envelope.payload {
            ProtocolPayload::RunStarted { episode_id, .. } => self.episode_id = Some(episode_id),
            ProtocolPayload::TaskCreated { agent_id, task } => {
                self.create_task(
                    envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id,
                    task,
                );
            }
            ProtocolPayload::TaskDelegated {
                task_id,
                target_agent_id,
                ..
            } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id: target_agent_id.clone(),
                    task_id,
                    state: TaskState::Delegated,
                    assignee_agent_id: Some(target_agent_id),
                    terminal: false,
                })?;
            }
            ProtocolPayload::TaskStarted { agent_id, task_id } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id,
                    task_id,
                    state: TaskState::Started,
                    assignee_agent_id: None,
                    terminal: false,
                })?;
            }
            ProtocolPayload::TaskCompleted { agent_id, task_id } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id,
                    task_id,
                    state: TaskState::Completed,
                    assignee_agent_id: None,
                    terminal: true,
                })?;
            }
            ProtocolPayload::TaskFailed {
                agent_id, task_id, ..
            } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id,
                    task_id,
                    state: TaskState::Failed,
                    assignee_agent_id: None,
                    terminal: true,
                })?;
            }
            ProtocolPayload::TaskReassigned {
                task_id,
                target_agent_id,
                ..
            } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id: target_agent_id.clone(),
                    task_id,
                    state: TaskState::Reassigned,
                    assignee_agent_id: Some(target_agent_id),
                    terminal: false,
                })?;
            }
            ProtocolPayload::TaskCancelled { agent_id, task_id } => {
                self.transition_task(ProjectedTaskTransition {
                    message_id: envelope.message_id,
                    sequence,
                    caused_by_message_id,
                    agent_id,
                    task_id,
                    state: TaskState::Cancelled,
                    assignee_agent_id: None,
                    terminal: true,
                })?;
            }
            ProtocolPayload::ToolRequest {
                agent_id,
                task_id,
                action_id,
                tool,
                purpose,
                arguments,
            } => {
                self.pending_actions.insert(
                    action_id,
                    PendingAction {
                        agent_id,
                        task_id,
                        request_message_id: envelope.message_id,
                        request_sequence: sequence,
                        caused_by_message_id,
                        tool,
                        purpose,
                        arguments,
                    },
                );
            }
            ProtocolPayload::ToolResult {
                action_id,
                tool,
                outcome,
                event_ids,
                result,
            } => self.complete_action(
                action_id,
                envelope.message_id,
                tool,
                outcome,
                event_ids,
                result,
            )?,
            ProtocolPayload::EvidenceShared {
                agent_id,
                task_id,
                evidence,
            } => {
                self.evidence.insert(
                    evidence.id.clone(),
                    ObservedEvidence {
                        agent_id,
                        task_id,
                        message_id: envelope.message_id,
                        evidence,
                    },
                );
            }
            ProtocolPayload::FindingProposed {
                agent_id,
                task_id,
                finding,
            } => {
                self.findings.insert(
                    finding.id.clone(),
                    ObservedFinding {
                        agent_id,
                        task_id,
                        message_id: envelope.message_id,
                        finding,
                    },
                );
            }
            ProtocolPayload::OperationalMessage {
                agent_id,
                target_agent_id,
                task_id,
                reason_code,
                message,
            } => self.messages.push(ObservedMessage {
                message_id: envelope.message_id,
                sequence,
                caused_by_message_id,
                agent_id,
                target_agent_id,
                task_id,
                reason_code,
                message,
            }),
            ProtocolPayload::FinalSubmission { submission, .. } => {
                if self.submission.replace(submission).is_some() {
                    return Err(StoredEvaluationError::InvalidProjection);
                }
            }
            ProtocolPayload::RunTerminated { status } => {
                self.completed_termination = status == "completed";
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn finish(self) -> Result<(ObservedRun, FinalSubmission), StoredEvaluationError> {
        if !self.pending_actions.is_empty() || !self.completed_termination {
            return Err(StoredEvaluationError::InvalidProjection);
        }
        let episode_id = self
            .episode_id
            .ok_or(StoredEvaluationError::InvalidProjection)?;
        let submission = self
            .submission
            .ok_or(StoredEvaluationError::InvalidProjection)?;
        let timeline = submission.timeline.clone();
        Ok((
            ObservedRun {
                run_id: self.expected_run_id,
                episode_id,
                actions: self.actions,
                tasks: self.tasks,
                evidence: self.evidence,
                findings: self.findings,
                messages: self.messages,
                task_transitions: self.task_transitions,
                message_sequences: self.message_sequences,
                event_timestamps: self.event_timestamps,
                timeline,
            },
            submission,
        ))
    }
}
