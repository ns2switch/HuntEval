use hunteval_domain::{SchemaVersion, Sha256Digest};
use serde::{Deserialize, Serialize};

use crate::{JsonlDecoder, ProtocolEnvelope, ProtocolError, ProtocolErrorCode, ProtocolSession};

/// One runner-authored append-only trajectory record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEvent {
    pub schema_version: SchemaVersion,
    pub sequence: u64,
    pub previous_event_sha256: Option<Sha256Digest>,
    pub envelope: ProtocolEnvelope,
}

/// Exact-byte trajectory writer with a SHA-256 predecessor chain.
#[derive(Debug, Default)]
pub struct TrajectoryRecorder {
    bytes: Vec<u8>,
    previous_line: Option<Vec<u8>>,
    next_sequence: u64,
}

impl TrajectoryRecorder {
    /// Creates an empty recorder whose first sequence number is one.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            previous_line: None,
            next_sequence: 1,
        }
    }

    /// Appends one compact JSON event and returns its exact line bytes.
    pub fn append(&mut self, envelope: ProtocolEnvelope) -> Result<&[u8], ProtocolError> {
        let previous_event_sha256 = self.previous_line.as_ref().map(Sha256Digest::from_bytes);
        let event = StoredEvent {
            schema_version: SchemaVersion::new(0, 3),
            sequence: self.next_sequence,
            previous_event_sha256,
            envelope,
        };
        let mut line = serde_json::to_vec(&event).map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "trajectory event could not be serialized",
            )
        })?;
        line.push(b'\n');
        self.next_sequence = self.next_sequence.checked_add(1).ok_or_else(|| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidState,
                "trajectory sequence overflow",
            )
        })?;
        self.bytes.extend_from_slice(&line);
        self.previous_line = Some(line);
        Ok(self.previous_line.as_deref().unwrap_or_default())
    }

    /// Returns the complete exact JSONL trajectory.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the digest of the complete trajectory file.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        Sha256Digest::from_bytes(&self.bytes)
    }
}

/// Deterministic output of replaying a complete trajectory.
#[derive(Debug)]
pub struct ReplayOutcome {
    pub session: ProtocolSession,
    pub event_count: u64,
    pub trajectory_sha256: Sha256Digest,
}

/// Validates exact-byte hash links and reconstructs protocol state without I/O.
pub fn replay_trajectory(
    bytes: &[u8],
    max_line_bytes: usize,
) -> Result<ReplayOutcome, ProtocolError> {
    let _decoder = JsonlDecoder::new(max_line_bytes)?;
    let mut session = ProtocolSession::new();
    let mut previous_line: Option<&[u8]> = None;
    let mut expected_sequence = 1_u64;

    for line in bytes.split_inclusive(|byte| *byte == b'\n') {
        if !line.ends_with(b"\n") {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "trajectory contains a partial final line",
            ));
        }
        if line.len() > max_line_bytes {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "trajectory line exceeds configured limit",
            ));
        }
        let event: StoredEvent = serde_json::from_slice(&line[..line.len() - 1]).map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "trajectory line is not a valid stored event",
            )
        })?;
        if event.sequence != expected_sequence {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidState,
                "trajectory sequence is not contiguous",
            ));
        }
        let expected_hash = previous_line.map(Sha256Digest::from_bytes);
        if event.previous_event_sha256 != expected_hash {
            return Err(ProtocolError::new(
                ProtocolErrorCode::ProvenanceViolation,
                "trajectory predecessor hash mismatch",
            ));
        }
        session.accept(&event.envelope)?;
        previous_line = Some(line);
        expected_sequence = expected_sequence.checked_add(1).ok_or_else(|| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidState,
                "trajectory sequence overflow",
            )
        })?;
    }

    session.finish()?;
    Ok(ReplayOutcome {
        session,
        event_count: expected_sequence - 1,
        trajectory_sha256: Sha256Digest::from_bytes(bytes),
    })
}
