use std::{fs, io, path::PathBuf};

use hunteval_protocol::{ProtocolEnvelope, TrajectoryRecorder, replay_trajectory};
use proptest::{
    prelude::*,
    test_runner::{Config, RngAlgorithm, TestRng, TestRunner},
};

#[test]
fn every_nonempty_trajectory_truncation_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let mut recorder = TrajectoryRecorder::new();
    for message in canonical_messages()? {
        recorder.append(message)?;
    }
    let bytes = recorder.as_bytes();
    let mut runner = deterministic_runner(512);
    runner.run(&(1_usize..bytes.len()), |length| {
        prop_assert!(replay_trajectory(&bytes[..length], 128 * 1024).is_err());
        Ok(())
    })?;
    Ok(())
}

fn canonical_messages() -> Result<Vec<ProtocolEnvelope>, Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))?
        .to_path_buf();
    Ok(serde_json::from_slice(&fs::read(
        root.join("examples/contracts/protocol-transcript.json"),
    )?)?)
}

fn deterministic_runner(cases: u32) -> TestRunner {
    TestRunner::new_with_rng(
        Config {
            cases,
            max_shrink_iters: 4096,
            ..Config::default()
        },
        TestRng::deterministic_rng(RngAlgorithm::ChaCha),
    )
}
