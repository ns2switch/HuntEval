//! Deterministic external reference deployment speaking HuntEval JSONL.

mod framing;
mod hunt;
mod peer;
mod topology;

use std::io;

use clap::Parser;
use hunteval_domain::{ContractValidationError, IdValidationError};
use hunteval_protocol::ProtocolError;
use thiserror::Error;

pub use topology::ReferenceTopology;

/// Command-line options for the reference protocol peer.
#[derive(Debug, Parser)]
pub struct ReferenceOptions {
    #[arg(long, value_enum)]
    pub topology: ReferenceTopology,
}

/// Runs the selected reference topology over bounded standard-input/output JSONL.
pub fn run_stdio(topology: ReferenceTopology) -> Result<(), ReferenceError> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    peer::run_peer(topology, stdin.lock(), stdout.lock())
}

/// Safe typed failures emitted by the reference deployment process.
#[derive(Debug, Error)]
pub enum ReferenceError {
    #[error("protocol input ended before the session completed")]
    EarlyEof,
    #[error("protocol input line exceeds the configured bound")]
    OversizedLine,
    #[error("protocol peer received an invalid runner message")]
    InvalidRunnerMessage,
    #[error("protocol peer received an unusable managed-tool result")]
    InvalidToolResult,
    #[error("protocol state validation failed: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("domain contract validation failed: {0}")]
    Contract(#[from] ContractValidationError),
    #[error("identifier validation failed: {0}")]
    Identifier(#[from] IdValidationError),
    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("protocol I/O failed: {0}")]
    Io(#[from] io::Error),
}
