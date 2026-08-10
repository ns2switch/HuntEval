use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::Sha256Digest;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentPartition {
    Training,
    Validation,
    HiddenTest,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionPolicy {
    pub policy_id: String,
    pub training_episode_ids: BTreeSet<String>,
    pub validation_episode_ids: BTreeSet<String>,
    pub hidden_test_episode_ids: BTreeSet<String>,
}

impl PartitionPolicy {
    pub fn validate(&self) -> Result<(), PartitionError> {
        if self.policy_id.is_empty()
            || self.policy_id.len() > 128
            || self.training_episode_ids.is_empty()
            || self.validation_episode_ids.is_empty()
            || overlaps(&self.training_episode_ids, &self.validation_episode_ids)
            || overlaps(&self.training_episode_ids, &self.hidden_test_episode_ids)
            || overlaps(&self.validation_episode_ids, &self.hidden_test_episode_ids)
            || self
                .training_episode_ids
                .iter()
                .chain(&self.validation_episode_ids)
                .chain(&self.hidden_test_episode_ids)
                .any(|id| id.is_empty() || id.len() > 128)
        {
            return Err(PartitionError::InvalidPolicy);
        }
        Ok(())
    }

    pub fn authorize_selection(
        &self,
        partition: ExperimentPartition,
    ) -> Result<SelectionAuthorization, PartitionError> {
        self.validate()?;
        match partition {
            ExperimentPartition::Training | ExperimentPartition::Validation => {
                Ok(SelectionAuthorization {
                    policy_sha256: policy_digest(self)?,
                    purpose: SelectionPurpose::CandidateSelection,
                })
            }
            ExperimentPartition::HiddenTest => Err(PartitionError::HiddenSelectionForbidden),
        }
    }

    pub fn contains(&self, partition: ExperimentPartition, episode_id: &str) -> bool {
        match partition {
            ExperimentPartition::Training => self.training_episode_ids.contains(episode_id),
            ExperimentPartition::Validation => self.validation_episode_ids.contains(episode_id),
            ExperimentPartition::HiddenTest => self.hidden_test_episode_ids.contains(episode_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionPurpose {
    CandidateSelection,
    SealedFinalAssessment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionAuthorization {
    pub policy_sha256: Sha256Digest,
    pub purpose: SelectionPurpose,
}

#[derive(Debug, Default)]
pub struct FinalAssessmentLedger {
    assessed: BTreeMap<String, Sha256Digest>,
}

impl FinalAssessmentLedger {
    pub fn authorize_once(
        &mut self,
        policy: &PartitionPolicy,
        lineage_id: &str,
        frozen_candidate: Sha256Digest,
        candidate_is_frozen: bool,
    ) -> Result<SelectionAuthorization, PartitionError> {
        policy.validate()?;
        if !candidate_is_frozen || lineage_id.is_empty() || lineage_id.len() > 128 {
            return Err(PartitionError::CandidateNotFrozen);
        }
        if self.assessed.contains_key(lineage_id) {
            return Err(PartitionError::FinalAssessmentAlreadyUsed);
        }
        self.assessed
            .insert(lineage_id.to_owned(), frozen_candidate);
        Ok(SelectionAuthorization {
            policy_sha256: policy_digest(policy)?,
            purpose: SelectionPurpose::SealedFinalAssessment,
        })
    }
}

fn overlaps(left: &BTreeSet<String>, right: &BTreeSet<String>) -> bool {
    left.iter().any(|item| right.contains(item))
}

fn policy_digest(policy: &PartitionPolicy) -> Result<Sha256Digest, PartitionError> {
    let bytes = serde_json::to_vec(&(
        &policy.policy_id,
        &policy.training_episode_ids,
        &policy.validation_episode_ids,
        &policy.hidden_test_episode_ids,
    ))
    .map_err(|_| PartitionError::InvalidPolicy)?;
    Ok(Sha256Digest::from_bytes(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PartitionError {
    #[error("evaluator-only partition policy is invalid")]
    InvalidPolicy,
    #[error("hidden-test use during candidate generation or selection is forbidden")]
    HiddenSelectionForbidden,
    #[error("candidate must be frozen before final assessment")]
    CandidateNotFrozen,
    #[error("final assessment has already been consumed for this lineage")]
    FinalAssessmentAlreadyUsed,
}
