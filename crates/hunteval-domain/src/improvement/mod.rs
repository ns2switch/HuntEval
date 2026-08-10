mod artifact;
mod experiment;
mod lifecycle;
mod recommendation;

pub use artifact::*;
pub use experiment::*;
pub use lifecycle::*;
pub use recommendation::*;

use thiserror::Error;

pub(crate) const MAX_ID_LEN: usize = 128;
pub(crate) const MAX_TEXT_LEN: usize = 4096;
pub(crate) const MAX_ITEMS: usize = 128;

pub(crate) fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_LEN
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'-'))
        })
}

pub(crate) fn valid_text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_TEXT_LEN && !value.contains('\0')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ImprovementContractError {
    #[error("schema version 0.8 is required")]
    UnsupportedVersion,
    #[error("controlled-improvement identifier or text is invalid")]
    InvalidValue,
    #[error("controlled-improvement collection is empty, duplicated, or outside its bound")]
    InvalidCollection,
    #[error("artifact content does not match its declared identity")]
    DigestMismatch,
    #[error("structured artifact violates section or mutability invariants")]
    InvalidStructure,
    #[error("controlled experiment violates its safety or equivalence contract")]
    InvalidExperiment,
    #[error("recommendation lifecycle transition is invalid")]
    InvalidTransition,
    #[error("human approval or external adoption confirmation is invalid")]
    InvalidApproval,
}

pub(crate) fn require_v08(version: crate::SchemaVersion) -> Result<(), ImprovementContractError> {
    if version == crate::SchemaVersion::new(0, 8) {
        Ok(())
    } else {
        Err(ImprovementContractError::UnsupportedVersion)
    }
}
