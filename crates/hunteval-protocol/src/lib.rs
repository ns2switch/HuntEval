//! Bounded JSONL protocol sessions and deterministic trajectory replay.

mod framing;
mod message;
mod replay;
mod session;

pub use framing::JsonlDecoder;
pub use message::{
    MessageOrigin, ProtocolEnvelope, ProtocolErrorCode, ProtocolPayload, ToolOutcome,
};
pub use replay::{ReplayOutcome, StoredEvent, TrajectoryRecorder, replay_trajectory};
pub use session::{ProtocolError, ProtocolPhase, ProtocolSession};
