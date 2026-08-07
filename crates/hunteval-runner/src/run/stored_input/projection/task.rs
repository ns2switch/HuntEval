use hunteval_domain::{AgentId, MessageId, TaskId, TaskSpec, TaskState};
use hunteval_evaluation::{ObservedTask, ObservedTaskTransition};

use super::ReplayProjection;
use crate::run::StoredEvaluationError;

pub(super) struct ProjectedTaskTransition {
    pub(super) message_id: MessageId,
    pub(super) sequence: u64,
    pub(super) caused_by_message_id: Option<MessageId>,
    pub(super) agent_id: AgentId,
    pub(super) task_id: TaskId,
    pub(super) state: TaskState,
    pub(super) assignee_agent_id: Option<AgentId>,
    pub(super) terminal: bool,
}

impl ReplayProjection {
    pub(super) fn create_task(
        &mut self,
        message_id: MessageId,
        sequence: u64,
        caused_by_message_id: Option<MessageId>,
        agent_id: AgentId,
        task: TaskSpec,
    ) {
        self.task_transitions.push(ObservedTaskTransition {
            message_id: message_id.clone(),
            sequence,
            caused_by_message_id,
            agent_id: agent_id.clone(),
            task_id: task.id.clone(),
            state: TaskState::Created,
        });
        self.tasks.insert(
            task.id.clone(),
            ObservedTask {
                task,
                creator_agent_id: agent_id,
                assignee_agent_id: None,
                state: TaskState::Created,
                created_message_id: message_id,
                terminal_message_id: None,
            },
        );
    }

    pub(super) fn transition_task(
        &mut self,
        transition: ProjectedTaskTransition,
    ) -> Result<(), StoredEvaluationError> {
        let task = self
            .tasks
            .get_mut(&transition.task_id)
            .ok_or(StoredEvaluationError::InvalidProjection)?;
        if let Some(assignee) = &transition.assignee_agent_id {
            task.assignee_agent_id = Some(assignee.clone());
        }
        task.state = transition.state;
        task.terminal_message_id = transition.terminal.then(|| transition.message_id.clone());
        self.task_transitions.push(ObservedTaskTransition {
            message_id: transition.message_id,
            sequence: transition.sequence,
            caused_by_message_id: transition.caused_by_message_id,
            agent_id: transition.agent_id,
            task_id: transition.task_id,
            state: transition.state,
        });
        Ok(())
    }
}
