use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Stdio},
};

use hunteval_commercial::{
    BearerSecret, CommercialMode, CommercialOperation, CommercialPlatform, CommercialPolicy,
    CommercialRequest, CommercialWorkerCommand, CommercialWorkerResponse, GatewayRequest,
    SecretReference, execute_worker_command,
};

fn command() -> CommercialWorkerCommand {
    CommercialWorkerCommand {
        policy: CommercialPolicy {
            policy_version: "0.1".to_owned(),
            mode: CommercialMode::FixtureReplay,
            platform: CommercialPlatform::CrowdstrikeFalcon,
            origin: "https://api.crowdstrike.example".to_owned(),
            operations: vec![CommercialOperation::DetectionsSearch],
            secret_reference: None,
            max_requests: 1,
            max_response_bytes: 4_096,
            max_records: 10,
            timeout_ms: 1_000,
        },
        target: BTreeMap::new(),
        request: GatewayRequest {
            request_id: "request-1".to_owned(),
            agent_id: "agent-1".to_owned(),
            task_id: "task-1".to_owned(),
            action_id: "action-1".to_owned(),
            request: CommercialRequest {
                platform: CommercialPlatform::CrowdstrikeFalcon,
                operation: CommercialOperation::DetectionsSearch,
                tenant_alias: "tenant-test".to_owned(),
                region: "region-test".to_owned(),
                arguments: BTreeMap::from([("limit".to_owned(), 1.into())]),
            },
        },
    }
}

#[test]
fn live_worker_rejects_fixture_mode_before_network_or_secret_use() {
    let secret = BearerSecret::new("canary-value-that-must-not-leak".to_owned())
        .unwrap_or_else(|error| unreachable!("valid secret fixture: {error}"));
    assert_eq!(
        execute_worker_command(command(), secret),
        CommercialWorkerResponse::Failure {
            reason_code: "invalid_policy".to_owned()
        }
    );
}

#[test]
fn live_worker_rejects_platform_mismatch_before_resolution() {
    let mut value = command();
    value.policy.mode = CommercialMode::LiveReadOnly;
    value.policy.secret_reference = Some(
        SecretReference::try_from("fixture-read-only".to_owned())
            .unwrap_or_else(|error| unreachable!("valid reference: {error}")),
    );
    value.request.request.platform = CommercialPlatform::GoogleSecops;
    let secret = BearerSecret::new("canary-value-that-must-not-leak".to_owned())
        .unwrap_or_else(|error| unreachable!("valid secret fixture: {error}"));
    assert_eq!(
        execute_worker_command(value, secret),
        CommercialWorkerResponse::Failure {
            reason_code: "invalid_policy".to_owned()
        }
    );
}

#[test]
fn worker_process_uses_separate_framing_and_never_echoes_the_secret() {
    let command = serde_json::to_vec(&command())
        .unwrap_or_else(|error| unreachable!("serializable command fixture: {error}"));
    let secret = b"canary-value-that-must-not-leak";
    let mut child = Command::new(env!("CARGO_BIN_EXE_hunteval-commercial-worker"))
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| unreachable!("worker fixture starts: {error}"));
    let mut stdin = child
        .stdin
        .take()
        .unwrap_or_else(|| unreachable!("worker stdin is configured"));
    stdin
        .write_all(&command)
        .and_then(|()| stdin.write_all(b"\n"))
        .and_then(|()| stdin.write_all(secret))
        .unwrap_or_else(|error| unreachable!("worker fixture input: {error}"));
    drop(stdin);
    let output = child
        .wait_with_output()
        .unwrap_or_else(|error| unreachable!("worker fixture exits: {error}"));
    assert!(output.status.success());
    assert!(
        !output
            .stdout
            .windows(secret.len())
            .any(|value| value == secret)
    );
    assert!(
        !output
            .stderr
            .windows(secret.len())
            .any(|value| value == secret)
    );
    let response: CommercialWorkerResponse = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| unreachable!("valid worker response: {error}"));
    assert_eq!(
        response,
        CommercialWorkerResponse::Failure {
            reason_code: "invalid_policy".to_owned()
        }
    );
}
