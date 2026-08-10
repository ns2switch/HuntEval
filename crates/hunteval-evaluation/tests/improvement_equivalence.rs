use std::collections::BTreeSet;

use hunteval_domain::*;
use hunteval_evaluation::{CandidateSafetyResult, evaluate_improvement_equivalence};

fn digest(value: &str) -> Sha256Digest {
    Sha256Digest::from_bytes(value)
}

#[test]
fn equivalence_requires_exact_bound_diff_and_safe_candidate() {
    let diff = ArtifactDiff {
        schema_version: SchemaVersion::new(0, 8),
        id: "diff".into(),
        baseline_artifact_sha256: digest("baseline"),
        candidate_artifact_sha256: digest("candidate"),
        changed_variable: "instruction".into(),
        operations: vec![],
        immutable_policy_status: SafetyStatus::Passed,
        reason_codes: vec![],
    };
    let diff_sha256 = digest("diff");
    let experiment = ImprovementExperiment {
        schema_version: SchemaVersion::new(0, 8),
        id: "experiment".into(),
        lineage_id: "lineage".into(),
        baseline_artifact_sha256: digest("baseline"),
        candidate_artifact_sha256: digest("candidate"),
        artifact_diff_sha256: diff_sha256,
        improvement_policy_sha256: digest("policy"),
        partition_policy_sha256: digest("partition"),
        scoring_profile_sha256: digest("scoring"),
        statistical_policy_sha256: digest("statistics"),
        changed_variable: "instruction".into(),
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
            baseline_cell_id: "cell:baseline".into(),
            candidate_cell_id: "cell:candidate".into(),
        }],
        candidate_frozen: true,
    };
    let safe = CandidateSafetyResult {
        safety_status: SafetyStatus::Passed,
        leakage_status: SafetyStatus::Passed,
        reason_codes: vec![],
    };
    let result = evaluate_improvement_equivalence(
        &experiment,
        digest("experiment"),
        &diff,
        diff_sha256,
        &safe,
    );
    assert_eq!(result.status, ImprovementEquivalenceStatus::Eligible);
    assert_eq!(
        result.actual_changed_variables,
        BTreeSet::from(["instruction".into()])
    );

    let rejected = CandidateSafetyResult {
        leakage_status: SafetyStatus::Rejected,
        ..safe
    };
    let result = evaluate_improvement_equivalence(
        &experiment,
        digest("experiment"),
        &diff,
        diff_sha256,
        &rejected,
    );
    assert_eq!(result.status, ImprovementEquivalenceStatus::Ineligible);
}
