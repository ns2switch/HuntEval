use std::{collections::BTreeMap, fs};

use hunteval_domain::{
    Applicability, BenchmarkAttemptId, BenchmarkCell, BenchmarkDefinition, BenchmarkId, Confidence,
    DeploymentId, EpisodeId, FinalSubmission, MetricDirection, MetricRange, MetricValue,
    ResolvedArtifact, ResolvedDeployment, ResolvedEpisode, ResourceProvenance, ResourceUsage,
    RunId, SchemaVersion, ScoringProfileId, Sha256Digest, SourcedCost, SubmissionStatus,
};
use hunteval_evaluation::{AggregateScore, MetricVector};
use hunteval_runner::{
    BenchmarkCellExecutor, BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkService,
    CellExecution, CellExecutionFailure,
};
use serde::Serialize;

#[derive(Debug)]
struct MetricExecutor {
    divergent_seed: Option<u64>,
    failed_seed: Option<u64>,
}

impl BenchmarkCellExecutor for MetricExecutor {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        if self.failed_seed == Some(cell.key.seed) {
            return Err(failure("fixture_failure"));
        }
        let divergent = self.divergent_seed == Some(cell.key.seed);
        let result = FixtureResult {
            schema_version: SchemaVersion::new(0, 4),
            cell_id: cell.cell_id,
            run_id: run_id.clone(),
            cell: cell.clone(),
            metrics: MetricVector(BTreeMap::from([(
                "event_recall".to_owned(),
                metric(if divergent { 0.0 } else { 1.0 }),
            )])),
            aggregate_score: AggregateScore {
                profile_id: "test".to_owned(),
                value: Some(if divergent { 0.0 } else { 1.0 }),
                omitted_metrics: BTreeMap::new(),
            },
            constraints: Vec::new(),
            usage: hunteval_runner::BudgetUsage::default(),
            resource_usage: resource_usage(),
            submission: submission(if divergent { "event-b" } else { "event-a" })?,
            artifact_hashes: BTreeMap::new(),
        };
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

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct FixtureResult {
    schema_version: SchemaVersion,
    cell_id: hunteval_domain::BenchmarkCellId,
    run_id: RunId,
    cell: BenchmarkCell,
    metrics: MetricVector,
    aggregate_score: AggregateScore,
    constraints: Vec<hunteval_evaluation::ConstraintEvaluation>,
    usage: hunteval_runner::BudgetUsage,
    resource_usage: ResourceUsage,
    submission: FinalSubmission,
    artifact_hashes: BTreeMap<String, Sha256Digest>,
}

#[test]
fn aggregates_identical_and_divergent_seed_repetitions() -> Result<(), Box<dyn std::error::Error>> {
    let identical_root = tempfile::tempdir()?;
    let plan = make_plan(&[29, 11])?;
    let identical = MetricExecutor {
        divergent_seed: None,
        failed_seed: None,
    };
    BenchmarkService::new(&identical).run(
        &plan,
        identical_root.path(),
        BenchmarkRunOptions::default(),
    )?;
    let summary = BenchmarkService::metrics(identical_root.path(), &plan.definition)?;
    assert_eq!(
        summary.groups[0].stability.submission_stability.value,
        Some(1.0)
    );
    assert_eq!(
        summary.groups[0].stability.metric_stability.value,
        Some(1.0)
    );

    let divergent_root = tempfile::tempdir()?;
    let divergent = MetricExecutor {
        divergent_seed: Some(29),
        failed_seed: None,
    };
    BenchmarkService::new(&divergent).run(
        &plan,
        divergent_root.path(),
        BenchmarkRunOptions::default(),
    )?;
    let summary = BenchmarkService::metrics(divergent_root.path(), &plan.definition)?;
    assert!(
        summary.groups[0]
            .stability
            .submission_stability
            .value
            .is_some_and(|value| value < 1.0)
    );
    assert_eq!(
        summary.groups[0].stability.metric_stability.value,
        Some(0.0)
    );
    assert_eq!(summary.groups[0].contributing_cell_ids.len(), 2);
    Ok(())
}

#[test]
fn reports_failed_and_tampered_repetitions_without_imputation()
-> Result<(), Box<dyn std::error::Error>> {
    let failed_root = tempfile::tempdir()?;
    let plan = make_plan(&[11, 29])?;
    let executor = MetricExecutor {
        divergent_seed: None,
        failed_seed: Some(29),
    };
    BenchmarkService::new(&executor).run(
        &plan,
        failed_root.path(),
        BenchmarkRunOptions::default(),
    )?;
    let summary = BenchmarkService::metrics(failed_root.path(), &plan.definition)?;
    assert_eq!(summary.groups[0].stability.metric_stability.value, None);
    assert_eq!(summary.groups[0].stability.unavailable[0].seed, 29);

    let complete_root = tempfile::tempdir()?;
    let complete = MetricExecutor {
        divergent_seed: None,
        failed_seed: None,
    };
    BenchmarkService::new(&complete).run(
        &plan,
        complete_root.path(),
        BenchmarkRunOptions::default(),
    )?;
    let state: serde_json::Value = serde_json::from_slice(&fs::read(
        complete_root.path().join("benchmark-state.json"),
    )?)?;
    let run_id = state["cells"][0]["run_id"]
        .as_str()
        .ok_or_else(|| std::io::Error::other("missing run id"))?;
    fs::write(
        complete_root
            .path()
            .join("runs")
            .join(run_id)
            .join("result.json"),
        b"tampered",
    )?;
    let summary = BenchmarkService::metrics(complete_root.path(), &plan.definition)?;
    assert_eq!(summary.groups[0].stability.metric_stability.value, None);
    assert_eq!(
        summary.groups[0].stability.unavailable[0].reason,
        hunteval_statistics::UnavailableRepetitionReason::InvalidArtifact
    );
    Ok(())
}

fn make_plan(seeds: &[u64]) -> Result<BenchmarkExecutionPlan, Box<dyn std::error::Error>> {
    let definition = BenchmarkDefinition::new(
        BenchmarkId::new("metric-test")?,
        vec![ResolvedDeployment {
            id: DeploymentId::new("deployment-a")?,
            configuration_sha256: Sha256Digest::from_bytes(b"deployment"),
        }],
        vec![ResolvedEpisode {
            id: EpisodeId::new("episode-a")?,
            package_sha256: Sha256Digest::from_bytes(b"episode"),
        }],
        seeds.to_vec(),
        ResolvedArtifact {
            id: ScoringProfileId::new("profile")?,
            sha256: Sha256Digest::from_bytes(b"profile"),
        },
        None,
    )?;
    Ok(BenchmarkExecutionPlan {
        deployments: BTreeMap::from([(DeploymentId::new("deployment-a")?, "deployment".into())]),
        episodes: BTreeMap::from([(EpisodeId::new("episode-a")?, "episode".into())]),
        scoring_profile: "profile.yaml".into(),
        definition,
    })
}

fn submission(event_id: &str) -> Result<FinalSubmission, CellExecutionFailure> {
    Ok(FinalSubmission {
        status: SubmissionStatus::NoMaliciousActivity,
        summary: "structured fixture".to_owned(),
        finding_ids: Default::default(),
        malicious_event_ids: [
            hunteval_domain::EventId::new(event_id).map_err(|_| failure("fixture_identifier"))?
        ]
        .into_iter()
        .collect(),
        malicious_entity_ids: Default::default(),
        attack_path: Vec::new(),
        attack_techniques: Default::default(),
        confidence: Confidence::new(0.5).map_err(|_| failure("fixture_confidence"))?,
        limitations: Vec::new(),
        timeline: None,
    })
}

fn metric(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        applicability: Applicability::Applicable,
        direction: MetricDirection::HigherIsBetter,
        range: MetricRange {
            minimum: 0.0,
            maximum: 1.0,
        },
        numerator: None,
        denominator: None,
    }
}

fn resource_usage() -> ResourceUsage {
    ResourceUsage {
        duration_ms: 10,
        tool_calls: 0,
        sql_queries: 0,
        messages: 0,
        input_tokens: None,
        output_tokens: None,
        token_provenance: ResourceProvenance::Unavailable,
        estimated_cost: SourcedCost {
            value: None,
            provenance: ResourceProvenance::Unavailable,
            currency: None,
        },
    }
}

fn failure(reason_code: &str) -> CellExecutionFailure {
    CellExecutionFailure {
        reason_code: reason_code.to_owned(),
    }
}
