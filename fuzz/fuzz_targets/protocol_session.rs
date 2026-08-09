#![no_main]

use hunteval_protocol::{ProtocolEnvelope, ProtocolSession};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 256 * 1024 {
        return;
    }
    let Ok(messages) = serde_json::from_slice::<Vec<ProtocolEnvelope>>(data) else {
        return;
    };
    let mut session = ProtocolSession::new();
    for message in messages.iter().take(1024) {
        if session.accept(message).is_err() {
            break;
        }
    }
    let _ = session.finish();
});
