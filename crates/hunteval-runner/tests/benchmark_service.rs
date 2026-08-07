use std::{
    collections::BTreeMap,
    fs,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

use hunteval_domain::{
    BenchmarkAttemptId, BenchmarkCell, BenchmarkDefinition, BenchmarkId, DeploymentId, EpisodeId,
    ResolvedArtifact, ResolvedDeployment, ResolvedEpisode, RunId, ScoringProfileId, Sha256Digest,
};
use hunteval_runner::{
    BenchmarkCellExecutor, BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkService,
    CellExecution, CellExecutionFailure, ComparisonReason, ComparisonStatus, RetryPolicy,
};

#[derive(Debug, Default)]
struct FakeExecutor {
    active: AtomicUsize,
    maximum_active: AtomicUsize,
    fail_once: AtomicBool,
}

impl FakeExecutor {
    fn failing_once() -> Self {
        Self {
            fail_once: AtomicBool::new(true),
            ..Self::default()
        }
    }
}

impl BenchmarkCellExecutor for FakeExecutor {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum_active.fetch_max(active, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(10));
        self.active.fetch_sub(1, Ordering::SeqCst);
        if self
            .fail_once
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return Err(CellExecutionFailure {
                reason_code: "forced_failure".to_owned(),
            });
        }
        let run_root = output_root.join(run_id.as_str());
        fs::create_dir_all(&run_root).map_err(|_| CellExecutionFailure {
            reason_code: "fixture_io".to_owned(),
        })?;
        let result_path = run_root.join("result.json");
        let bytes = serde_json::to_vec(&serde_json::json!({
            "cell_id": cell.cell_id,
            "run_id": run_id,
            "status": "completed"
        }))
        .map_err(|_| CellExecutionFailure {
            reason_code: "fixture_serialization".to_owned(),
        })?;
        fs::write(&result_path, bytes).map_err(|_| CellExecutionFailure {
            reason_code: "fixture_io".to_owned(),
        })?;
        Ok(CellExecution {
            run_id: run_id.clone(),
            result_path,
        })
    }
}

#[test]
fn executes_exact_cartesian_matrix_with_bounded_parallelism()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let plan = make_plan(&["left", "right"], &["episode-a", "episode-b"], &[11, 29])?;
    let executor = FakeExecutor::default();
    let summary = BenchmarkService::new(&executor).run(
        &plan,
        temporary.path(),
        BenchmarkRunOptions {
            jobs: 2,
            ..BenchmarkRunOptions::default()
        },
    )?;
    assert_eq!(summary.total, 8);
    assert_eq!(summary.completed, 8);
    assert_eq!(summary.failed, 0);
    assert_eq!(executor.maximum_active.load(Ordering::SeqCst), 2);
    Ok(())
}

#[test]
fn resumes_pending_and_authorized_failed_cells_without_overwriting_attempts()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let plan = make_plan(&["left", "right"], &["episode-a", "episode-b"], &[11])?;
    let executor = FakeExecutor::failing_once();
    let service = BenchmarkService::new(&executor);
    let first = service.run(
        &plan,
        temporary.path(),
        BenchmarkRunOptions {
            jobs: 2,
            fail_fast: true,
            retry: RetryPolicy::None,
        },
    )?;
    assert_eq!(first.failed, 1);
    assert_eq!(first.pending, 2);
    let resumed = service.run(
        &plan,
        temporary.path(),
        BenchmarkRunOptions {
            jobs: 2,
            retry: RetryPolicy::Failed,
            ..BenchmarkRunOptions::default()
        },
    )?;
    assert_eq!(resumed.completed, 4);
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(temporary.path().join("benchmark-state.json"))?)?;
    let attempts = state["cells"]
        .as_array()
        .ok_or_else(|| std::io::Error::other("cells missing"))?
        .iter()
        .map(|cell| cell["attempt_ids"].as_array().map_or(0, Vec::len))
        .sum::<usize>();
    assert_eq!(attempts, 5);
    Ok(())
}

#[test]
fn rejects_configuration_drift_and_reports_ineligible_missing_pair()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let plan = make_plan(&["left", "right"], &["episode-a"], &[11])?;
    let executor = FakeExecutor::default();
    let service = BenchmarkService::new(&executor);
    service.run(&plan, temporary.path(), BenchmarkRunOptions::default())?;
    let eligible = BenchmarkService::compare(
        temporary.path(),
        &plan.definition,
        &DeploymentId::new("left")?,
        &DeploymentId::new("right")?,
    )?;
    assert_eq!(eligible.status, ComparisonStatus::Eligible);
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(temporary.path().join("benchmark-state.json"))?)?;
    let run_id = state["cells"][0]["run_id"]
        .as_str()
        .ok_or_else(|| std::io::Error::other("run id missing"))?;
    fs::write(
        temporary
            .path()
            .join("runs")
            .join(run_id)
            .join("result.json"),
        b"tampered",
    )?;
    let tampered = BenchmarkService::compare(
        temporary.path(),
        &plan.definition,
        &DeploymentId::new("left")?,
        &DeploymentId::new("right")?,
    )?;
    assert!(
        tampered
            .reasons
            .contains(&ComparisonReason::ArtifactVerificationFailed)
    );
    let missing = BenchmarkService::compare(
        temporary.path(),
        &plan.definition,
        &DeploymentId::new("left")?,
        &DeploymentId::new("absent")?,
    )?;
    assert_eq!(missing.status, ComparisonStatus::Ineligible);

    let drifted = make_plan(&["left", "right"], &["episode-a"], &[29])?;
    assert!(matches!(
        service.run(&drifted, temporary.path(), BenchmarkRunOptions::default()),
        Err(hunteval_runner::BenchmarkServiceError::ConfigurationDrift)
    ));
    Ok(())
}

fn make_plan(
    deployments: &[&str],
    episodes: &[&str],
    seeds: &[u64],
) -> Result<BenchmarkExecutionPlan, Box<dyn std::error::Error>> {
    let deployments = deployments
        .iter()
        .map(|id| {
            Ok(ResolvedDeployment {
                id: DeploymentId::new(*id)?,
                configuration_sha256: Sha256Digest::from_bytes(id.as_bytes()),
            })
        })
        .collect::<Result<Vec<_>, hunteval_domain::IdValidationError>>()?;
    let episodes = episodes
        .iter()
        .map(|id| {
            Ok(ResolvedEpisode {
                id: EpisodeId::new(*id)?,
                package_sha256: Sha256Digest::from_bytes(id.as_bytes()),
            })
        })
        .collect::<Result<Vec<_>, hunteval_domain::IdValidationError>>()?;
    let definition = BenchmarkDefinition::new(
        BenchmarkId::new("service-test")?,
        deployments,
        episodes,
        seeds.to_vec(),
        ResolvedArtifact {
            id: ScoringProfileId::new("balanced")?,
            sha256: Sha256Digest::from_bytes(b"profile"),
        },
        None,
    )?;
    Ok(BenchmarkExecutionPlan {
        deployments: definition
            .deployments
            .iter()
            .map(|item| (item.id.clone(), item.id.as_str().into()))
            .collect::<BTreeMap<_, _>>(),
        episodes: definition
            .episodes
            .iter()
            .map(|item| (item.id.clone(), item.id.as_str().into()))
            .collect::<BTreeMap<_, _>>(),
        scoring_profile: "profile.yaml".into(),
        definition,
    })
}
