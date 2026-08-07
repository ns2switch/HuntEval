use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ProtocolEnvelope, ProtocolErrorCode, ProtocolPayload,
    message::{EvidenceIds, FindingIds},
};
use hunteval_domain::{
    ActionId, AgentId, MessageId, ProtocolVersion, RunId, TaskId, TaskRecord, TaskState,
};

mod action;
mod api;
mod fault;
mod support;

use action::ActionRecord;
pub use fault::ProtocolError;
use fault::{contract_error, error};

/// Ordered lifecycle of one bidirectional protocol session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolPhase {
    New,
    AwaitingRegistration,
    Registered,
    Active,
    Submitted,
    Terminated,
}

/// Validated replay state for one process session.
#[derive(Debug, Clone)]
pub struct ProtocolSession {
    phase: ProtocolPhase,
    run_id: Option<RunId>,
    supported_minimum: Option<ProtocolVersion>,
    supported_maximum: Option<ProtocolVersion>,
    selected_version: Option<ProtocolVersion>,
    max_agents: Option<u16>,
    registration_message_id: Option<MessageId>,
    messages: BTreeSet<MessageId>,
    agents: BTreeSet<AgentId>,
    tasks: BTreeMap<TaskId, TaskRecord>,
    actions: BTreeMap<ActionId, ActionRecord>,
    evidence: EvidenceIds,
    findings: FindingIds,
}

impl ProtocolSession {
    pub(super) fn apply(&mut self, message: &ProtocolEnvelope) -> Result<(), ProtocolError> {
        match &message.payload {
            ProtocolPayload::RunStarted {
                supported_minimum,
                supported_maximum,
                limits,
                ..
            } => {
                self.require_phase(ProtocolPhase::New)?;
                limits.validate().map_err(contract_error)?;
                if supported_minimum.major() != supported_maximum.major()
                    || supported_minimum > supported_maximum
                {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "invalid protocol range",
                    ));
                }
                self.run_id = Some(message.run_id.clone());
                self.supported_minimum = Some(*supported_minimum);
                self.supported_maximum = Some(*supported_maximum);
                self.max_agents = Some(limits.max_agents);
                self.phase = ProtocolPhase::AwaitingRegistration;
            }
            ProtocolPayload::RegisterDeployment {
                selected_protocol_version,
                deployment,
            } => {
                self.require_phase(ProtocolPhase::AwaitingRegistration)?;
                self.require_supported(*selected_protocol_version, message.protocol_version)?;
                deployment
                    .validate(self.max_agents.unwrap_or_default())
                    .map_err(contract_error)?;
                self.agents = deployment
                    .agents
                    .iter()
                    .map(|agent| agent.id.clone())
                    .collect();
                self.selected_version = Some(*selected_protocol_version);
                self.registration_message_id = Some(message.message_id.clone());
                self.phase = ProtocolPhase::Registered;
            }
            ProtocolPayload::RegistrationAccepted {
                selected_protocol_version,
            } => {
                self.require_phase(ProtocolPhase::Registered)?;
                if self.selected_version != Some(*selected_protocol_version)
                    || message.caused_by_message_id.as_ref()
                        != self.registration_message_id.as_ref()
                {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "registration correlation failed",
                    ));
                }
                self.phase = ProtocolPhase::Active;
            }
            ProtocolPayload::TaskCreated { agent_id, task } => {
                self.require_active_agent(agent_id)?;
                if self.tasks.contains_key(&task.id) {
                    return Err(error(
                        ProtocolErrorCode::DuplicateIdentifier,
                        "duplicate task identifier",
                    ));
                }
                for dependency in &task.dependencies {
                    self.require_task(dependency)?;
                }
                let record =
                    TaskRecord::create(task.clone(), agent_id.clone()).map_err(contract_error)?;
                self.tasks.insert(task.id.clone(), record);
            }
            ProtocolPayload::TaskDelegated {
                agent_id,
                task_id,
                target_agent_id,
                ..
            } => {
                self.require_active_agent(agent_id)?;
                self.require_agent(target_agent_id)?;
                let task = self.task_mut(task_id)?;
                if &task.creator != agent_id {
                    return Err(error(
                        ProtocolErrorCode::InvalidState,
                        "only task creator may delegate",
                    ));
                }
                task.delegate(target_agent_id.clone())
                    .map_err(contract_error)?;
            }
            ProtocolPayload::TaskStarted { agent_id, task_id } => {
                self.require_active_agent(agent_id)?;
                self.task_mut(task_id)?
                    .start(agent_id)
                    .map_err(contract_error)?;
            }
            ProtocolPayload::TaskCompleted { agent_id, task_id } => {
                self.require_active_agent(agent_id)?;
                self.task_mut(task_id)?
                    .complete(agent_id)
                    .map_err(contract_error)?;
            }
            ProtocolPayload::TaskFailed {
                agent_id, task_id, ..
            } => {
                self.require_active_agent(agent_id)?;
                self.task_mut(task_id)?
                    .fail(agent_id)
                    .map_err(contract_error)?;
            }
            ProtocolPayload::TaskReassigned {
                agent_id,
                task_id,
                target_agent_id,
            } => {
                self.require_active_agent(agent_id)?;
                self.require_agent(target_agent_id)?;
                let task = self.task_mut(task_id)?;
                if &task.creator != agent_id {
                    return Err(error(
                        ProtocolErrorCode::InvalidState,
                        "only task creator may reassign",
                    ));
                }
                task.reassign(target_agent_id.clone())
                    .map_err(contract_error)?;
            }
            ProtocolPayload::TaskCancelled { agent_id, task_id } => {
                self.require_active_agent(agent_id)?;
                self.task_mut(task_id)?.cancel().map_err(contract_error)?;
            }
            ProtocolPayload::OperationalMessage {
                agent_id,
                target_agent_id,
                task_id,
                message: operational_message,
                reason_code,
            } => {
                self.require_active_agent(agent_id)?;
                self.require_agent(target_agent_id)?;
                if let Some(task_id) = task_id {
                    self.require_task(task_id)?;
                }
                if operational_message.trim().is_empty() || reason_code.trim().is_empty() {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "operational message fields must not be empty",
                    ));
                }
            }
            ProtocolPayload::HypothesisUpdated {
                agent_id,
                task_id,
                status,
                reason_code,
                ..
            } => {
                self.require_active_agent(agent_id)?;
                self.require_task(task_id)?;
                if status.trim().is_empty() || reason_code.trim().is_empty() {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "hypothesis fields must not be empty",
                    ));
                }
            }
            ProtocolPayload::ToolRequest {
                agent_id,
                task_id,
                action_id,
                ..
            } => {
                self.require_active_agent(agent_id)?;
                let task = self.require_task(task_id)?;
                if task.state != TaskState::Started || task.assignee.as_ref() != Some(agent_id) {
                    return Err(error(
                        ProtocolErrorCode::InvalidState,
                        "tool request has no active assigned task",
                    ));
                }
                if self.actions.contains_key(action_id) {
                    return Err(error(
                        ProtocolErrorCode::DuplicateIdentifier,
                        "duplicate action identifier",
                    ));
                }
                self.actions.insert(
                    action_id.clone(),
                    ActionRecord {
                        request_message_id: message.message_id.clone(),
                        event_ids: None,
                    },
                );
            }
            ProtocolPayload::ToolResult {
                action_id,
                event_ids,
                ..
            } => {
                self.require_phase(ProtocolPhase::Active)?;
                let action = self.actions.get_mut(action_id).ok_or_else(|| {
                    error(
                        ProtocolErrorCode::UnknownAction,
                        "unknown action identifier",
                    )
                })?;
                if action.event_ids.is_some()
                    || message.caused_by_message_id.as_ref() != Some(&action.request_message_id)
                {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "tool result correlation failed",
                    ));
                }
                action.event_ids = Some(event_ids.clone());
            }
            ProtocolPayload::EvidenceShared {
                agent_id,
                task_id,
                evidence,
            } => {
                self.require_active_agent(agent_id)?;
                self.require_task(task_id)?;
                evidence.validate().map_err(contract_error)?;
                self.validate_provenance(evidence)?;
                if !self.evidence.insert(evidence.id.clone()) {
                    return Err(error(
                        ProtocolErrorCode::DuplicateIdentifier,
                        "duplicate evidence identifier",
                    ));
                }
            }
            ProtocolPayload::FindingProposed {
                agent_id,
                task_id,
                finding,
            } => {
                self.require_active_agent(agent_id)?;
                self.require_task(task_id)?;
                finding.validate().map_err(contract_error)?;
                if !finding
                    .evidence_ids
                    .iter()
                    .all(|id| self.evidence.contains(id))
                {
                    return Err(error(
                        ProtocolErrorCode::ProvenanceViolation,
                        "finding references unknown evidence",
                    ));
                }
                if !self.findings.insert(finding.id.clone()) {
                    return Err(error(
                        ProtocolErrorCode::DuplicateIdentifier,
                        "duplicate finding identifier",
                    ));
                }
            }
            ProtocolPayload::FindingReviewed {
                agent_id,
                finding_id,
                reason_code,
                ..
            } => {
                self.require_active_agent(agent_id)?;
                if !self.findings.contains(finding_id) {
                    return Err(error(
                        ProtocolErrorCode::InvalidState,
                        "review references unknown finding",
                    ));
                }
                if reason_code.trim().is_empty() {
                    return Err(error(
                        ProtocolErrorCode::InvalidMessage,
                        "review reason code must not be empty",
                    ));
                }
            }
            ProtocolPayload::FinalSubmission {
                agent_id,
                submission,
            } => {
                self.require_active_agent(agent_id)?;
                submission.validate().map_err(contract_error)?;
                if !submission
                    .finding_ids
                    .iter()
                    .all(|id| self.findings.contains(id))
                {
                    return Err(error(
                        ProtocolErrorCode::ProvenanceViolation,
                        "submission references unknown finding",
                    ));
                }
                self.phase = ProtocolPhase::Submitted;
            }
            ProtocolPayload::ProtocolError { .. } => self.require_not_new()?,
            ProtocolPayload::RunTerminated { .. } => {
                self.require_not_new()?;
                self.phase = ProtocolPhase::Terminated;
            }
        }
        Ok(())
    }
}
