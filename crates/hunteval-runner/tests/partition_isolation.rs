use std::collections::BTreeSet;

use hunteval_domain::Sha256Digest;
use hunteval_runner::{
    ExperimentPartition, FinalAssessmentLedger, PartitionError, PartitionPolicy,
};

fn policy() -> PartitionPolicy {
    PartitionPolicy {
        policy_id: "partition-v1".into(),
        training_episode_ids: BTreeSet::from(["episode-training".into()]),
        validation_episode_ids: BTreeSet::from(["episode-validation".into()]),
        hidden_test_episode_ids: BTreeSet::from(["episode-hidden".into()]),
    }
}

#[test]
fn hidden_membership_cannot_authorize_candidate_selection() -> Result<(), Box<dyn std::error::Error>>
{
    let policy = policy();
    policy.authorize_selection(ExperimentPartition::Training)?;
    policy.authorize_selection(ExperimentPartition::Validation)?;
    assert_eq!(
        policy.authorize_selection(ExperimentPartition::HiddenTest),
        Err(PartitionError::HiddenSelectionForbidden)
    );
    Ok(())
}

#[test]
fn final_assessment_is_single_use_and_digest_bound() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = FinalAssessmentLedger::default();
    let candidate = Sha256Digest::from_bytes(b"candidate");
    ledger.authorize_once(&policy(), "lineage", candidate, true)?;
    assert_eq!(
        ledger.authorize_once(&policy(), "lineage", candidate, true),
        Err(PartitionError::FinalAssessmentAlreadyUsed)
    );
    Ok(())
}
