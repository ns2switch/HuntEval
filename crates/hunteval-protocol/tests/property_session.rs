use std::{fs, io, path::PathBuf};

use hunteval_protocol::{ProtocolEnvelope, ProtocolErrorCode, ProtocolSession};
use proptest::{
    prelude::*,
    test_runner::{Config, RngAlgorithm, TestRng, TestRunner},
};

#[test]
fn accepted_prefixes_reject_every_duplicate_identifier() -> Result<(), Box<dyn std::error::Error>> {
    let messages = canonical_messages()?;
    let length = messages.len();
    let mut runner = deterministic_runner(256);
    runner.run(
        &(1_usize..=length).prop_flat_map(|prefix| (Just(prefix), 0..prefix)),
        |(prefix, duplicate)| {
            let mut session = ProtocolSession::new();
            for message in messages.iter().take(prefix) {
                session
                    .accept(message)
                    .map_err(|error| TestCaseError::fail(error.to_string()))?;
            }
            let error = session.accept(&messages[duplicate]).err();
            prop_assert_eq!(
                error.map(|value| value.code),
                Some(ProtocolErrorCode::DuplicateIdentifier)
            );
            Ok(())
        },
    )?;
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
