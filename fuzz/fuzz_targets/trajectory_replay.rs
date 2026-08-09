#![no_main]

use hunteval_protocol::replay_trajectory;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() <= 1024 * 1024 {
        let _ = replay_trajectory(data, 128 * 1024);
    }
});
