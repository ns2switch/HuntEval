use std::{
    collections::BTreeSet,
    io::{self, Write},
    process::{Command, Output, Stdio},
};

use hunteval_protocol::{
    ProtocolEnvelope, ProtocolPayload, ProtocolPhase, ProtocolSession, ToolOutcome,
};

fn canonical_messages() -> Result<Vec<ProtocolEnvelope>, serde_json::Error> {
    serde_json::from_str(include_str!(
        "../../../examples/contracts/protocol-transcript.json"
    ))
}

fn runner_messages(seed: u64) -> Result<Vec<ProtocolEnvelope>, Box<dyn std::error::Error>> {
    let canonical = canonical_messages()?;
    let mut started = canonical
        .first()
        .cloned()
        .ok_or_else(|| io::Error::other("canonical transcript is empty"))?;
    if let ProtocolPayload::RunStarted {
        seed: message_seed,
        limits,
        ..
    } = &mut started.payload
    {
        *message_seed = seed;
        limits.max_agents = 4;
    } else {
        return Err(io::Error::other("canonical transcript has no run_started").into());
    }

    let mut accepted = canonical
        .get(2)
        .cloned()
        .ok_or_else(|| io::Error::other("canonical transcript has no acceptance"))?;
    accepted.message_id = format!("runner-{seed}-002").parse()?;
    accepted.caused_by_message_id = Some(format!("deployment-{seed}-001").parse()?);

    let mut tool_result = canonical
        .get(7)
        .cloned()
        .ok_or_else(|| io::Error::other("canonical transcript has no tool result"))?;
    tool_result.message_id = format!("runner-{seed}-003").parse()?;
    tool_result.caused_by_message_id = Some(format!("deployment-{seed}-005").parse()?);
    if let ProtocolPayload::ToolResult {
        action_id,
        outcome,
        event_ids,
        result,
        ..
    } = &mut tool_result.payload
    {
        *action_id = format!("action-{seed}").parse()?;
        *outcome = ToolOutcome::Success;
        *event_ids = BTreeSet::from(["evt-0004".parse()?, "evt-0005".parse()?]);
        *result = serde_json::json!({
            "columns": ["event_id", "principal", "event_time", "event_name"],
            "rows": [
                [
                    {"type": "string", "value": "evt-0004"},
                    {"type": "string", "value": "suspected-identity"},
                    {"type": "string", "value": "2026-01-01T00:03:00Z"},
                    {"type": "string", "value": "AssumeAdmin"}
                ],
                [
                    {"type": "string", "value": "evt-0005"},
                    {"type": "string", "value": "suspected-identity"},
                    {"type": "string", "value": "2026-01-01T00:04:00Z"},
                    {"type": "string", "value": "GrantPrivilege"}
                ]
            ],
            "truncated": false
        });
    } else {
        return Err(io::Error::other("canonical transcript has no tool result payload").into());
    }

    let mut terminated = canonical
        .get(12)
        .cloned()
        .ok_or_else(|| io::Error::other("canonical transcript has no termination"))?;
    terminated.message_id = format!("runner-{seed}-004").parse()?;
    terminated.caused_by_message_id = Some(format!("deployment-{seed}-009").parse()?);
    Ok(vec![started, accepted, tool_result, terminated])
}

fn run_process(topology: &str, input: &[u8]) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_hunteval-reference-deployment"))
        .args(["--topology", topology])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("reference deployment stdin is unavailable"))?
        .write_all(input)?;
    Ok(child.wait_with_output()?)
}

fn run_session(
    topology: &str,
    seed: u64,
) -> Result<(Output, Vec<ProtocolEnvelope>), Box<dyn std::error::Error>> {
    let runner = runner_messages(seed)?;
    let mut input = Vec::new();
    for message in &runner {
        serde_json::to_writer(&mut input, message)?;
        input.push(b'\n');
    }
    let output = run_process(topology, &input)?;
    let deployment = output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice)
        .collect::<Result<Vec<ProtocolEnvelope>, _>>()?;
    Ok((output, deployment))
}

fn validate_transcript(
    runner: &[ProtocolEnvelope],
    deployment: &[ProtocolEnvelope],
) -> Result<(), Box<dyn std::error::Error>> {
    if deployment.len() != 9 {
        return Err(io::Error::other("reference deployment emitted an unexpected flow").into());
    }
    let mut session = ProtocolSession::new();
    session.accept(&runner[0])?;
    session.accept(&deployment[0])?;
    session.accept(&runner[1])?;
    for message in &deployment[1..5] {
        session.accept(message)?;
    }
    session.accept(&runner[2])?;
    for message in &deployment[5..] {
        session.accept(message)?;
    }
    session.accept(&runner[3])?;
    session.finish()?;
    assert_eq!(session.phase(), ProtocolPhase::Terminated);
    Ok(())
}

#[test]
fn all_reference_topologies_complete_a_managed_protocol_session()
-> Result<(), Box<dyn std::error::Error>> {
    for topology in [
        "single-agent",
        "supervisor-worker",
        "supervisor-specialist",
        "supervisor-specialists",
    ] {
        let seed = 11;
        let runner = runner_messages(seed)?;
        let (output, deployment) = run_session(topology, seed)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8(output.stderr)?
        );
        validate_transcript(&runner, &deployment)?;

        let requests = deployment
            .iter()
            .filter_map(|message| match &message.payload {
                ProtocolPayload::ToolRequest {
                    tool, arguments, ..
                } => Some((tool, arguments)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, "duckdb_sql");
        let query = requests[0].1["query"]
            .as_str()
            .ok_or_else(|| io::Error::other("tool request query is not text"))?;
        assert!(query.contains("source_ip = ?"));
        assert!(!query.contains("evt-"));
    }
    Ok(())
}

#[test]
fn same_seed_produces_identical_protocol_output() -> Result<(), Box<dyn std::error::Error>> {
    let (first, _) = run_session("supervisor-worker", 29)?;
    let (second, _) = run_session("supervisor-worker", 29)?;
    let (different, _) = run_session("supervisor-worker", 31)?;
    assert!(first.status.success());
    assert!(second.status.success());
    assert!(different.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_ne!(first.stdout, different.stdout);
    Ok(())
}

#[test]
fn malformed_input_and_early_eof_fail_without_echoing_input()
-> Result<(), Box<dyn std::error::Error>> {
    let marker = "PRIVATE-MARKER-SHOULD-NOT-BE-ECHOED";
    let malformed = run_process("single-agent", format!("{{{marker}}}\n").as_bytes())?;
    assert!(!malformed.status.success());
    assert!(malformed.stdout.is_empty());
    assert!(!String::from_utf8(malformed.stderr)?.contains(marker));

    let early = run_process("single-agent", b"")?;
    assert!(!early.status.success());
    assert!(early.stdout.is_empty());
    Ok(())
}
