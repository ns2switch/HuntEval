use std::{
    collections::BTreeMap,
    io,
    path::{Path, PathBuf},
};

use crate::{ArtifactWriter, EpisodePackage, RunManifest, hash_file};
use hunteval_domain::{
    ArtifactReferences, DeploymentId, MetricVector as DomainMetricVector, ResourceProvenance,
    ResourceUsage, RunId, RunResult, RunStatus, SchemaVersion, SourcedCost,
};
use hunteval_evaluation::{
    DeterministicEvaluator, EfficiencyInput, EvaluationInput, Evaluator, MetricVector,
    ScoringProfileArtifact, normalize_profile, score_profile,
};
use hunteval_protocol::{ProtocolEnvelope, ProtocolPayload, TrajectoryRecorder, replay_trajectory};

pub fn run_vertical_slice(
    episode_root: &Path,
    deployment_root: &Path,
    output: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let package = EpisodePackage::load(episode_root)?;
    let deployment_manifest = deployment_root.join("deployment.yaml");
    if !deployment_manifest.is_file() {
        return Err(io::Error::other("deployment manifest is unavailable").into());
    }
    execute_reference_query(&package)?;
    let messages = reference_transcript()?;
    let trajectory = record_messages(&messages)?;
    replay_trajectory(trajectory.as_bytes(), 128 * 1024)?;
    let submission = messages
        .iter()
        .find_map(|message| match &message.payload {
            ProtocolPayload::FinalSubmission { submission, .. } => Some(submission.clone()),
            _ => None,
        })
        .ok_or_else(|| io::Error::other("reference deployment omitted its submission"))?;
    let metrics = evaluate(&package, &messages, &submission)?;
    let profile = normalize_profile(serde_yaml_ng::from_slice::<ScoringProfileArtifact>(
        include_bytes!("../../../examples/scoring-profile-balanced.yaml"),
    )?)?;
    let aggregate = score_profile(&metrics, &profile)?;
    let run_id = RunId::new("run-001")?;
    let writer = ArtifactWriter::create(output, &RunId::new("latest")?)?;
    writer.write_json(
        Path::new("execution-policy.json"),
        &hunteval_sandbox::ResolvedExecutionPolicy::hardened_default(),
    )?;
    writer.append(Path::new("trajectory.jsonl"), trajectory.as_bytes())?;
    writer.write_json(Path::new("submission.json"), &submission)?;
    writer.write_json(Path::new("metrics.json"), &metrics)?;
    let result = build_result(&package, &metrics, aggregate.value, run_id.clone())?;
    writer.write_json(Path::new("result.json"), &result)?;
    let manifest = build_manifest(&package, deployment_manifest, writer.partial_root(), run_id)?;
    writer.write_json(Path::new("manifest.json"), &manifest)?;
    Ok(writer.finalize()?)
}

fn execute_reference_query(package: &EpisodePackage) -> Result<(), Box<dyn std::error::Error>> {
    let tables: Vec<_> = package
        .public()
        .manifest
        .telemetry
        .tables
        .iter()
        .map(|table| hunteval_duckdb::TableRegistration {
            name: table.name.clone(),
            parquet_path: package.public().public_root.join(&table.path),
        })
        .collect();
    let current = std::env::current_exe()?;
    let worker = current
        .parent()
        .ok_or_else(|| io::Error::other("binary directory is unavailable"))?
        .join("hunteval-duckdb-worker");
    let request = hunteval_duckdb::SqlRequest {
        query: "SELECT event_id, event_name FROM aws_cloudtrail WHERE event_id IN ('evt-0004', 'evt-0005', 'evt-0006') ORDER BY event_id".to_owned(),
        parameters: Vec::new(),
        limits: hunteval_duckdb::QueryLimits::default(),
    };
    let result = hunteval_duckdb::DuckDbWorker::new(worker, tables).execute(request)?;
    if result.rows.len() != 3 {
        return Err(
            io::Error::other("reference query did not recover the expected public events").into(),
        );
    }
    Ok(())
}

fn reference_transcript() -> Result<Vec<ProtocolEnvelope>, Box<dyn std::error::Error>> {
    let original = include_str!("../../../examples/contracts/protocol-transcript.json");
    let adapted = original
        .replace("evt-0012", "evt-0004")
        .replace("evt-0019", "evt-0005")
        .replace("principal:suspected", "arn:aws:iam::111122223333:user/alex");
    Ok(serde_json::from_str(&adapted)?)
}

fn record_messages(
    messages: &[ProtocolEnvelope],
) -> Result<TrajectoryRecorder, Box<dyn std::error::Error>> {
    let mut recorder = TrajectoryRecorder::new();
    for message in messages {
        recorder.append(message.clone())?;
    }
    Ok(recorder)
}

fn evaluate(
    package: &EpisodePackage,
    messages: &[ProtocolEnvelope],
    submission: &hunteval_domain::FinalSubmission,
) -> Result<MetricVector, Box<dyn std::error::Error>> {
    let evidence = messages
        .iter()
        .filter(|message| matches!(message.payload, ProtocolPayload::EvidenceShared { .. }))
        .count() as u64;
    let tasks_created = messages
        .iter()
        .filter(|message| matches!(message.payload, ProtocolPayload::TaskCreated { .. }))
        .count() as u64;
    let tasks_completed = messages
        .iter()
        .filter(|message| matches!(message.payload, ProtocolPayload::TaskCompleted { .. }))
        .count() as u64;
    let tool_calls = messages
        .iter()
        .filter(|message| matches!(message.payload, ProtocolPayload::ToolRequest { .. }))
        .count() as u64;
    Ok(DeterministicEvaluator.evaluate(&EvaluationInput {
        truth_events: package.ground_truth().malicious_event_ids.clone(),
        submitted_events: submission.malicious_event_ids.clone(),
        truth_entities: package.ground_truth().malicious_entity_ids.clone(),
        submitted_entities: submission.malicious_entity_ids.clone(),
        expected_attack_path: package.ground_truth().expected_attack_path.clone(),
        submitted_attack_path: submission.attack_path.clone(),
        expected_timeline_windows: package.ground_truth().expected_timeline_windows.clone(),
        submitted_timeline: submission.timeline.clone(),
        acceptable_submission_statuses: package
            .ground_truth()
            .acceptable_submission_statuses
            .clone(),
        submitted_status: submission.status,
        expected_attack_techniques: package.ground_truth().expected_attack_techniques.clone(),
        submitted_attack_techniques: submission.attack_techniques.clone(),
        grounded_evidence_events: submission.malicious_event_ids.clone(),
        grounded_evidence_entities: submission.malicious_entity_ids.clone(),
        submitted_grounded_evidence_items: evidence,
        minimum_evidence_items: u64::from(package.ground_truth().minimum_evidence_items),
        duplicate_tool_calls: 0,
        useful_messages: 0,
        operational_messages: 0,
        benign_scored_episode: package.public().manifest.benign_evaluation,
        evidence_items: evidence,
        grounded_evidence_items: evidence,
        findings_submitted: submission.finding_ids.len() as u64,
        provenance_references: evidence,
        valid_provenance_references: evidence,
        tasks_created,
        tasks_completed,
        tool_calls_used: tool_calls,
        tool_call_limit: package.public().manifest.limits.max_tool_calls.into(),
        resources: EfficiencyInput::default(),
    })?)
}

fn build_result(
    package: &EpisodePackage,
    metrics: &MetricVector,
    aggregate: Option<f64>,
    run_id: RunId,
) -> Result<RunResult, Box<dyn std::error::Error>> {
    let quality = average(
        metrics,
        &[
            "event_precision",
            "event_recall",
            "entity_precision",
            "entity_recall",
        ],
    );
    let evidence = average(metrics, &["evidence_grounding", "provenance_validity"]);
    let coordination = average(metrics, &["task_completion"]);
    let mut scores = BTreeMap::new();
    if let Some(value) = aggregate {
        scores.insert("balanced-0.3".into(), value);
    }
    let result = RunResult {
        schema_version: SchemaVersion::new(0, 3),
        run_id,
        episode_id: package.public().manifest.id.clone(),
        deployment_id: DeploymentId::new("two-agent-scripted")?,
        status: RunStatus::Completed,
        raw_metrics: metrics.0.clone(),
        metric_vector: DomainMetricVector {
            investigation_quality: quality,
            evidence_quality: evidence,
            coordination_quality: coordination,
            resilience: None,
            efficiency: None,
            reproducibility: None,
        },
        aggregate_scores: scores,
        aggregate_score_omissions: BTreeMap::new(),
        constraint_violations: Vec::new(),
        resource_usage: ResourceUsage {
            duration_ms: 0,
            tool_calls: 1,
            sql_queries: 1,
            messages: 13,
            input_tokens: None,
            output_tokens: None,
            token_provenance: ResourceProvenance::Unavailable,
            estimated_cost: SourcedCost {
                value: None,
                provenance: ResourceProvenance::Unavailable,
                currency: None,
            },
        },
        artifacts: ArtifactReferences {
            trajectory: "trajectory.jsonl".into(),
            submission: "submission.json".into(),
            metrics: "metrics.json".into(),
        },
    };
    result.validate()?;
    Ok(result)
}

fn average(metrics: &MetricVector, names: &[&str]) -> Option<f64> {
    let values: Vec<_> = names
        .iter()
        .filter_map(|name| metrics.0.get(*name).and_then(|metric| metric.value))
        .collect();
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn build_manifest(
    package: &EpisodePackage,
    deployment: PathBuf,
    run_root: &Path,
    run_id: RunId,
) -> Result<RunManifest, Box<dyn std::error::Error>> {
    let mut hashes = BTreeMap::from([
        ("episode_manifest".into(), package.digests().public_manifest),
        (
            "ground_truth".into(),
            package.digests().private_ground_truth,
        ),
        ("deployment_configuration".into(), hash_file(&deployment)?),
        (
            "scoring_profile".into(),
            hunteval_domain::Sha256Digest::from_bytes(include_bytes!(
                "../../../examples/scoring-profile-balanced.yaml"
            )),
        ),
        (
            "runner_binary".into(),
            hash_file(&std::env::current_exe()?)?,
        ),
    ]);
    for (name, digest) in &package.digests().public_telemetry {
        hashes.insert(format!("dataset:{name}"), *digest);
    }
    for (name, file) in [
        ("trajectory", "trajectory.jsonl"),
        ("submission", "submission.json"),
        ("metrics", "metrics.json"),
        ("result", "result.json"),
        ("execution_policy", "execution-policy.json"),
    ] {
        hashes.insert(name.to_owned(), hash_file(&run_root.join(file))?);
    }
    Ok(RunManifest {
        schema_version: SchemaVersion::new(0, 5),
        run_id,
        hashes,
        partial: false,
    })
}
