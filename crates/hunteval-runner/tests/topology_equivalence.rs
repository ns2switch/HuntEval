use std::{collections::BTreeSet, fs, path::PathBuf};

use hunteval_domain::{
    AgentId, AgentRegistration, ControlHashes, DeploymentArchitecture, DeploymentId,
    DeploymentRegistration, EquivalenceStatus, Sha256Digest, TopologyExperiment,
    TopologyExperimentId, TopologyMetricApplicability, TopologyMetricValue,
};
use hunteval_runner::{
    ReportFormat, build_controlled_topology_analysis, evaluate_topology_equivalence,
    execute_controlled_topology_ablation, load_deployment_topology,
    registration_conforms_to_topology, render_controlled_topology_report,
};
use hunteval_statistics::{
    CalibrationPolicy, ComparisonClass, EffectSizeMethod, IntervalMethod, MultiplicityMethod,
    MultiplicityPolicy, StatisticalPolicy,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn digest(label: &str) -> Sha256Digest {
    Sha256Digest::from_bytes(label.as_bytes())
}

fn controls() -> ControlHashes {
    ControlHashes {
        episodes: digest("episodes"),
        seeds: digest("seeds"),
        budgets: digest("budgets"),
        models: digest("models"),
        managed_tool_policy: digest("tools"),
        scoring_profile: digest("scoring"),
        execution_policy: digest("execution"),
        schemas: digest("schemas"),
        binaries: digest("binaries"),
    }
}

#[test]
fn reference_topologies_are_normative_and_content_addressed()
-> Result<(), Box<dyn std::error::Error>> {
    for directory in [
        "single-agent-scripted",
        "two-agent-scripted",
        "supervisor-specialist-scripted",
        "supervisor-specialists-scripted",
    ] {
        let deployment = root().join("deployments").join(directory);
        let resolved =
            load_deployment_topology(&deployment)?.ok_or("reference topology should exist")?;
        let bytes = fs::read(deployment.join("topology.json"))?;
        assert_eq!(resolved.sha256, Sha256Digest::from_bytes(&bytes));
    }
    Ok(())
}

#[test]
fn registration_must_match_normative_agent_identity_role_and_architecture()
-> Result<(), Box<dyn std::error::Error>> {
    let resolved = load_deployment_topology(&root().join("deployments/single-agent-scripted"))?
        .ok_or("topology should exist")?;
    let registration = DeploymentRegistration {
        id: DeploymentId::new("single-agent-scripted")?,
        architecture: DeploymentArchitecture::SingleAgent,
        version: "test".to_owned(),
        agents: vec![AgentRegistration {
            id: AgentId::new("investigator")?,
            role: "investigator".to_owned(),
            capabilities: BTreeSet::from(["investigate".to_owned()]),
            prompt_version: "test".to_owned(),
            prompt_sha256: digest("prompt"),
            model: "scripted/reference".to_owned(),
            model_parameters: Default::default(),
        }],
    };
    assert!(registration_conforms_to_topology(
        &registration,
        &resolved.topology
    ));

    let mut wrong_role = registration;
    wrong_role.agents[0].role = "supervisor".to_owned();
    assert!(!registration_conforms_to_topology(
        &wrong_role,
        &resolved.topology
    ));
    Ok(())
}

#[test]
fn controlled_comparison_rejects_undeclared_changes_and_stale_hashes()
-> Result<(), Box<dyn std::error::Error>> {
    let baseline = fs::read(root().join("deployments/two-agent-scripted/topology.json"))?;
    let candidate =
        fs::read(root().join("deployments/supervisor-specialists-scripted/topology.json"))?;
    let experiment = TopologyExperiment {
        schema_version: hunteval_domain::SchemaVersion::new(0, 6),
        id: TopologyExperimentId::new("specialist-ablation")?,
        baseline_topology_sha256: Sha256Digest::from_bytes(&baseline),
        candidate_topology_sha256: Sha256Digest::from_bytes(&candidate),
        changed_variables: BTreeSet::from([
            "/agents".to_owned(),
            "/execution_pattern".to_owned(),
            "/id".to_owned(),
            "/kind".to_owned(),
            "/relationships".to_owned(),
            "/task_allocation".to_owned(),
        ]),
        control_hashes: controls(),
        paired_cell_ids: BTreeSet::from([
            format!("cell:{}", "a".repeat(64)).parse()?,
            format!("cell:{}", "b".repeat(64)).parse()?,
            format!("cell:{}", "c".repeat(64)).parse()?,
            format!("cell:{}", "d".repeat(64)).parse()?,
            format!("cell:{}", "e".repeat(64)).parse()?,
            format!("cell:{}", "f".repeat(64)).parse()?,
        ]),
    };
    let result =
        evaluate_topology_equivalence(&experiment, digest("experiment"), &baseline, &candidate)?;
    assert_eq!(result.status, EquivalenceStatus::Eligible);
    let analysis = build_controlled_topology_analysis(
        &experiment,
        &result,
        std::collections::BTreeMap::from([(
            "role_contribution".to_owned(),
            TopologyMetricValue {
                applicability: TopologyMetricApplicability::Unavailable,
                value: None,
                reason_code: Some("insufficient_controlled_samples".to_owned()),
            },
        )]),
    )?;
    assert!(analysis.topology_dependent);
    assert!(
        analysis
            .limitations
            .contains("experimental_topology_dependent")
    );
    let policy = StatisticalPolicy {
        schema_version: hunteval_domain::SchemaVersion::new(0, 6),
        id: hunteval_domain::StatisticalPolicyId::new("topology-validation")?,
        comparison_class: ComparisonClass::Validation,
        minimum_paired_samples: 3,
        confidence_level: 0.95,
        interval_method: IntervalMethod::DeterministicPairedBootstrap,
        effect_size_method: EffectSizeMethod::PairedMeanDifference,
        multiplicity: MultiplicityPolicy {
            method: MultiplicityMethod::HolmBonferroni,
            family: "topology-primary".to_owned(),
        },
        calibration: CalibrationPolicy::NotRequired,
    };
    let observations = hunteval_runner::TopologyAblationObservations {
        baseline: std::collections::BTreeMap::from([
            (
                "investigation_quality".to_owned(),
                vec![Some(0.5), Some(0.5), Some(0.5)],
            ),
            (
                "coordination_overhead".to_owned(),
                vec![Some(0.2), Some(0.2), Some(0.2)],
            ),
        ]),
        candidate: std::collections::BTreeMap::from([
            (
                "investigation_quality".to_owned(),
                vec![Some(0.8), Some(0.8), Some(0.8)],
            ),
            (
                "coordination_overhead".to_owned(),
                vec![Some(0.4), Some(0.4), Some(0.4)],
            ),
        ]),
    };
    let ablation = execute_controlled_topology_ablation(
        &experiment,
        &result,
        &policy,
        digest("statistical-policy"),
        &observations,
        7,
    )?;
    assert_eq!(ablation.analysis.metrics["role_contribution"].value, None);
    assert!(
        ablation
            .analysis
            .limitations
            .contains("multiplicity_adjusted_inference_unavailable")
    );
    assert!(
        ablation.analysis.metrics["coordination_overhead_delta"]
            .value
            .is_some_and(|value| (value - 0.2).abs() < 1e-12)
    );
    let experiment_bytes = serde_json::to_vec(&experiment)?;
    let policy_bytes = serde_json::to_vec(&policy)?;
    let observation_bytes = serde_json::to_vec(&observations)?;
    let rendered =
        render_controlled_topology_report(hunteval_runner::ControlledTopologyReportInput {
            experiment: &experiment_bytes,
            baseline_topology: &baseline,
            candidate_topology: &candidate,
            statistical_policy: &policy_bytes,
            scoring_profile: b"scoring-profile-v1",
            observations: &observation_bytes,
            seed: 7,
            format: ReportFormat::Json,
        })?;
    let report: serde_json::Value = serde_json::from_slice(&rendered)?;
    assert_eq!(report["aggregate_score"], serde_json::Value::Null);
    assert_eq!(report["constraint_first_status"], "incomparable");
    assert_eq!(report["analysis"]["topology_dependent"], true);

    let mut incomplete = experiment.clone();
    incomplete.changed_variables.remove("/agents");
    let result =
        evaluate_topology_equivalence(&incomplete, digest("experiment"), &baseline, &candidate)?;
    assert_eq!(result.status, EquivalenceStatus::Ineligible);
    assert!(
        build_controlled_topology_analysis(
            &incomplete,
            &result,
            std::collections::BTreeMap::from([(
                "role_contribution".to_owned(),
                TopologyMetricValue {
                    applicability: TopologyMetricApplicability::Unavailable,
                    value: None,
                    reason_code: Some("ineligible_experiment".to_owned()),
                },
            )]),
        )
        .is_err()
    );
    assert!(
        result
            .mismatch_reason_codes
            .contains("changed_variable_mismatch")
    );

    let mut stale = experiment;
    stale.candidate_topology_sha256 = digest("stale");
    let result =
        evaluate_topology_equivalence(&stale, digest("experiment"), &baseline, &candidate)?;
    assert_eq!(result.status, EquivalenceStatus::Ineligible);
    assert!(
        result
            .mismatch_reason_codes
            .contains("candidate_hash_mismatch")
    );
    Ok(())
}
