use std::{fs, io, path::PathBuf};

use hunteval_protocol::{
    JsonlDecoder, ProtocolEnvelope, ProtocolError, ProtocolErrorCode, ProtocolPayload,
    ProtocolPhase, ProtocolSession, TrajectoryRecorder, replay_trajectory,
};

fn require_protocol_error<T>(result: Result<T, ProtocolError>) -> Result<ProtocolError, io::Error> {
    match result {
        Ok(_) => Err(io::Error::other("operation unexpectedly succeeded")),
        Err(error) => Ok(error),
    }
}

fn canonical_messages() -> Result<Vec<ProtocolEnvelope>, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("protocol crate is not inside the workspace"))?;
    let json =
        fs::read_to_string(workspace_root.join("examples/contracts/protocol-transcript.json"))?;
    Ok(serde_json::from_str(&json)?)
}

#[test]
fn canonical_transcript_replays_to_terminal_state() -> Result<(), Box<dyn std::error::Error>> {
    let messages = canonical_messages()?;
    let mut recorder = TrajectoryRecorder::new();
    for message in messages.clone() {
        recorder.append(message)?;
    }

    let outcome = replay_trajectory(recorder.as_bytes(), 128 * 1024)?;
    assert_eq!(outcome.event_count as usize, messages.len());
    assert_eq!(outcome.session.phase(), ProtocolPhase::Terminated);
    assert_eq!(outcome.session.task_count(), 1);
    assert_eq!(outcome.trajectory_sha256, recorder.digest());
    Ok(())
}

#[test]
fn direct_session_rejects_duplicate_message() -> Result<(), Box<dyn std::error::Error>> {
    let messages = canonical_messages()?;
    let first = messages
        .first()
        .ok_or_else(|| io::Error::other("canonical transcript is empty"))?;
    let mut session = ProtocolSession::new();
    session.accept(first)?;
    let error = require_protocol_error(session.accept(first))?;
    assert_eq!(error.code, ProtocolErrorCode::DuplicateIdentifier);
    Ok(())
}

#[test]
fn replay_rejects_tampered_hash_link() -> Result<(), Box<dyn std::error::Error>> {
    let mut recorder = TrajectoryRecorder::new();
    for message in canonical_messages()? {
        recorder.append(message)?;
    }
    let mut tampered = recorder.as_bytes().to_vec();
    let marker = b"\"previous_event_sha256\":\"";
    let position = tampered
        .windows(marker.len())
        .position(|window| window == marker)
        .ok_or_else(|| io::Error::other("trajectory has no predecessor hash"))?
        + marker.len();
    tampered[position] = if tampered[position] == b'0' {
        b'1'
    } else {
        b'0'
    };

    let error = require_protocol_error(replay_trajectory(&tampered, 128 * 1024))?;
    assert_eq!(error.code, ProtocolErrorCode::ProvenanceViolation);
    Ok(())
}

#[test]
fn replay_rejects_early_eof() -> Result<(), Box<dyn std::error::Error>> {
    let messages = canonical_messages()?;
    let mut recorder = TrajectoryRecorder::new();
    for message in messages.into_iter().take(3) {
        recorder.append(message)?;
    }
    let error = require_protocol_error(replay_trajectory(recorder.as_bytes(), 128 * 1024))?;
    assert_eq!(error.code, ProtocolErrorCode::ProcessFailure);
    Ok(())
}

#[test]
fn decoder_rejects_malformed_and_oversized_lines() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = JsonlDecoder::new(16)?;
    assert!(decoder.decode(b"{}without-newline").is_err());
    assert!(decoder.decode(b"{not-json}\n").is_err());
    assert!(decoder.decode(b"{\"message\":\"too long\"}\n").is_err());
    assert!(decoder.decode(b"{}\n{}\n").is_err());
    Ok(())
}

#[test]
fn session_rejects_unknown_agent_and_invalid_task_order() -> Result<(), Box<dyn std::error::Error>>
{
    let messages = canonical_messages()?;
    let mut session = ProtocolSession::new();
    for message in messages.iter().take(3) {
        session.accept(message)?;
    }

    let mut unknown_agent = messages[3].clone();
    if let ProtocolPayload::TaskCreated { agent_id, .. } = &mut unknown_agent.payload {
        *agent_id = "unregistered-agent".parse()?;
    }
    let error = require_protocol_error(session.accept(&unknown_agent))?;
    assert_eq!(error.code, ProtocolErrorCode::UnknownAgent);

    let started_before_creation = &messages[5];
    let error = require_protocol_error(session.accept(started_before_creation))?;
    assert_eq!(error.code, ProtocolErrorCode::UnknownTask);
    Ok(())
}

#[test]
fn session_rejects_forged_evidence_event() -> Result<(), Box<dyn std::error::Error>> {
    let messages = canonical_messages()?;
    let mut session = ProtocolSession::new();
    for message in messages.iter().take(8) {
        session.accept(message)?;
    }
    let mut forged = messages[8].clone();
    if let ProtocolPayload::EvidenceShared { evidence, .. } = &mut forged.payload {
        evidence.event_ids.insert("evt-forged".parse()?);
    }
    let error = require_protocol_error(session.accept(&forged))?;
    assert_eq!(error.code, ProtocolErrorCode::ProvenanceViolation);
    Ok(())
}
