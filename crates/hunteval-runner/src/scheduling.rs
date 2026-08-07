use hunteval_domain::{AgentId, TaskId};
use serde::{Deserialize, Serialize};

/// One logical scheduling decision independent of wall-clock timing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledTask {
    pub ready_sequence: u64,
    pub agent_id: AgentId,
    pub task_id: TaskId,
}

/// Returns a stable schedule for replay and paired tests.
#[must_use]
pub fn deterministic_schedule(mut tasks: Vec<ScheduledTask>) -> Vec<ScheduledTask> {
    tasks.sort_by(|left, right| {
        left.ready_sequence
            .cmp(&right.ready_sequence)
            .then_with(|| left.agent_id.cmp(&right.agent_id))
            .then_with(|| left.task_id.cmp(&right.task_id))
    });
    tasks
}
