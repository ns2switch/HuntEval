use std::collections::BTreeSet;

use hunteval_domain::{
    AdoptionRecord, ControlledValidationDecision, HumanDecision, ReviewDecision, SchemaVersion,
    Sha256Digest, UtcTimestamp, ValidationStatus,
};
use hunteval_evaluation::{verify_external_adoption, verify_human_decision};

#[test]
fn passing_validation_review_and_external_adoption_are_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    let candidate = Sha256Digest::from_bytes(b"candidate");
    let experiment = Sha256Digest::from_bytes(b"experiment");
    let policy = Sha256Digest::from_bytes(b"policy");
    let validation = ControlledValidationDecision {
        schema_version: SchemaVersion::new(0, 8),
        id: "validation".into(),
        experiment_id: "experiment".into(),
        experiment_sha256: experiment,
        equivalence_sha256: Sha256Digest::from_bytes(b"equivalence"),
        improvement_policy_sha256: policy,
        status: ValidationStatus::Passed,
        paired_samples: 2,
        missing_pairs: vec![],
        metric_deltas: vec![],
        constraints: vec![],
        hidden_test_used_in_selection: false,
        human_review_required: true,
        limitations: BTreeSet::from(["topology_dependent".into()]),
    };
    let validation_sha = Sha256Digest::from_bytes(serde_json::to_vec(&validation)?);
    let human = HumanDecision {
        schema_version: SchemaVersion::new(0, 8),
        id: "review".into(),
        recommendation_id: "recommendation".into(),
        candidate_artifact_sha256: candidate,
        experiment_sha256: experiment,
        validation_decision_sha256: validation_sha,
        improvement_policy_sha256: policy,
        reviewer_id: "reviewer".into(),
        reviewed_at: "2026-08-10T13:00:00Z".parse::<UtcTimestamp>()?,
        decision: ReviewDecision::Approve,
        reason_codes: BTreeSet::from(["controlled_validation_reviewed".into()]),
        explicit_confirmation: true,
    };
    let human_sha = Sha256Digest::from_bytes(serde_json::to_vec(&human)?);
    verify_human_decision(&human, human_sha, &validation, validation_sha)?;
    let adoption = AdoptionRecord {
        schema_version: SchemaVersion::new(0, 8),
        id: "adoption".into(),
        recommendation_id: "recommendation".into(),
        candidate_artifact_sha256: candidate,
        human_decision_sha256: human_sha,
        adopted_deployment_sha256: Sha256Digest::from_bytes(b"external-deployment"),
        actor_id: "maintainer".into(),
        adopted_at: "2026-08-10T14:00:00Z".parse::<UtcTimestamp>()?,
        external_adoption_confirmed: true,
    };
    verify_external_adoption(&adoption, &human, human_sha)?;
    Ok(())
}
