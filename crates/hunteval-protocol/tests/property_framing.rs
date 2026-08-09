use hunteval_protocol::JsonlDecoder;
use proptest::{
    collection::vec,
    prelude::*,
    test_runner::{Config, RngAlgorithm, TestRng, TestRunner},
};

#[test]
fn arbitrary_bounded_frames_are_deterministic_and_panic_free()
-> Result<(), Box<dyn std::error::Error>> {
    let mut runner = deterministic_runner(512);
    runner.run(&vec(any::<u8>(), 0..=2048), |bytes| {
        let decoder =
            JsonlDecoder::new(1024).map_err(|error| TestCaseError::fail(error.to_string()))?;
        let first = decoder.decode(&bytes);
        let second = decoder.decode(&bytes);
        prop_assert_eq!(first, second);
        Ok(())
    })?;
    Ok(())
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
