use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use hunteval_runner::{DeploymentProcess, ProcessError, ProcessSpec};

fn spec(script: &str) -> ProcessSpec {
    ProcessSpec {
        executable: PathBuf::from("/bin/sh"),
        arguments: vec!["-c".into(), script.into()],
        working_directory: PathBuf::from("/tmp"),
        environment: BTreeMap::new(),
        timeout: Duration::from_millis(100),
        output_limit_bytes: 64,
        redacted_values: vec!["hunter-secret".into()],
    }
}

#[test]
fn captures_bounded_output_and_redacts_stderr() -> Result<(), Box<dyn std::error::Error>> {
    let output = DeploymentProcess::run(&spec("printf 'ok'; printf 'hunter-secret' >&2"))?;
    assert_eq!(output.stdout, b"ok");
    assert_eq!(output.stderr, "[REDACTED]");
    Ok(())
}

#[test]
fn normalizes_nonzero_exit() {
    let error = DeploymentProcess::run(&spec("printf 'hunter-secret' >&2; exit 7"));
    assert!(matches!(error, Err(ProcessError::Exit { code: 7, stderr }) if stderr == "[REDACTED]"));
}

#[test]
fn terminates_timed_out_process() {
    let error = DeploymentProcess::run(&spec("sleep 2"));
    assert!(matches!(error, Err(ProcessError::Timeout)));
}
