#![no_main]

use hunteval_protocol::{ProtocolEnvelope, TrajectoryRecorder, replay_trajectory};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 256 * 1024 {
        return;
    }
    let Ok(messages) = serde_json::from_slice::<Vec<ProtocolEnvelope>>(data) else {
        return;
    };
    let mut recorder = TrajectoryRecorder::new();
    for message in messages.into_iter().take(1024) {
        if recorder.append(message).is_err() {
            return;
        }
    }
    let _ = replay_trajectory(recorder.as_bytes(), 128 * 1024);
});
