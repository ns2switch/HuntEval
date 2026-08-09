use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ContractValidationError, DatasetReviewId, EpisodeId, ReviewerId, SchemaVersion, Sha256Digest,
    UtcTimestamp,
};

const SCIENCE_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 6);
const MAX_CAPABILITIES: usize = 32;
const MAX_SHAPES: usize = 8;
const MAX_REASON_CODES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeDifficulty {
    Introductory,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeCapability {
    IdentityAnalysis,
    TimelineReconstruction,
    CrossBoundaryCorrelation,
    BenignDisambiguation,
    AttackPathAnalysis,
    EvidenceCorrelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationShape {
    SingleStage,
    MultiStage,
    CrossBoundary,
    AmbiguousAlternative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeClassification {
    pub schema_version: SchemaVersion,
    pub episode_id: EpisodeId,
    pub difficulty: EpisodeDifficulty,
    pub capabilities: BTreeSet<EpisodeCapability>,
    pub investigation_shapes: BTreeSet<InvestigationShape>,
}

impl EpisodeClassification {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        if self.capabilities.is_empty() || self.capabilities.len() > MAX_CAPABILITIES {
            return Err(ContractValidationError::new(
                "capabilities",
                "capability count is outside the supported bound",
            ));
        }
        if self.investigation_shapes.is_empty() || self.investigation_shapes.len() > MAX_SHAPES {
            return Err(ContractValidationError::new(
                "investigation_shapes",
                "investigation shape count is outside the supported bound",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetReviewStatus {
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetReviewRecord {
    pub schema_version: SchemaVersion,
    pub review_id: DatasetReviewId,
    pub episode_id: EpisodeId,
    pub reviewer_id: ReviewerId,
    pub reviewed_at: UtcTimestamp,
    pub status: DatasetReviewStatus,
    pub public_package_sha256: Sha256Digest,
    pub private_ground_truth_sha256: Sha256Digest,
    pub reference_query_sha256: Sha256Digest,
    pub review_policy_sha256: Sha256Digest,
    pub reason_codes: BTreeSet<String>,
}

impl DatasetReviewRecord {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        if self.reason_codes.len() > MAX_REASON_CODES
            || self
                .reason_codes
                .iter()
                .any(|code| !valid_reason_code(code))
        {
            return Err(ContractValidationError::new(
                "reason_codes",
                "reason codes are malformed or exceed the supported bound",
            ));
        }
        if self.status == DatasetReviewStatus::Approved && !self.reason_codes.is_empty() {
            return Err(ContractValidationError::new(
                "reason_codes",
                "approved reviews cannot contain rejection reasons",
            ));
        }
        if self.status == DatasetReviewStatus::Rejected && self.reason_codes.is_empty() {
            return Err(ContractValidationError::new(
                "reason_codes",
                "rejected reviews require at least one reason",
            ));
        }
        Ok(())
    }
}

fn require_version(version: SchemaVersion) -> Result<(), ContractValidationError> {
    if version != SCIENCE_SCHEMA_VERSION {
        return Err(ContractValidationError::new(
            "schema_version",
            "schema version is unsupported",
        ));
    }
    Ok(())
}

fn valid_reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}
