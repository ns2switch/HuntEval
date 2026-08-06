use crate::{ProtocolEnvelope, ProtocolError, ProtocolErrorCode};

/// Bounded decoder for one UTF-8 JSON object terminated by a newline.
#[derive(Debug, Clone, Copy)]
pub struct JsonlDecoder {
    max_line_bytes: usize,
}

impl JsonlDecoder {
    /// Creates a decoder with a nonzero inclusive byte limit.
    pub fn new(max_line_bytes: usize) -> Result<Self, ProtocolError> {
        if max_line_bytes == 0 {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "JSONL line limit must be positive",
            ));
        }
        Ok(Self { max_line_bytes })
    }

    /// Decodes exactly one newline-terminated message.
    pub fn decode(&self, line: &[u8]) -> Result<ProtocolEnvelope, ProtocolError> {
        if line.len() > self.max_line_bytes {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "JSONL line exceeds configured limit",
            ));
        }
        if !line.ends_with(b"\n") || line[..line.len().saturating_sub(1)].contains(&b'\n') {
            return Err(ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "input must contain exactly one newline-terminated JSON value",
            ));
        }
        let json = std::str::from_utf8(&line[..line.len() - 1]).map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "JSONL input must be valid UTF-8",
            )
        })?;
        serde_json::from_str(json).map_err(|_| {
            ProtocolError::new(
                ProtocolErrorCode::InvalidMessage,
                "JSONL input must be one valid protocol object",
            )
        })
    }
}
