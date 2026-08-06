use std::collections::BTreeSet;

use hunteval_domain::{
    ActionId, AgentId, DeploymentRegistration, EpisodeId, EpisodeLimits, Evidence, EvidenceId,
    FinalSubmission, Finding, FindingId, MessageId, ProtocolVersion, RunId, TaskId, TaskSpec,
    UtcTimestamp,
};
use serde::{Deserialize, Serialize};

/// Trusted direction of a protocol payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageOrigin {
    Runner,
    Deployment,
}

/// Stable protocol error codes safe to expose to an untrusted deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorCode {
    UnsupportedProtocolVersion,
    InvalidMessage,
    InvalidState,
    UnknownAgent,
    UnknownTask,
    UnknownAction,
    DuplicateIdentifier,
    ProvenanceViolation,
    ProcessFailure,
}

/// Result status of a HuntEval-managed tool action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    Success,
    Error,
}

/// Common versioned envelope for every JSONL message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolEnvelope {
    pub protocol_version: ProtocolVersion,
    pub message_id: MessageId,
    pub run_id: RunId,
    pub timestamp: UtcTimestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caused_by_message_id: Option<MessageId>,
    #[serde(flatten)]
    pub payload: ProtocolPayload,
}

/// Versioned payload subset required by the first complete protocol flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProtocolPayload {
    RunStarted {
        supported_minimum: ProtocolVersion,
        supported_maximum: ProtocolVersion,
        episode_id: EpisodeId,
        objective: String,
        tables: BTreeSet<String>,
        limits: EpisodeLimits,
        seed: u64,
    },
    RegisterDeployment {
        selected_protocol_version: ProtocolVersion,
        deployment: DeploymentRegistration,
    },
    RegistrationAccepted {
        selected_protocol_version: ProtocolVersion,
    },
    TaskCreated {
        agent_id: AgentId,
        task: TaskSpec,
    },
    TaskDelegated {
        agent_id: AgentId,
        task_id: TaskId,
        target_agent_id: AgentId,
        reason_code: String,
    },
    TaskStarted {
        agent_id: AgentId,
        task_id: TaskId,
    },
    TaskCompleted {
        agent_id: AgentId,
        task_id: TaskId,
    },
    TaskFailed {
        agent_id: AgentId,
        task_id: TaskId,
        reason_code: String,
    },
    TaskReassigned {
        agent_id: AgentId,
        task_id: TaskId,
        target_agent_id: AgentId,
    },
    TaskCancelled {
        agent_id: AgentId,
        task_id: TaskId,
    },
    ToolRequest {
        agent_id: AgentId,
        task_id: TaskId,
        action_id: ActionId,
        tool: String,
        purpose: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        action_id: ActionId,
        tool: String,
        outcome: ToolOutcome,
        event_ids: BTreeSet<hunteval_domain::EventId>,
        result: serde_json::Value,
    },
    EvidenceShared {
        agent_id: AgentId,
        task_id: TaskId,
        evidence: Evidence,
    },
    FindingProposed {
        agent_id: AgentId,
        task_id: TaskId,
        finding: Finding,
    },
    FinalSubmission {
        agent_id: AgentId,
        submission: FinalSubmission,
    },
    ProtocolError {
        code: ProtocolErrorCode,
        message: String,
        retryable: bool,
    },
    RunTerminated {
        status: String,
    },
}

impl ProtocolPayload {
    /// Returns the only trusted origin allowed to emit this payload kind.
    #[must_use]
    pub const fn origin(&self) -> MessageOrigin {
        match self {
            Self::RunStarted { .. }
            | Self::RegistrationAccepted { .. }
            | Self::ToolResult { .. }
            | Self::ProtocolError { .. }
            | Self::RunTerminated { .. } => MessageOrigin::Runner,
            Self::RegisterDeployment { .. }
            | Self::TaskCreated { .. }
            | Self::TaskDelegated { .. }
            | Self::TaskStarted { .. }
            | Self::TaskCompleted { .. }
            | Self::TaskFailed { .. }
            | Self::TaskReassigned { .. }
            | Self::TaskCancelled { .. }
            | Self::ToolRequest { .. }
            | Self::EvidenceShared { .. }
            | Self::FindingProposed { .. }
            | Self::FinalSubmission { .. } => MessageOrigin::Deployment,
        }
    }
}

/// Compact aliases used by session state without exposing mutable collections.
pub(crate) type EvidenceIds = BTreeSet<EvidenceId>;
pub(crate) type FindingIds = BTreeSet<FindingId>;
