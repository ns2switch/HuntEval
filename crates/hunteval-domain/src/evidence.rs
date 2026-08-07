use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{ActionId, ContractValidationError, EventId, EvidenceId, FindingId, UtcTimestamp};

/// Finite confidence value in the inclusive range `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Confidence(f64);

impl Confidence {
    /// Creates a bounded confidence value.
    pub fn new(value: f64) -> Result<Self, ContractValidationError> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(ContractValidationError::new(
                "confidence",
                "confidence must be finite and within zero and one",
            ));
        }
        Ok(Self(value))
    }

    /// Returns the validated numeric value.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// Inclusive observed time range associated with evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeRange {
    pub start: UtcTimestamp,
    pub end: UtcTimestamp,
}

impl TimeRange {
    /// Validates chronological ordering.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.start > self.end {
            return Err(ContractValidationError::new(
                "time_range",
                "start must not be after end",
            ));
        }
        Ok(())
    }
}

/// Agent-produced evidence grounded in HuntEval-issued action results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: EvidenceId,
    pub summary: String,
    pub source_action_ids: BTreeSet<ActionId>,
    pub event_ids: BTreeSet<EventId>,
    pub entity_ids: BTreeSet<String>,
    pub time_range: Option<TimeRange>,
    pub confidence: Confidence,
}

impl Evidence {
    /// Validates locally decidable evidence requirements.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.summary.trim().is_empty() || self.summary.len() > 16_384 {
            return Err(ContractValidationError::new(
                "evidence.summary",
                "summary must not be empty",
            ));
        }
        if self.source_action_ids.is_empty() {
            return Err(ContractValidationError::new(
                "evidence.source_action_ids",
                "at least one source action is required",
            ));
        }
        if let Some(time_range) = self.time_range {
            time_range.validate()?;
        }
        Ok(())
    }
}

/// Severity assigned to a proposed finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

/// Threat-hunting conclusion that retains evidence and event provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: FindingId,
    pub title: String,
    pub severity: FindingSeverity,
    pub evidence_ids: BTreeSet<EvidenceId>,
    pub event_ids: BTreeSet<EventId>,
    pub entity_ids: BTreeSet<String>,
    pub attack_techniques: BTreeSet<String>,
    pub benign_alternatives: Vec<String>,
    pub confidence: Confidence,
}

impl Finding {
    /// Validates the minimum evidence-bearing finding shape.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.title.trim().is_empty() || self.evidence_ids.is_empty() {
            return Err(ContractValidationError::new(
                "finding",
                "title and at least one evidence reference are required",
            ));
        }
        if self
            .benign_alternatives
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(ContractValidationError::new(
                "finding.benign_alternatives",
                "benign alternatives must not be empty",
            ));
        }
        Ok(())
    }
}

/// High-level conclusion submitted by the deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
    ConfirmedMaliciousActivity,
    SuspiciousActivity,
    NoMaliciousActivity,
    Inconclusive,
}

/// Final structured output evaluated against private ground truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalSubmission {
    pub status: SubmissionStatus,
    pub summary: String,
    pub finding_ids: BTreeSet<FindingId>,
    pub malicious_event_ids: BTreeSet<EventId>,
    pub malicious_entity_ids: BTreeSet<String>,
    pub attack_path: Vec<EventId>,
    pub attack_techniques: BTreeSet<String>,
    pub confidence: Confidence,
    #[serde(default)]
    pub limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Vec<TimelineEntry>>,
}

impl FinalSubmission {
    /// Validates locally decidable final submission requirements.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.summary.trim().is_empty() {
            return Err(ContractValidationError::new(
                "submission.summary",
                "summary must not be empty",
            ));
        }
        let malicious = matches!(
            self.status,
            SubmissionStatus::ConfirmedMaliciousActivity | SubmissionStatus::SuspiciousActivity
        );
        if malicious && self.finding_ids.is_empty() {
            return Err(ContractValidationError::new(
                "submission.finding_ids",
                "malicious submissions require a finding",
            ));
        }
        if self.finding_ids.len() > 10_000
            || self.malicious_event_ids.len() > 100_000
            || self.malicious_entity_ids.len() > 100_000
            || self.attack_path.len() > 100_000
            || self.attack_techniques.len() > 10_000
            || self.limitations.len() > 1_024
            || self.limitations.iter().any(|value| value.len() > 4_096)
            || self
                .attack_techniques
                .iter()
                .any(|value| value.is_empty() || value.len() > 128)
        {
            return Err(ContractValidationError::new(
                "submission",
                "submission collection or text limit exceeded",
            ));
        }
        if self
            .timeline
            .as_ref()
            .is_some_and(|timeline| timeline.len() > 100_000)
        {
            return Err(ContractValidationError::new(
                "submission.timeline",
                "timeline entry limit exceeded",
            ));
        }
        let mut timeline_events = BTreeSet::new();
        for entry in self.timeline.iter().flatten() {
            entry.validate()?;
            if !timeline_events.insert(&entry.event_id) {
                return Err(ContractValidationError::new(
                    "submission.timeline.event_id",
                    "event identifiers must be unique",
                ));
            }
        }
        Ok(())
    }
}

/// One deployment-submitted, evidence-linked timeline observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimelineEntry {
    pub event_id: EventId,
    pub observed_at: UtcTimestamp,
    pub summary: String,
    pub evidence_ids: BTreeSet<EvidenceId>,
}

impl TimelineEntry {
    /// Validates locally decidable timeline fields.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.summary.trim().is_empty()
            || self.summary.len() > 4_096
            || self.evidence_ids.len() > 1_024
        {
            return Err(ContractValidationError::new(
                "submission.timeline.summary",
                "summary must not be empty",
            ));
        }
        Ok(())
    }
}

/// Versioned artifact adapter for the standalone v0.4 submission contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalSubmissionArtifact {
    pub schema_version: crate::SchemaVersion,
    #[serde(flatten)]
    pub submission: FinalSubmission,
}

impl FinalSubmissionArtifact {
    /// Normalizes a validated v0.4 artifact into the current in-memory contract.
    pub fn into_submission(self) -> Result<FinalSubmission, ContractValidationError> {
        if self.schema_version != crate::SchemaVersion::new(0, 4)
            || self.submission.timeline.is_none()
        {
            return Err(ContractValidationError::new(
                "submission.schema_version",
                "a v0.4 submission with a timeline field is required",
            ));
        }
        self.submission.validate()?;
        Ok(self.submission)
    }
}
