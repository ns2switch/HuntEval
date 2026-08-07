use hunteval_domain::{MessageId, RunId};

use super::{transport::TransportError, types::RunFailureKind};

pub(super) struct RunnerMessageIds {
    prefix: String,
    next: u64,
}

impl RunnerMessageIds {
    pub(super) fn new(run_id: &RunId) -> Self {
        Self {
            prefix: run_id.as_str().to_owned(),
            next: 1,
        }
    }

    pub(super) fn next(&mut self) -> Result<MessageId, EngineError> {
        let id = MessageId::new(format!("runner-{}-{:04}", self.prefix, self.next))
            .map_err(|_| EngineError::InvalidConfiguration)?;
        self.next = self
            .next
            .checked_add(1)
            .ok_or(EngineError::InvalidConfiguration)?;
        Ok(id)
    }
}

#[derive(Debug)]
pub(super) enum EngineError {
    Artifact,
    Budget,
    InvalidConfiguration,
    Evaluation,
    Process,
    ProtocolViolation,
    Timeout,
}

impl EngineError {
    pub(super) const fn kind(&self) -> RunFailureKind {
        match self {
            Self::Artifact => RunFailureKind::Artifact,
            Self::Budget => RunFailureKind::BudgetExceeded,
            Self::InvalidConfiguration => RunFailureKind::InvalidConfiguration,
            Self::Evaluation => RunFailureKind::Evaluation,
            Self::Process => RunFailureKind::ProcessCrash,
            Self::ProtocolViolation => RunFailureKind::ProtocolViolation,
            Self::Timeout => RunFailureKind::Timeout,
        }
    }
}

impl From<hunteval_evaluation::EvaluationError> for EngineError {
    fn from(_: hunteval_evaluation::EvaluationError) -> Self {
        Self::Evaluation
    }
}

impl From<hunteval_evaluation::ProfileError> for EngineError {
    fn from(_: hunteval_evaluation::ProfileError) -> Self {
        Self::Evaluation
    }
}

impl From<crate::PolicyError> for EngineError {
    fn from(_: crate::PolicyError) -> Self {
        Self::InvalidConfiguration
    }
}

impl From<crate::ArtifactError> for EngineError {
    fn from(_: crate::ArtifactError) -> Self {
        Self::Artifact
    }
}

impl From<crate::OrchestratorError> for EngineError {
    fn from(error: crate::OrchestratorError) -> Self {
        if matches!(error, crate::OrchestratorError::Budget(_)) {
            Self::Budget
        } else {
            Self::ProtocolViolation
        }
    }
}

impl From<hunteval_protocol::ProtocolError> for EngineError {
    fn from(_: hunteval_protocol::ProtocolError) -> Self {
        Self::ProtocolViolation
    }
}

impl From<TransportError> for EngineError {
    fn from(error: TransportError) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_process_failure() {
            Self::Process
        } else {
            Self::ProtocolViolation
        }
    }
}
