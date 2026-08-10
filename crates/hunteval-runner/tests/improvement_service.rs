use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use hunteval_domain::*;
use hunteval_runner::*;

#[derive(Debug)]
struct FakeExecutor;

impl BenchmarkCellExecutor for FakeExecutor {
    fn execute(
        &self,
        cell: &BenchmarkCell,
        _attempt_id: &BenchmarkAttemptId,
        run_id: &RunId,
        output_root: &std::path::Path,
    ) -> Result<CellExecution, CellExecutionFailure> {
        let root = output_root.join(run_id.as_str());
        fs::create_dir_all(&root).map_err(|_| CellExecutionFailure {
            reason_code: "fixture_io".into(),
        })?;
        let result_path = root.join("result.json");
        let bytes = serde_json::to_vec(&serde_json::json!({
            "cell_id": cell.cell_id,
            "run_id": run_id,
            "status": "completed"
        }))
        .map_err(|_| CellExecutionFailure {
            reason_code: "fixture_serialization".into(),
        })?;
        fs::write(&result_path, bytes).map_err(|_| CellExecutionFailure {
            reason_code: "fixture_io".into(),
        })?;
        Ok(CellExecution {
            run_id: run_id.clone(),
            result_path,
        })
    }
}

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::from_bytes(value)
}

fn plan() -> Result<BenchmarkExecutionPlan, Box<dyn std::error::Error>> {
    let deployments = ["baseline", "candidate"]
        .into_iter()
        .map(|id| {
            Ok(ResolvedDeployment {
                id: DeploymentId::new(id)?,
                configuration_sha256: digest(id),
            })
        })
        .collect::<Result<Vec<_>, hunteval_domain::IdValidationError>>()?;
    let episodes = vec![ResolvedEpisode {
        id: EpisodeId::new("episode")?,
        package_sha256: digest("episode"),
    }];
    let definition = BenchmarkDefinition::new(
        BenchmarkId::new("improvement-service")?,
        deployments,
        episodes,
        vec![7],
        ResolvedArtifact {
            id: ScoringProfileId::new("balanced")?,
            sha256: digest("profile"),
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

#[test]
fn improvement_orchestration_reuses_canonical_benchmark_service()
-> Result<(), Box<dyn std::error::Error>> {
    let plan = plan()?;
    let cells = plan.definition.cells()?;
    let experiment_sha = digest("experiment");
    let diff_sha = digest("diff");
    let experiment = ImprovementExperiment {
        schema_version: SchemaVersion::new(0, 8),
        id: "experiment".into(),
        lineage_id: "lineage".into(),
        baseline_artifact_sha256: digest("baseline-artifact"),
        candidate_artifact_sha256: digest("candidate-artifact"),
        artifact_diff_sha256: diff_sha,
        improvement_policy_sha256: digest("policy"),
        partition_policy_sha256: digest("partitions"),
        scoring_profile_sha256: digest("profile"),
        statistical_policy_sha256: digest("statistics"),
        changed_variable: "supervisor_instruction".into(),
        control_hashes: ImprovementControlHashes {
            episode_set: digest("episodes"),
            seed_set: digest("seeds"),
            budgets: digest("budgets"),
            models: digest("models"),
            topology: digest("topology"),
            managed_tool_policy: digest("tools"),
            execution_policy: digest("execution"),
            schemas: digest("schemas"),
            runtime_binaries: digest("binaries"),
        },
        paired_cells: vec![PairedCellReference {
            baseline_cell_id: cells[0].cell_id.to_string(),
            candidate_cell_id: cells[1].cell_id.to_string(),
        }],
        candidate_frozen: true,
    };
    let equivalence = ImprovementEquivalenceResult {
        schema_version: SchemaVersion::new(0, 8),
        experiment_id: experiment.id.clone(),
        experiment_sha256: experiment_sha,
        artifact_diff_sha256: diff_sha,
        status: ImprovementEquivalenceStatus::Eligible,
        declared_changed_variable: experiment.changed_variable.clone(),
        actual_changed_variables: BTreeSet::from([experiment.changed_variable.clone()]),
        controls_equal: true,
        safety_status: SafetyStatus::Passed,
        leakage_status: SafetyStatus::Passed,
        reason_codes: vec![],
    };
    let temp = tempfile::tempdir()?;
    let executor = FakeExecutor;
    let service = ImprovementService::new(BenchmarkService::new(&executor));
    let summary = service.run(ImprovementRunRequest {
        experiment: &experiment,
        experiment_sha256: experiment_sha,
        equivalence: &equivalence,
        current_candidate_sha256: experiment.candidate_artifact_sha256,
        plan: &plan,
        output_root: temp.path(),
        options: BenchmarkRunOptions::default(),
    })?;
    assert_eq!(summary.benchmark.completed, 2);
    assert!(summary.all_pairs_terminal);
    Ok(())
}

#[test]
fn file_validation_binds_exact_experiment_and_candidate_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut experiment: ImprovementExperiment = serde_json::from_slice(include_bytes!(
        "../../../examples/contracts/v0.8/improvement-experiment.json"
    ))?;
    let candidate = b"bounded candidate instructions";
    experiment.candidate_artifact_sha256 = Sha256Digest::from_bytes(candidate);
    let experiment_bytes = serde_json::to_vec(&experiment)?;

    let mut equivalence: ImprovementEquivalenceResult = serde_json::from_slice(include_bytes!(
        "../../../examples/contracts/v0.8/improvement-equivalence-result.json"
    ))?;
    equivalence.experiment_id = experiment.id.clone();
    equivalence.experiment_sha256 = Sha256Digest::from_bytes(&experiment_bytes);
    equivalence.artifact_diff_sha256 = experiment.artifact_diff_sha256;
    equivalence.declared_changed_variable = experiment.changed_variable.clone();
    equivalence.actual_changed_variables = BTreeSet::from([experiment.changed_variable.clone()]);
    let equivalence_bytes = serde_json::to_vec(&equivalence)?;

    let temp = tempfile::tempdir()?;
    let experiment_path = temp.path().join("experiment.json");
    let equivalence_path = temp.path().join("equivalence.json");
    let candidate_path = temp.path().join("candidate.json");
    fs::write(&experiment_path, experiment_bytes)?;
    fs::write(&equivalence_path, equivalence_bytes)?;
    fs::write(&candidate_path, candidate)?;

    validate_improvement_inputs(&experiment_path, &equivalence_path, &candidate_path, None)?;
    fs::write(&candidate_path, b"changed candidate instructions")?;
    assert_eq!(
        validate_improvement_inputs(&experiment_path, &equivalence_path, &candidate_path, None,),
        Err(ImprovementInputError::IneligibleOrStale)
    );
    Ok(())
}
