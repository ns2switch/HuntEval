use thiserror::Error;

use crate::{DigestParseError, IdValidationError, TimestampError, VersionParseError};

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
}
