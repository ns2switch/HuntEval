use thiserror::Error;

use crate::{DigestParseError, IdValidationError, TimestampError, VersionParseError};

/// A stable validation failure for a versioned authored or normalized contract.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid field '{field}': {reason}")]
pub struct ContractValidationError {
    field: &'static str,
    reason: &'static str,
}

impl ContractValidationError {
    /// Creates a safe validation error without embedding untrusted values.
    #[must_use]
    pub const fn new(field: &'static str, reason: &'static str) -> Self {
        Self { field, reason }
    }

    /// Returns the stable field path associated with the failure.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a safe reason suitable for logs and protocol errors.
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

/// Top-level typed error for infrastructure-independent domain validation.
#[derive(Debug, Error)]
pub enum DomainError {
    /// An opaque identifier failed validation.
    #[error(transparent)]
    Identifier(#[from] IdValidationError),
    /// A timestamp was malformed or did not use UTC.
    #[error(transparent)]
    Timestamp(#[from] TimestampError),
    /// A schema or protocol version was malformed.
    #[error(transparent)]
    Version(#[from] VersionParseError),
    /// An artifact digest was malformed.
    #[error(transparent)]
    Digest(#[from] DigestParseError),
    /// A versioned contract violated a domain invariant.
    #[error(transparent)]
    Contract(#[from] ContractValidationError),
}
