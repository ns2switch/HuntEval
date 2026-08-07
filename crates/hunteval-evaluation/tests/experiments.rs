use std::collections::BTreeMap;

use hunteval_domain::SchemaVersion;
use hunteval_evaluation::{
    CandidateConstraint, ExperimentManifest, ExperimentObservation, Partition, validate_candidate,
    validate_experiment_manifest,
};

fn manifest() -> ExperimentManifest {
    ExperimentManifest {
        schema_version: SchemaVersion::new(0, 1),
        id: "exp-001".into(),
        baseline_id: "baseline".into(),
        candidate_id: "candidate".into(),
        changed_variables: ["investigation_instruction".into()].into_iter().collect(),
        baseline_immutable_hashes: immutable(),
        candidate_immutable_hashes: immutable(),
        selection_partitions: [Partition::Training].into_iter().collect(),
        validation_partitions: [Partition::Validation, Partition::HiddenTest]
            .into_iter()
            .collect(),
        constraints: vec![
            CandidateConstraint::MaximumRegression {
                metric: "event_recall".into(),
                maximum: 0.02,
            },
            CandidateConstraint::MaximumVerifiedCostIncrease { maximum: 0.25 },
        ],
        human_review_required: true,
    }
}

fn immutable() -> BTreeMap<String, String> {
    [
        ("authorization".into(), "sha256:a".into()),
        ("data_handling".into(), "sha256:b".into()),
        ("tool_access".into(), "sha256:c".into()),
    ]
    .into_iter()
    .collect()
}

#[test]
fn experiments_reject_policy_diffs_hidden_selection_and_multiple_variables() {
    let mut changed_policy = manifest();
    changed_policy
        .candidate_immutable_hashes
        .insert("tool_access".into(), "sha256:changed".into());
    assert!(validate_experiment_manifest(&changed_policy).is_err());

    let mut hidden_selection = manifest();
    hidden_selection
        .selection_partitions
        .insert(Partition::HiddenTest);
    assert!(validate_experiment_manifest(&hidden_selection).is_err());

    let mut multiple_variables = manifest();
    multiple_variables.changed_variables.insert("model".into());
    assert!(validate_experiment_manifest(&multiple_variables).is_err());

    let mut authorization_change = manifest();
    authorization_change.changed_variables = ["authorization".into()].into_iter().collect();
    assert!(validate_experiment_manifest(&authorization_change).is_err());
}

#[test]
fn experiments_apply_regression_and_verified_cost_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let observation = ExperimentObservation {
        baseline_metrics: [("event_recall".into(), 0.90)].into_iter().collect(),
        candidate_metrics: [("event_recall".into(), 0.80)].into_iter().collect(),
        baseline_verified_cost: Some(1.0),
        candidate_verified_cost: Some(1.5),
    };
    let decision = validate_candidate(&manifest(), &observation)?;
    assert!(!decision.controlled_validation_passed);
    assert_eq!(decision.violations.len(), 2);
    assert!(decision.human_review_required);

    let missing_verified_cost = ExperimentObservation {
        candidate_verified_cost: None,
        ..observation
    };
    assert!(validate_candidate(&manifest(), &missing_verified_cost).is_err());
    Ok(())
}

#[test]
fn experiment_requires_human_review() {
    let mut value = manifest();
    value.human_review_required = false;
    assert!(validate_experiment_manifest(&value).is_err());
}
