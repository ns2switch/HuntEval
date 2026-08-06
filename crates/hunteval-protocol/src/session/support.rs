use std::collections::BTreeSet;

use hunteval_domain::{AgentId, ProtocolVersion, TaskId, TaskRecord};

use crate::{
    ProtocolErrorCode,
    session::{ProtocolError, ProtocolPhase, ProtocolSession, error},
};

impl ProtocolSession {
    pub(super) fn validate_provenance(
        &self,
        evidence: &hunteval_domain::Evidence,
    ) -> Result<(), ProtocolError> {
        let mut issued_events = BTreeSet::new();
        for action_id in &evidence.source_action_ids {
            let events = self
                .actions
                .get(action_id)
                .and_then(|action| action.event_ids.as_ref())
                .ok_or_else(|| {
                    error(
                        ProtocolErrorCode::ProvenanceViolation,
                        "evidence references action without result",
                    )
                })?;
            issued_events.extend(events.iter().cloned());
        }
        if !evidence.event_ids.is_subset(&issued_events) {
            return Err(error(
                ProtocolErrorCode::ProvenanceViolation,
                "evidence references unissued event",
            ));
        }
        Ok(())
    }

    pub(super) fn require_supported(
        &self,
        selected: ProtocolVersion,
        envelope: ProtocolVersion,
    ) -> Result<(), ProtocolError> {
        let supported = self
            .supported_minimum
            .is_some_and(|minimum| selected >= minimum)
            && self
                .supported_maximum
                .is_some_and(|maximum| selected <= maximum)
            && selected == envelope;
        if !supported {
            return Err(error(
                ProtocolErrorCode::UnsupportedProtocolVersion,
                "unsupported protocol version",
            ));
        }
        Ok(())
    }

    pub(super) fn require_active_agent(&self, agent_id: &AgentId) -> Result<(), ProtocolError> {
        self.require_phase(ProtocolPhase::Active)?;
        self.require_agent(agent_id)
    }

    pub(super) fn require_agent(&self, agent_id: &AgentId) -> Result<(), ProtocolError> {
        if !self.agents.contains(agent_id) {
            return Err(error(
                ProtocolErrorCode::UnknownAgent,
                "unknown agent identifier",
            ));
        }
        Ok(())
    }

    pub(super) fn require_task(&self, task_id: &TaskId) -> Result<&TaskRecord, ProtocolError> {
        self.tasks
            .get(task_id)
            .ok_or_else(|| error(ProtocolErrorCode::UnknownTask, "unknown task identifier"))
    }

    pub(super) fn task_mut(&mut self, task_id: &TaskId) -> Result<&mut TaskRecord, ProtocolError> {
        self.tasks
            .get_mut(task_id)
            .ok_or_else(|| error(ProtocolErrorCode::UnknownTask, "unknown task identifier"))
    }

    pub(super) fn require_phase(&self, expected: ProtocolPhase) -> Result<(), ProtocolError> {
        if self.phase != expected {
            return Err(error(
                ProtocolErrorCode::InvalidState,
                "message is invalid in the current phase",
            ));
        }
        Ok(())
    }

    pub(super) fn require_not_new(&self) -> Result<(), ProtocolError> {
        if self.phase == ProtocolPhase::New {
            return Err(error(
                ProtocolErrorCode::InvalidState,
                "session has not started",
            ));
        }
        Ok(())
    }
}
