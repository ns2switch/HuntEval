use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{AgentId, ContractValidationError, TaskId};

/// Scheduling priority declared for an observable task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Immutable task data supplied when the task is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSpec {
    pub id: TaskId,
    pub objective: String,
    pub priority: TaskPriority,
    #[serde(default)]
    pub dependencies: BTreeSet<TaskId>,
    #[serde(default)]
    pub required_capabilities: BTreeSet<String>,
    pub parent_task_id: Option<TaskId>,
}

/// Valid task lifecycle states reconstructed from trajectory events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Created,
    Delegated,
    Started,
    Completed,
    Failed,
    Reassigned,
    Cancelled,
}

/// Replayable task state with creator and current assignee attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskRecord {
    pub spec: TaskSpec,
    pub creator: AgentId,
    pub assignee: Option<AgentId>,
    pub state: TaskState,
}

impl TaskRecord {
    /// Creates a task in the only valid initial state.
    pub fn create(spec: TaskSpec, creator: AgentId) -> Result<Self, ContractValidationError> {
        if spec.objective.trim().is_empty() {
            return Err(ContractValidationError::new(
                "task.objective",
                "objective must not be empty",
            ));
        }
        if spec.dependencies.contains(&spec.id) || spec.parent_task_id.as_ref() == Some(&spec.id) {
            return Err(ContractValidationError::new(
                "task.dependencies",
                "task cannot depend on or parent itself",
            ));
        }
        Ok(Self {
            spec,
            creator,
            assignee: None,
            state: TaskState::Created,
        })
    }

    /// Delegates a newly created task.
    pub fn delegate(&mut self, assignee: AgentId) -> Result<(), ContractValidationError> {
        self.require_state(TaskState::Created)?;
        self.assignee = Some(assignee);
        self.state = TaskState::Delegated;
        Ok(())
    }

    /// Marks a delegated or reassigned task as started.
    pub fn start(&mut self, agent: &AgentId) -> Result<(), ContractValidationError> {
        if !matches!(self.state, TaskState::Delegated | TaskState::Reassigned)
            || self.assignee.as_ref() != Some(agent)
        {
            return Err(invalid_transition());
        }
        self.state = TaskState::Started;
        Ok(())
    }

    /// Marks a started task as completed.
    pub fn complete(&mut self, agent: &AgentId) -> Result<(), ContractValidationError> {
        self.require_assigned_started(agent)?;
        self.state = TaskState::Completed;
        Ok(())
    }

    /// Marks a started task as failed so it can be reassigned.
    pub fn fail(&mut self, agent: &AgentId) -> Result<(), ContractValidationError> {
        self.require_assigned_started(agent)?;
        self.state = TaskState::Failed;
        Ok(())
    }

    /// Assigns a failed task to another registered agent.
    pub fn reassign(&mut self, assignee: AgentId) -> Result<(), ContractValidationError> {
        self.require_state(TaskState::Failed)?;
        self.assignee = Some(assignee);
        self.state = TaskState::Reassigned;
        Ok(())
    }

    /// Cancels a task that has not reached a terminal success state.
    pub fn cancel(&mut self) -> Result<(), ContractValidationError> {
        if matches!(self.state, TaskState::Completed | TaskState::Cancelled) {
            return Err(invalid_transition());
        }
        self.state = TaskState::Cancelled;
        Ok(())
    }

    fn require_assigned_started(&self, agent: &AgentId) -> Result<(), ContractValidationError> {
        if self.state != TaskState::Started || self.assignee.as_ref() != Some(agent) {
            return Err(invalid_transition());
        }
        Ok(())
    }

    fn require_state(&self, expected: TaskState) -> Result<(), ContractValidationError> {
        if self.state != expected {
            return Err(invalid_transition());
        }
        Ok(())
    }
}

fn invalid_transition() -> ContractValidationError {
    ContractValidationError::new("task.state", "invalid task state transition or owner")
}
