use std::io::{BufRead, Write};

use hunteval_protocol::{JsonlDecoder, ProtocolEnvelope};

use crate::ReferenceError;

pub(super) const MAX_PROTOCOL_LINE_BYTES: usize = 128 * 1024;

pub(super) fn read_message<R: BufRead>(
    reader: &mut R,
    decoder: JsonlDecoder,
) -> Result<ProtocolEnvelope, ReferenceError> {
    let mut line = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(ReferenceError::EarlyEof);
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |position| position + 1);
        if line.len().saturating_add(count) > MAX_PROTOCOL_LINE_BYTES {
            return Err(ReferenceError::OversizedLine);
        }
        line.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return decoder.decode(&line).map_err(ReferenceError::from);
        }
    }
}

pub(super) fn write_message<W: Write>(
    writer: &mut W,
    message: &ProtocolEnvelope,
) -> Result<(), ReferenceError> {
    let bytes = serde_json::to_vec(message)?;
    if bytes.len().saturating_add(1) > MAX_PROTOCOL_LINE_BYTES {
        return Err(ReferenceError::OversizedLine);
    }
    writer.write_all(&bytes)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
