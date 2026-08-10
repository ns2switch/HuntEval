use hunteval_domain::{
    AdoptionRecord, ControlledValidationDecision, HumanDecision, ReviewDecision, Sha256Digest,
    ValidationStatus,
};
use thiserror::Error;

pub fn verify_human_decision(
    decision: &HumanDecision,
    decision_sha256: Sha256Digest,
    validation: &ControlledValidationDecision,
    validation_sha256: Sha256Digest,
) -> Result<(), ReviewError> {
    decision
        .validate()
        .map_err(|_| ReviewError::InvalidReview)?;
    if decision.validation_decision_sha256 != validation_sha256
        || decision.experiment_sha256 != validation.experiment_sha256
        || decision.improvement_policy_sha256 != validation.improvement_policy_sha256
        || (decision.decision == ReviewDecision::Approve
            && validation.status != ValidationStatus::Passed)
        || Sha256Digest::from_bytes(
            serde_json::to_vec(decision).map_err(|_| ReviewError::InvalidReview)?,
        ) != decision_sha256
    {
        return Err(ReviewError::InvalidReview);
    }
    Ok(())
}

pub fn verify_external_adoption(
    adoption: &AdoptionRecord,
    human: &HumanDecision,
    human_sha256: Sha256Digest,
) -> Result<(), ReviewError> {
    adoption
        .validate()
        .map_err(|_| ReviewError::InvalidAdoption)?;
    if human.decision != ReviewDecision::Approve
        || adoption.human_decision_sha256 != human_sha256
        || adoption.recommendation_id != human.recommendation_id
        || adoption.candidate_artifact_sha256 != human.candidate_artifact_sha256
    {
        return Err(ReviewError::InvalidAdoption);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReviewError {
    #[error("human decision is not an explicit review of the exact passing validation")]
    InvalidReview,
    #[error("adoption is not an externally confirmed action for the exact approved candidate")]
    InvalidAdoption,
}
