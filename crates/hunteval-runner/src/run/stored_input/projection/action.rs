use hunteval_domain::{ActionId, EventId, MessageId};
use hunteval_evaluation::{ObservedAction, ObservedToolOutcome};
use hunteval_protocol::ToolOutcome;

use super::ReplayProjection;
use crate::run::StoredEvaluationError;

impl ReplayProjection {
    pub(super) fn complete_action(
        &mut self,
        action_id: ActionId,
        result_message_id: MessageId,
        tool: String,
        outcome: ToolOutcome,
        event_ids: std::collections::BTreeSet<EventId>,
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
                request_sequence: pending.request_sequence,
                caused_by_message_id: pending.caused_by_message_id,
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
