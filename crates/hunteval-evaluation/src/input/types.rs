use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use hunteval_domain::{
    ActionId, AgentId, EpisodeId, EventId, Evidence, EvidenceId, FinalSubmission, Finding,
    FindingId, GroundTruth, MessageId, RunId, Sha256Digest, TaskId, TaskSpec, TaskState,
    TimelineEntry,
};
use serde::Serialize;

use crate::{EvaluationInput, input::validate::validate_input};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedToolOutcome {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedAction {
    pub action_id: ActionId,
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub request_message_id: MessageId,
    pub result_message_id: MessageId,
    pub tool: String,
    pub purpose: String,
    pub arguments: serde_json::Value,
    pub outcome: ObservedToolOutcome,
    pub event_ids: BTreeSet<EventId>,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedTask {
    pub task: TaskSpec,
    pub creator_agent_id: AgentId,
    pub assignee_agent_id: Option<AgentId>,
    pub state: TaskState,
    pub created_message_id: MessageId,
    pub terminal_message_id: Option<MessageId>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedEvidence {
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub message_id: MessageId,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedFinding {
    pub agent_id: AgentId,
    pub task_id: TaskId,
    pub message_id: MessageId,
    pub finding: Finding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedMessage {
    pub message_id: MessageId,
    pub caused_by_message_id: Option<MessageId>,
    pub agent_id: AgentId,
    pub target_agent_id: AgentId,
    pub task_id: Option<TaskId>,
    pub reason_code: String,
    pub message: String,
}

pub type SubmittedTimelineEntry = TimelineEntry;

/// Deployment-safe replay projection. It deliberately has no ground-truth field.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedRun {
    pub run_id: RunId,
    pub episode_id: EpisodeId,
    pub actions: BTreeMap<ActionId, ObservedAction>,
    pub tasks: BTreeMap<TaskId, ObservedTask>,
    pub evidence: BTreeMap<EvidenceId, ObservedEvidence>,
    pub findings: BTreeMap<FindingId, ObservedFinding>,
    pub messages: Vec<ObservedMessage>,
    pub timeline: Option<Vec<SubmittedTimelineEntry>>,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationProvenance {
    pub run_id: RunId,
    pub trajectory_sha256: Sha256Digest,
    pub submission_sha256: Sha256Digest,
    pub ground_truth_sha256: Sha256Digest,
    pub trajectory_event_count: u64,
}

impl fmt::Debug for EvaluationProvenance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EvaluationProvenance")
            .field("run_id", &self.run_id)
            .field("trajectory_sha256", &self.trajectory_sha256)
            .field("submission_sha256", &self.submission_sha256)
            .field("ground_truth_sha256", &"<redacted>")
            .field("trajectory_event_count", &self.trajectory_event_count)
            .finish()
    }
}

/// Inputs accepted at the trusted evaluation boundary after protocol replay.
pub struct TrustedRunInput {
    pub observed: ObservedRun,
    pub submission: FinalSubmission,
    pub terminal_submission: FinalSubmission,
    pub ground_truth: GroundTruth,
    pub provenance: EvaluationProvenance,
    pub tool_call_limit: u64,
    pub benign_scored_episode: bool,
}

/// Validated evaluator view. This type is intentionally not serializable.
pub struct TrustedRunView {
    observed: ObservedRun,
    submission: FinalSubmission,
    ground_truth: GroundTruth,
    provenance: EvaluationProvenance,
    tool_call_limit: u64,
    benign_scored_episode: bool,
}

impl fmt::Debug for TrustedRunInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TrustedRunInput")
            .field("observed", &self.observed)
            .field("submission", &self.submission)
            .field("terminal_submission", &self.terminal_submission)
            .field("ground_truth", &"<redacted>")
            .field("provenance", &self.provenance)
            .field("tool_call_limit", &self.tool_call_limit)
            .field("benign_scored_episode", &self.benign_scored_episode)
            .finish()
    }
}

impl fmt::Debug for TrustedRunView {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TrustedRunView")
            .field("observed", &self.observed)
            .field("submission", &self.submission)
            .field("ground_truth", &"<redacted>")
            .field("provenance", &self.provenance)
            .field("tool_call_limit", &self.tool_call_limit)
            .field("benign_scored_episode", &self.benign_scored_episode)
            .finish()
    }
}

impl TrustedRunView {
    pub fn reduce(input: TrustedRunInput) -> Result<Self, super::TrustedViewError> {
        validate_input(&input)?;
        Ok(Self {
            observed: input.observed,
            submission: input.submission,
            ground_truth: input.ground_truth,
            provenance: input.provenance,
            tool_call_limit: input.tool_call_limit,
            benign_scored_episode: input.benign_scored_episode,
        })
    }

    #[must_use]
    pub const fn observed(&self) -> &ObservedRun {
        &self.observed
    }

    #[must_use]
    pub const fn submission(&self) -> &FinalSubmission {
        &self.submission
    }

    #[must_use]
    pub const fn provenance(&self) -> &EvaluationProvenance {
        &self.provenance
    }

    #[must_use]
    pub fn evaluation_input(&self) -> EvaluationInput {
        let completed_tasks = self
            .observed
            .tasks
            .values()
            .filter(|task| task.state == TaskState::Completed)
            .count() as u64;
        let provenance_references = self
            .observed
            .evidence
            .values()
            .map(|item| item.evidence.source_action_ids.len() as u64)
            .sum();
        EvaluationInput {
            truth_events: self.ground_truth.malicious_event_ids.clone(),
            submitted_events: self.submission.malicious_event_ids.clone(),
            truth_entities: self.ground_truth.malicious_entity_ids.clone(),
            submitted_entities: self.submission.malicious_entity_ids.clone(),
            expected_attack_path: self.ground_truth.expected_attack_path.clone(),
            submitted_attack_path: self.submission.attack_path.clone(),
            expected_timeline_windows: self.ground_truth.expected_timeline_windows.clone(),
            submitted_timeline: self.submission.timeline.clone(),
            acceptable_submission_statuses: self
                .ground_truth
                .acceptable_submission_statuses
                .clone(),
            submitted_status: self.submission.status,
            expected_attack_techniques: self.ground_truth.expected_attack_techniques.clone(),
            submitted_attack_techniques: self.submission.attack_techniques.clone(),
            benign_scored_episode: self.benign_scored_episode,
            evidence_items: self.observed.evidence.len() as u64,
            grounded_evidence_items: self.observed.evidence.len() as u64,
            findings_submitted: self.observed.findings.len() as u64,
            provenance_references,
            valid_provenance_references: provenance_references,
            tasks_created: self.observed.tasks.len() as u64,
            tasks_completed: completed_tasks,
            tool_calls_used: self.observed.actions.len() as u64,
            tool_call_limit: self.tool_call_limit,
        }
    }
}
