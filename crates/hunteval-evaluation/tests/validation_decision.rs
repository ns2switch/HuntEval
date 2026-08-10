use std::collections::BTreeSet;

use hunteval_domain::*;
use hunteval_evaluation::{ControlledValidationInput, PairedMetricObservation, decide_candidate};

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::from_bytes(value)
}

fn policy() -> ImprovementPolicy {
    ImprovementPolicy {
        schema_version: SchemaVersion::new(0, 8),
        id: "policy".into(),
        immutable_section_classes: BTreeSet::from([
            ImmutableSectionClass::AuthorizationPolicy,
            ImmutableSectionClass::ToolAccessPolicy,
            ImmutableSectionClass::FilesystemPolicy,
            ImmutableSectionClass::NetworkPolicy,
            ImmutableSectionClass::DataHandlingPolicy,
            ImmutableSectionClass::GroundTruthIsolation,
            ImmutableSectionClass::BenchmarkConstraints,
            ImmutableSectionClass::OutputIntegrity,
            ImmutableSectionClass::SecurityControls,
        ]),
        allowed_targets: BTreeSet::from([MutableSectionClass::TaskPlanning]),
        allowed_operations: BTreeSet::from([DiffOperationKind::ReplaceSection]),
        max_artifact_bytes: 4096,
        max_growth_percent: 20,
        answer_leakage_check_required: true,
        hidden_test_feedback_during_selection: false,
        human_review_required: true,
        autonomous_adoption: false,
        constraints: vec![
            ImprovementConstraint {
                kind: ConstraintKind::MinimumMetric,
                metric: "event_recall".into(),
                threshold: 0.75,
                required_provenance: RequiredProvenance::None,
            },
            ImprovementConstraint {
                kind: ConstraintKind::MaximumRegression,
                metric: "event_recall".into(),
                threshold: 0.02,
                required_provenance: RequiredProvenance::None,
            },
        ],
    }
}

#[test]
fn raw_pairs_drive_constraint_first_decision_without_imputation()
-> Result<(), Box<dyn std::error::Error>> {
    let experiment: ImprovementExperiment = serde_json::from_slice(&std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/contracts/v0.8/improvement-experiment.json"),
    )?)?;
    let experiment_sha = digest("experiment");
    let equivalence = ImprovementEquivalenceResult {
        schema_version: SchemaVersion::new(0, 8),
        experiment_id: experiment.id.clone(),
        experiment_sha256: experiment_sha,
        artifact_diff_sha256: experiment.artifact_diff_sha256,
        status: ImprovementEquivalenceStatus::Eligible,
        declared_changed_variable: experiment.changed_variable.clone(),
        actual_changed_variables: BTreeSet::from([experiment.changed_variable.clone()]),
        controls_equal: true,
        safety_status: SafetyStatus::Passed,
        leakage_status: SafetyStatus::Passed,
        reason_codes: vec![],
    };
    let observations = vec![PairedMetricObservation {
        pair_id: "pair-1".into(),
        metric: "event_recall".into(),
        version: SchemaVersion::new(0, 3),
        baseline: Some(0.80),
        candidate: Some(0.84),
        provenance: ResourceProvenance::Measured,
    }];
    let policy = policy();
    let decision = decide_candidate(ControlledValidationInput {
        id: "decision",
        experiment: &experiment,
        experiment_sha256: experiment_sha,
        equivalence: &equivalence,
        equivalence_sha256: digest("equivalence"),
        policy: &policy,
        policy_sha256: digest("policy"),
        observations: &observations,
    })?;
    assert_eq!(decision.status, ValidationStatus::Passed);
    assert_eq!(decision.paired_samples, 1);
    let difference = decision.metric_deltas[0]
        .difference
        .ok_or("missing difference")?;
    assert!((difference - 0.04).abs() < 1e-12);

    let missing = vec![PairedMetricObservation {
        candidate: None,
        ..observations[0].clone()
    }];
    let decision = decide_candidate(ControlledValidationInput {
        id: "decision-missing",
        experiment: &experiment,
        experiment_sha256: experiment_sha,
        equivalence: &equivalence,
        equivalence_sha256: digest("equivalence"),
        policy: &policy,
        policy_sha256: digest("policy"),
        observations: &missing,
    })?;
    assert_eq!(decision.status, ValidationStatus::Failed);
    assert_eq!(decision.metric_deltas[0].baseline, None);
    Ok(())
}
