use std::collections::BTreeSet;

use hunteval_domain::{
    ArtifactDiff, ImprovementEquivalenceResult, ImprovementEquivalenceStatus,
    ImprovementExperiment, SafetyStatus, SchemaVersion, Sha256Digest,
};

use super::CandidateSafetyResult;

pub fn evaluate_improvement_equivalence(
    experiment: &ImprovementExperiment,
    experiment_sha256: Sha256Digest,
    diff: &ArtifactDiff,
    diff_sha256: Sha256Digest,
    safety: &CandidateSafetyResult,
) -> ImprovementEquivalenceResult {
    let mut reasons = Vec::new();
    if experiment.validate().is_err() {
        reasons.push("invalid_experiment".to_owned());
    }
    if experiment.artifact_diff_sha256 != diff_sha256
        || experiment.baseline_artifact_sha256 != diff.baseline_artifact_sha256
        || experiment.candidate_artifact_sha256 != diff.candidate_artifact_sha256
    {
        reasons.push("stale_artifact_diff".to_owned());
    }
    if experiment.changed_variable != diff.changed_variable {
        reasons.push("changed_variable_mismatch".to_owned());
    }
    if safety.safety_status != SafetyStatus::Passed {
        reasons.push("candidate_safety_rejected".to_owned());
    }
    if safety.leakage_status != SafetyStatus::Passed {
        reasons.push("candidate_leakage_rejected".to_owned());
    }
    reasons.sort();
    reasons.dedup();
    let actual_changed_variables = BTreeSet::from([diff.changed_variable.clone()]);
    ImprovementEquivalenceResult {
        schema_version: SchemaVersion::new(0, 8),
        experiment_id: experiment.id.clone(),
        experiment_sha256,
        artifact_diff_sha256: diff_sha256,
        status: if reasons.is_empty() {
            ImprovementEquivalenceStatus::Eligible
        } else {
            ImprovementEquivalenceStatus::Ineligible
        },
        declared_changed_variable: experiment.changed_variable.clone(),
        actual_changed_variables,
        controls_equal: true,
        safety_status: safety.safety_status,
        leakage_status: safety.leakage_status,
        reason_codes: reasons,
    }
}
