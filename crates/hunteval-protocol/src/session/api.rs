use std::collections::{BTreeMap, BTreeSet};

use crate::{ProtocolEnvelope, ProtocolErrorCode};

use super::{ProtocolError, ProtocolPhase, ProtocolSession, error};

impl Default for ProtocolSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolSession {
    /// Creates an empty session that accepts only `run_started`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            phase: ProtocolPhase::New,
            run_id: None,
            supported_minimum: None,
            supported_maximum: None,
            selected_version: None,
            max_agents: None,
            registration_message_id: None,
            messages: BTreeSet::new(),
            agents: BTreeSet::new(),
            tasks: BTreeMap::new(),
            actions: BTreeMap::new(),
            evidence: BTreeSet::new(),
            findings: BTreeSet::new(),
        }
    }

    /// Returns the current validated phase.
    #[must_use]
    pub const fn phase(&self) -> ProtocolPhase {
        self.phase
    }

    /// Returns the number of reconstructed tasks.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Validates and applies one message atomically to replay state.
    pub fn accept(&mut self, message: &ProtocolEnvelope) -> Result<(), ProtocolError> {
        if self.messages.contains(&message.message_id) {
            return Err(error(
                ProtocolErrorCode::DuplicateIdentifier,
                "duplicate message identifier",
            ));
        }
        if let Some(run_id) = &self.run_id
            && run_id != &message.run_id
        {
            return Err(error(
                ProtocolErrorCode::InvalidMessage,
                "message belongs to another run",
            ));
        }

        self.apply(message)?;
        self.messages.insert(message.message_id.clone());
        Ok(())
    }

    /// Requires a normal terminal event after a complete transcript.
    pub fn finish(&self) -> Result<(), ProtocolError> {
        if self.phase != ProtocolPhase::Terminated {
            return Err(error(
                ProtocolErrorCode::ProcessFailure,
                "session ended before termination",
            ));
        }
        Ok(())
    }
}
