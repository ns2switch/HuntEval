use std::{collections::BTreeMap, fs};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCell, BenchmarkDefinition, BenchmarkId, DeploymentId, EpisodeId,
    ResolvedArtifact, ResolvedDeployment, ResolvedEpisode, RunId, ScoringProfileId, Sha256Digest,
};
use hunteval_runner::{
    BenchmarkCellExecutor, BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkService,
    CellExecution, CellExecutionFailure, ReportFormat, generate_report, verify_report,
};

#[derive(Debug)]
struct FailingExecutor;

impl BenchmarkCellExecutor for FailingExecutor {
    fn execute(
        &self,
        _cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        _run_id: &RunId,
        _output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        Err(CellExecutionFailure {
            reason_code: "fixture_failure".to_owned(),
        })
    }
}

#[derive(Debug)]
struct CompletedExecutor;

impl BenchmarkCellExecutor for CompletedExecutor {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        let result = serde_json::json!({
            "schema_version": "0.4",
            "cell_id": cell.cell_id,
            "run_id": run_id,
            "cell": cell,
            "metrics": {
                "event_recall": {
                    "value": 0.75,
                    "applicability": "applicable",
                    "direction": "higher_is_better",
                    "range": {"minimum": 0.0, "maximum": 1.0},
                    "numerator": 3,
                    "denominator": 4
                }
            },
            "aggregate_score": {
                "profile_id": "profile",
                "value": 0.75,
                "omitted_metrics": {}
            },
            "constraints": [],
            "usage": {
                "tool_calls": 1,
                "messages": 1,
                "tokens": 0
            },
            "resource_usage": {
                "duration_ms": 10,
                "tool_calls": 1,
                "sql_queries": 1,
                "messages": 1,
                "input_tokens": null,
                "output_tokens": null,
                "token_provenance": "unavailable",
                "estimated_cost": {"value": null, "provenance": "unavailable", "currency": null}
            },
            "submission": {
                "status": "no_malicious_activity",
                "summary": "No malicious activity observed.",
                "finding_ids": [],
                "malicious_event_ids": [],
                "malicious_entity_ids": [],
                "attack_path": [],
                "attack_techniques": [],
                "confidence": 0.5,
                "limitations": [],
                "timeline": []
            },
            "artifact_hashes": {}
        });
        let run_root = output_root.join(run_id.as_str());
        fs::create_dir_all(&run_root).map_err(|_| failure("fixture_io"))?;
        let result_path = run_root.join("result.json");
        let mut bytes =
            serde_json::to_vec_pretty(&result).map_err(|_| failure("fixture_serialization"))?;
        bytes.push(b'\n');
        fs::write(&result_path, bytes).map_err(|_| failure("fixture_io"))?;
        Ok(CellExecution {
            run_id: run_id.clone(),
            result_path,
        })
    }
}

#[test]
fn benchmark_reports_are_deterministic_static_and_verifiable()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let plan = plan()?;
    BenchmarkService::new(&FailingExecutor).run(
        &plan,
        root.path(),
        BenchmarkRunOptions::default(),
    )?;

    generate_report(root.path(), ReportFormat::Json)?;
    let first = fs::read(root.path().join("benchmark-report.json"))?;
    generate_report(root.path(), ReportFormat::Json)?;
    assert_eq!(first, fs::read(root.path().join("benchmark-report.json"))?);

    generate_report(root.path(), ReportFormat::Html)?;
    let html = String::from_utf8(fs::read(root.path().join("benchmark-report.html"))?)?;
    assert!(html.starts_with("<!doctype html>"));
    assert!(!html.contains("<script"));
    assert!(verify_report(root.path())?.valid);

    fs::write(root.path().join("benchmark-events.jsonl"), b"tampered\n")?;
    let verification = verify_report(root.path())?;
    assert!(!verification.valid);
    assert!(
        verification
            .errors
            .iter()
            .any(|error| error == "digest_mismatch:benchmark-events.jsonl")
    );
    Ok(())
}

#[test]
fn completed_cells_feed_metrics_pairing_and_ranking() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let plan = plan()?;
    BenchmarkService::new(&CompletedExecutor).run(
        &plan,
        root.path(),
        BenchmarkRunOptions::default(),
    )?;
    generate_report(root.path(), ReportFormat::Json)?;
    let report: hunteval_reporting::BenchmarkResult =
        serde_json::from_slice(&fs::read(root.path().join("benchmark-report.json"))?)?;
    assert!(
        report
            .deployments
            .iter()
            .all(|item| item.completed_cells == 1)
    );
    assert_eq!(report.comparisons[0].aggregate_difference.count, 1);
    assert_eq!(report.rankings[0].deployments.len(), 2);
    assert!(
        report
            .cells
            .iter()
            .all(|cell| cell.resource_usage.is_some())
    );
    Ok(())
}

fn plan() -> Result<BenchmarkExecutionPlan, Box<dyn std::error::Error>> {
    let deployments = vec![
        ResolvedDeployment {
            id: DeploymentId::new("left")?,
            configuration_sha256: Sha256Digest::from_bytes(b"left"),
        },
        ResolvedDeployment {
            id: DeploymentId::new("right")?,
            configuration_sha256: Sha256Digest::from_bytes(b"right"),
        },
    ];
    let definition = BenchmarkDefinition::new(
        BenchmarkId::new("report-test")?,
        deployments,
        vec![ResolvedEpisode {
            id: EpisodeId::new("episode-a")?,
            package_sha256: Sha256Digest::from_bytes(b"episode"),
        }],
        vec![11],
        ResolvedArtifact {
            id: ScoringProfileId::new("profile")?,
            sha256: Sha256Digest::from_bytes(b"profile"),
        },
        None,
    )?;
    Ok(BenchmarkExecutionPlan {
        deployments: BTreeMap::from([
            (DeploymentId::new("left")?, "left.json".into()),
            (DeploymentId::new("right")?, "right.json".into()),
        ]),
        episodes: BTreeMap::from([(EpisodeId::new("episode-a")?, "episode".into())]),
        scoring_profile: "profile.yaml".into(),
        definition,
    })
}

fn failure(reason_code: &str) -> CellExecutionFailure {
    CellExecutionFailure {
        reason_code: reason_code.to_owned(),
    }
}
