use thiserror::Error;

use crate::ProtocolErrorCode;

/// Typed protocol validation failure with a stable public code.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct ProtocolError {
    pub code: ProtocolErrorCode,
    message: &'static str,
}

impl ProtocolError {
    /// Creates a safe protocol failure without embedding untrusted input.
    #[must_use]
    pub const fn new(code: ProtocolErrorCode, message: &'static str) -> Self {
        Self { code, message }
    }

    /// Returns the stable safe message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

pub(super) fn error(code: ProtocolErrorCode, message: &'static str) -> ProtocolError {
    ProtocolError::new(code, message)
}

pub(super) fn contract_error(_: hunteval_domain::ContractValidationError) -> ProtocolError {
    error(
        ProtocolErrorCode::InvalidMessage,
        "domain contract validation failed",
    )
}
