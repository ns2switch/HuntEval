use std::collections::BTreeMap;

use hunteval_domain::{ActionId, FinalSubmission, RunId, TaskId, TaskState};
use hunteval_evaluation::{
    ObservedAction, ObservedEvidence, ObservedFinding, ObservedMessage, ObservedRun, ObservedTask,
    ObservedToolOutcome,
};
use hunteval_protocol::{ProtocolPayload, StoredEvent, ToolOutcome};

use super::StoredEvaluationError;

#[derive(Debug)]
struct PendingAction {
    agent_id: hunteval_domain::AgentId,
    task_id: TaskId,
    request_message_id: hunteval_domain::MessageId,
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
            submission: None,
            completed_termination: false,
        }
    }

    pub(super) fn apply(&mut self, event: StoredEvent) -> Result<(), StoredEvaluationError> {
        let envelope = event.envelope;
        if envelope.run_id != self.expected_run_id {
            return Err(StoredEvaluationError::InvalidProjection);
        }
        let caused_by_message_id = envelope.caused_by_message_id.clone();
        match envelope.payload {
            ProtocolPayload::RunStarted { episode_id, .. } => self.episode_id = Some(episode_id),
            ProtocolPayload::TaskCreated { agent_id, task } => {
                self.tasks.insert(
                    task.id.clone(),
                    ObservedTask {
                        task,
                        creator_agent_id: agent_id,
                        assignee_agent_id: None,
                        state: TaskState::Created,
                        created_message_id: envelope.message_id,
                        terminal_message_id: None,
                    },
                );
            }
            ProtocolPayload::TaskDelegated {
                task_id,
                target_agent_id,
                ..
            } => self.update_task(&task_id, TaskState::Delegated, Some(target_agent_id), None)?,
            ProtocolPayload::TaskStarted { task_id, .. } => {
                self.update_task(&task_id, TaskState::Started, None, None)?;
            }
            ProtocolPayload::TaskCompleted { task_id, .. } => self.update_task(
                &task_id,
                TaskState::Completed,
                None,
                Some(envelope.message_id),
            )?,
            ProtocolPayload::TaskFailed { task_id, .. } => {
                self.update_task(&task_id, TaskState::Failed, None, Some(envelope.message_id))?
            }
            ProtocolPayload::TaskReassigned {
                task_id,
                target_agent_id,
                ..
            } => self.update_task(&task_id, TaskState::Reassigned, Some(target_agent_id), None)?,
            ProtocolPayload::TaskCancelled { task_id, .. } => self.update_task(
                &task_id,
                TaskState::Cancelled,
                None,
                Some(envelope.message_id),
            )?,
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
        Ok((
            ObservedRun {
                run_id: self.expected_run_id,
                episode_id,
                actions: self.actions,
                tasks: self.tasks,
                evidence: self.evidence,
                findings: self.findings,
                messages: self.messages,
                timeline: Vec::new(),
            },
            submission,
        ))
    }

    fn update_task(
        &mut self,
        task_id: &TaskId,
        state: TaskState,
        assignee: Option<hunteval_domain::AgentId>,
        terminal_message_id: Option<hunteval_domain::MessageId>,
    ) -> Result<(), StoredEvaluationError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or(StoredEvaluationError::InvalidProjection)?;
        if let Some(assignee) = assignee {
            task.assignee_agent_id = Some(assignee);
        }
        task.state = state;
        task.terminal_message_id = terminal_message_id;
        Ok(())
    }

    fn complete_action(
        &mut self,
        action_id: ActionId,
        result_message_id: hunteval_domain::MessageId,
        tool: String,
        outcome: ToolOutcome,
        event_ids: std::collections::BTreeSet<hunteval_domain::EventId>,
        result: serde_json::Value,
    ) -> Result<(), StoredEvaluationError> {
        if outcome == ToolOutcome::Error && !event_ids.is_empty() {
            return Err(StoredEvaluationError::InvalidProjection);
        }
        let pending = self
            .pending_actions
            .remove(&action_id)
            .ok_or(StoredEvaluationError::InvalidProjection)?;
        if pending.tool != tool {
            return Err(StoredEvaluationError::InvalidProjection);
        }
        self.actions.insert(
            action_id.clone(),
            ObservedAction {
                action_id,
                agent_id: pending.agent_id,
                task_id: pending.task_id,
                request_message_id: pending.request_message_id,
                result_message_id,
                tool,
                purpose: pending.purpose,
                arguments: pending.arguments,
                outcome: match outcome {
                    ToolOutcome::Success => ObservedToolOutcome::Success,
                    ToolOutcome::Error => ObservedToolOutcome::Error,
                },
                event_ids,
                result,
            },
        );
        Ok(())
    }
}
