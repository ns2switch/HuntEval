use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{ImprovementContractError, require_v08, valid_id};
use crate::{SchemaVersion, Sha256Digest, UtcTimestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStatusV08 {
    Proposed,
    Testing,
    Validated,
    Rejected,
    Approved,
    Adopted,
    Invalidated,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendationEvent {
    pub schema_version: SchemaVersion,
    pub recommendation_id: String,
    pub sequence: u64,
    pub timestamp: UtcTimestamp,
    pub previous_event_sha256: Option<Sha256Digest>,
    pub event: RecommendationStatusV08,
    pub candidate_artifact_sha256: Sha256Digest,
    pub caused_by_artifact_sha256: Sha256Digest,
    pub reason_code: String,
    pub validation_decision_sha256: Option<Sha256Digest>,
    pub human_decision_sha256: Option<Sha256Digest>,
    pub adoption_record_sha256: Option<Sha256Digest>,
}

impl RecommendationEvent {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.recommendation_id)
            || !valid_id(&self.reason_code)
            || self.sequence == 0
            || (self.sequence == 1) != self.previous_event_sha256.is_none()
        {
            return Err(ImprovementContractError::InvalidTransition);
        }
        let refs_ok = match self.event {
            RecommendationStatusV08::Validated => self.validation_decision_sha256.is_some(),
            RecommendationStatusV08::Approved => {
                self.validation_decision_sha256.is_some() && self.human_decision_sha256.is_some()
            }
            RecommendationStatusV08::Adopted => {
                self.validation_decision_sha256.is_some()
                    && self.human_decision_sha256.is_some()
                    && self.adoption_record_sha256.is_some()
            }
            _ => true,
        };
        if !refs_ok {
            return Err(ImprovementContractError::InvalidTransition);
        }
        Ok(())
    }
}

pub fn project_recommendation(
    events: &[RecommendationEvent],
) -> Result<RecommendationState, ImprovementContractError> {
    if events.is_empty() {
        return Err(ImprovementContractError::InvalidTransition);
    }
    let mut previous_status = None;
    let mut previous_digest = None;
    let mut validation = None;
    let mut human = None;
    let mut adoption = None;
    let mut invalidation_reasons = BTreeSet::new();
    let recommendation_id = &events[0].recommendation_id;
    for (index, event) in events.iter().enumerate() {
        event.validate()?;
        if event.recommendation_id != *recommendation_id
            || event.sequence != u64::try_from(index + 1).unwrap_or(u64::MAX)
            || event.previous_event_sha256 != previous_digest
            || !allowed_transition(previous_status, event.event)
        {
            return Err(ImprovementContractError::InvalidTransition);
        }
        validation = event.validation_decision_sha256.or(validation);
        human = event.human_decision_sha256.or(human);
        adoption = event.adoption_record_sha256.or(adoption);
        if event.event == RecommendationStatusV08::Invalidated {
            invalidation_reasons.insert(event.reason_code.clone());
            validation = None;
            human = None;
            adoption = None;
        }
        previous_status = Some(event.event);
        previous_digest = Some(event_digest(event)?);
    }
    let last = events
        .last()
        .ok_or(ImprovementContractError::InvalidTransition)?;
    Ok(RecommendationState {
        schema_version: SchemaVersion::new(0, 8),
        recommendation_id: recommendation_id.clone(),
        last_sequence: last.sequence,
        last_event_sha256: previous_digest.ok_or(ImprovementContractError::InvalidTransition)?,
        status: last.event,
        candidate_artifact_sha256: last.candidate_artifact_sha256,
        validation_decision_sha256: validation,
        human_decision_sha256: human,
        adoption_record_sha256: adoption,
        invalidation_reasons,
    })
}

pub fn event_digest(event: &RecommendationEvent) -> Result<Sha256Digest, ImprovementContractError> {
    serde_json::to_vec(event)
        .map(Sha256Digest::from_bytes)
        .map_err(|_| ImprovementContractError::InvalidTransition)
}

const fn allowed_transition(
    previous: Option<RecommendationStatusV08>,
    next: RecommendationStatusV08,
) -> bool {
    use RecommendationStatusV08::{
        Adopted, Approved, Invalidated, Proposed, Rejected, Superseded, Testing, Validated,
    };
    matches!(
        (previous, next),
        (None, Proposed)
            | (Some(Proposed), Testing | Rejected | Superseded)
            | (Some(Testing), Validated | Rejected | Invalidated)
            | (
                Some(Validated),
                Approved | Rejected | Invalidated | Superseded
            )
            | (Some(Approved), Adopted | Invalidated)
            | (Some(Adopted), Invalidated)
            | (Some(Invalidated), Superseded)
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendationState {
    pub schema_version: SchemaVersion,
    pub recommendation_id: String,
    pub last_sequence: u64,
    pub last_event_sha256: Sha256Digest,
    pub status: RecommendationStatusV08,
    pub candidate_artifact_sha256: Sha256Digest,
    pub validation_decision_sha256: Option<Sha256Digest>,
    pub human_decision_sha256: Option<Sha256Digest>,
    pub adoption_record_sha256: Option<Sha256Digest>,
    pub invalidation_reasons: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approve,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanDecision {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub recommendation_id: String,
    pub candidate_artifact_sha256: Sha256Digest,
    pub experiment_sha256: Sha256Digest,
    pub validation_decision_sha256: Sha256Digest,
    pub improvement_policy_sha256: Sha256Digest,
    pub reviewer_id: String,
    pub reviewed_at: UtcTimestamp,
    pub decision: ReviewDecision,
    pub reason_codes: BTreeSet<String>,
    pub explicit_confirmation: bool,
}

impl HumanDecision {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id)
            || !valid_id(&self.recommendation_id)
            || !valid_id(&self.reviewer_id)
            || self.reason_codes.is_empty()
            || !self.explicit_confirmation
        {
            return Err(ImprovementContractError::InvalidApproval);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionRecord {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub recommendation_id: String,
    pub candidate_artifact_sha256: Sha256Digest,
    pub human_decision_sha256: Sha256Digest,
    pub adopted_deployment_sha256: Sha256Digest,
    pub actor_id: String,
    pub adopted_at: UtcTimestamp,
    pub external_adoption_confirmed: bool,
}

impl AdoptionRecord {
    pub fn validate(&self) -> Result<(), ImprovementContractError> {
        require_v08(self.schema_version)?;
        if !valid_id(&self.id)
            || !valid_id(&self.recommendation_id)
            || !valid_id(&self.actor_id)
            || !self.external_adoption_confirmed
        {
            return Err(ImprovementContractError::InvalidApproval);
        }
        Ok(())
    }
}
