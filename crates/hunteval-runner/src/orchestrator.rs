use std::path::Path;

use hunteval_protocol::{
    MessageOrigin, ProtocolEnvelope, ProtocolError, ProtocolPayload, ProtocolSession,
    TrajectoryRecorder,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ArtifactError, ArtifactWriter, BudgetError, BudgetLedger, BudgetLimits};

/// Trusted settings for one mediated protocol session.
#[derive(Debug, Clone)]
pub struct RunConfig {
    pub budgets: BudgetLimits,
    pub maximum_trajectory_line_bytes: usize,
}

/// Terminal outcome retained even for an incomplete deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunTerminalStatus {
    Completed,
    Failed,
    BudgetExceeded,
    ProtocolViolation,
}

/// In-memory policy core for process adapters and managed tools.
#[derive(Debug)]
pub struct RunOrchestrator {
    session: ProtocolSession,
    trajectory: TrajectoryRecorder,
    budgets: BudgetLedger,
    maximum_line_bytes: usize,
}

impl RunOrchestrator {
    #[must_use]
    pub fn new(config: RunConfig) -> Self {
        Self {
            session: ProtocolSession::new(),
            trajectory: TrajectoryRecorder::new(),
            budgets: BudgetLedger::new(config.budgets),
            maximum_line_bytes: config.maximum_trajectory_line_bytes,
        }
    }

    pub fn accept_runner(&mut self, envelope: ProtocolEnvelope) -> Result<(), OrchestratorError> {
        self.accept(envelope, MessageOrigin::Runner)
    }

    pub fn accept_deployment(
        &mut self,
        envelope: ProtocolEnvelope,
    ) -> Result<(), OrchestratorError> {
        self.accept(envelope, MessageOrigin::Deployment)
    }

    pub fn finish(self, writer: ArtifactWriter) -> Result<std::path::PathBuf, OrchestratorError> {
        self.session.finish().map_err(OrchestratorError::Protocol)?;
        writer.append(Path::new("trajectory.jsonl"), self.trajectory.as_bytes())?;
        writer.finalize().map_err(OrchestratorError::Artifact)
    }

    pub fn preserve_partial(&self, writer: &ArtifactWriter) -> Result<(), OrchestratorError> {
        writer.append(Path::new("trajectory.jsonl"), self.trajectory.as_bytes())?;
        Ok(())
    }

    #[must_use]
    pub fn trajectory(&self) -> &[u8] {
        self.trajectory.as_bytes()
    }

    #[must_use]
    pub const fn usage(&self) -> crate::BudgetUsage {
        self.budgets.usage()
    }

    fn accept(
        &mut self,
        envelope: ProtocolEnvelope,
        expected: MessageOrigin,
    ) -> Result<(), OrchestratorError> {
        if envelope.payload.origin() != expected {
            return Err(OrchestratorError::WrongOrigin);
        }
        self.budgets.charge_message()?;
        if matches!(envelope.payload, ProtocolPayload::ToolRequest { .. }) {
            self.budgets.charge_tool_call()?;
        }
        self.session
            .accept(&envelope)
            .map_err(OrchestratorError::Protocol)?;
        let line = self
            .trajectory
            .append(envelope)
            .map_err(OrchestratorError::Protocol)?;
        if line.len() > self.maximum_line_bytes {
            return Err(OrchestratorError::TrajectoryLineTooLarge);
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("message was emitted by the wrong protocol peer")]
    WrongOrigin,
    #[error("trajectory event exceeds the configured line limit")]
    TrajectoryLineTooLarge,
    #[error(transparent)]
    Budget(#[from] BudgetError),
    #[error("protocol session failed: {0}")]
    Protocol(ProtocolError),
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
}
