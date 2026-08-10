use std::{collections::BTreeMap, fs, io, path::Path};

use hunteval_domain::{
    BenchmarkDefinition, BenchmarkId, FinalSubmission, ResolvedArtifact, ResolvedDeployment,
    ResolvedEpisode, RunId, SchemaVersion, ScoringProfileId, Sha256Digest,
};
use hunteval_protocol::{ProtocolEnvelope, ProtocolPayload, TrajectoryRecorder};
use hunteval_runner::{
    BenchmarkCellState, BenchmarkCellStatus, BenchmarkState, DiagnosticBundleManifest,
    DiagnosticVerificationStatus, RunManifest, generate_benchmark_diagnosis,
    verify_diagnostic_bundle,
};

#[test]
fn benchmark_diagnosis_generates_recurrence_and_verifiable_report()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let benchmark = temporary.path().join("benchmark");
    let run_id = "run-001";
    fs::create_dir_all(benchmark.join("runs").join(run_id))?;
    let definition = definition()?;
    let cell = definition.cells()?.remove(0);
    let result_sha256 = write_failed_tool_run(
        &benchmark.join("runs").join(run_id),
        &cell.cell_id.to_string(),
    )?;
    fs::write(
        benchmark.join("benchmark-definition.json"),
        serde_json::to_vec_pretty(&definition)?,
    )?;
    let state = BenchmarkState {
        schema_version: SchemaVersion::new(0, 4),
        benchmark_id: definition.id.clone(),
        last_sequence: 1,
        last_event_sha256: Sha256Digest::from_bytes(b"benchmark-event"),
        cells: vec![BenchmarkCellState {
            cell_id: cell.cell_id,
            status: BenchmarkCellStatus::Completed,
            attempt_ids: Vec::new(),
            run_id: Some(RunId::new(run_id)?),
            result_sha256: Some(result_sha256),
            reason_code: None,
        }],
    };
    fs::write(
        benchmark.join("benchmark-state.json"),
        serde_json::to_vec_pretty(&state)?,
    )?;
    let output = temporary.path().join("diagnosis");
    generate_benchmark_diagnosis(&benchmark, &output)?;
    let verification = verify_diagnostic_bundle(&output);
    assert_eq!(
        verification.status,
        DiagnosticVerificationStatus::Verified,
        "{:?}",
        verification.reasons
    );
    let recurrence = fs::read_to_string(output.join("diagnostic-recurrence.json"))?;
    assert!(recurrence.contains("task_incomplete"));
    assert!(recurrence.contains("recurrence_is_not_causality"));
    let manifest: DiagnosticBundleManifest =
        serde_json::from_slice(&fs::read(output.join("diagnostic-bundle-manifest.json"))?)?;
    assert!(manifest.artifacts.iter().all(|artifact| {
        !artifact
            .path
            .chars()
            .any(|character| matches!(character, '"' | ':' | '<' | '>' | '|' | '*' | '?'))
    }));
    Ok(())
}

fn definition() -> Result<BenchmarkDefinition, Box<dyn std::error::Error>> {
    Ok(BenchmarkDefinition::new(
        BenchmarkId::new("diagnostic-benchmark")?,
        vec![ResolvedDeployment {
            configuration_sha256: Sha256Digest::from_bytes(b"deployment-config"),
            id: "deployment-001".parse()?,
        }],
        vec![ResolvedEpisode {
            id: "episode-001".parse()?,
            package_sha256: Sha256Digest::from_bytes(b"episode"),
        }],
        vec![11],
        ResolvedArtifact {
            id: ScoringProfileId::new("scoring-001")?,
            sha256: Sha256Digest::from_bytes(b"scoring"),
        },
        None,
    )?)
}

fn write_failed_tool_run(
    root: &Path,
    cell_id: &str,
) -> Result<Sha256Digest, Box<dyn std::error::Error>> {
    let mut messages: Vec<ProtocolEnvelope> = serde_json::from_str(include_str!(
        "../../../examples/contracts/protocol-transcript.json"
    ))?;
    for message in &mut messages {
        if let ProtocolPayload::TaskCompleted { agent_id, task_id } = &message.payload {
            message.payload = ProtocolPayload::TaskFailed {
                agent_id: agent_id.clone(),
                task_id: task_id.clone(),
                reason_code: "fixture_failure".into(),
            };
            break;
        }
    }
    let mut recorder = TrajectoryRecorder::new();
    let mut submission: Option<FinalSubmission> = None;
    for message in messages {
        if let ProtocolPayload::FinalSubmission {
            submission: value, ..
        } = &message.payload
        {
            submission = Some(value.clone());
        }
        recorder.append(message)?;
    }
    let submission = submission.ok_or_else(|| io::Error::other("fixture has no submission"))?;
    let trajectory = recorder.as_bytes();
    let mut submission_bytes = serde_json::to_vec_pretty(&submission)?;
    submission_bytes.push(b'\n');
    let metrics = b"{}\n";
    let aggregate = b"{}\n";
    fs::write(root.join("trajectory.jsonl"), trajectory)?;
    fs::write(root.join("submission.json"), &submission_bytes)?;
    fs::write(root.join("metrics.json"), metrics)?;
    fs::write(root.join("aggregate-score.json"), aggregate)?;
    let mut hashes = BTreeMap::from([
        ("trajectory".into(), Sha256Digest::from_bytes(trajectory)),
        (
            "submission".into(),
            Sha256Digest::from_bytes(&submission_bytes),
        ),
        ("metrics".into(), Sha256Digest::from_bytes(metrics)),
        (
            "aggregate_score".into(),
            Sha256Digest::from_bytes(aggregate),
        ),
    ]);
    let result = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": "0.4",
        "run_id": "run-001",
        "cell_id": cell_id,
        "artifact_hashes": {
            "trajectory": hashes["trajectory"],
            "submission": hashes["submission"],
            "metrics": hashes["metrics"],
            "aggregate_score": hashes["aggregate_score"]
        }
    }))?;
    let result_sha256 = Sha256Digest::from_bytes(&result);
    fs::write(root.join("result.json"), &result)?;
    hashes.insert("result".into(), result_sha256);
    let manifest = RunManifest {
        schema_version: SchemaVersion::new(0, 4),
        run_id: RunId::new("run-001")?,
        hashes,
        partial: false,
    };
    fs::write(root.join("manifest.json"), serde_json::to_vec(&manifest)?)?;
    Ok(result_sha256)
}
