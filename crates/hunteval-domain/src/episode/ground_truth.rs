use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    ContractValidationError, EpisodeId, EventId, SchemaVersion, SubmissionStatus, UtcTimestamp,
};

/// Trusted evaluator-only ground truth loaded from the private episode root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundTruth {
    pub schema_version: SchemaVersion,
    pub episode_id: EpisodeId,
    #[serde(default)]
    pub malicious_event_ids: BTreeSet<EventId>,
    #[serde(default)]
    pub malicious_entity_ids: BTreeSet<String>,
    #[serde(default)]
    pub expected_attack_path: Vec<EventId>,
    #[serde(default)]
    pub expected_attack_techniques: BTreeSet<String>,
    #[serde(default)]
    pub acceptable_conclusions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptable_submission_statuses: Option<BTreeSet<SubmissionStatus>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_timeline_windows: Option<Vec<ExpectedTimelineWindow>>,
    pub minimum_evidence_items: u32,
}

/// Evaluator-only acceptable UTC interval for one expected event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedTimelineWindow {
    pub event_id: EventId,
    pub earliest: UtcTimestamp,
    pub latest: UtcTimestamp,
}

impl ExpectedTimelineWindow {
    /// Validates chronological ordering of the inclusive interval.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.earliest > self.latest {
            return Err(ContractValidationError::new(
                "expected_timeline_windows",
                "earliest must not be after latest",
            ));
        }
        Ok(())
    }
}

impl GroundTruth {
    /// Returns whether the private truth declares an explicitly empty scored case.
    #[must_use]
    pub fn is_benign_scored_episode(&self) -> bool {
        self.malicious_event_ids.is_empty()
            && self.malicious_entity_ids.is_empty()
            && self.expected_attack_path.is_empty()
            && self.expected_attack_techniques.is_empty()
    }

    /// Validates the explicitly supported v0.3 and v0.4 normalized forms.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        const V03: SchemaVersion = SchemaVersion::new(0, 3);
        const V04: SchemaVersion = SchemaVersion::new(0, 4);
        if self.schema_version != V03 && self.schema_version != V04 {
            return Err(invalid("schema version is unsupported"));
        }
        self.validate_limits()?;
        if self.schema_version == V03
            && (self.acceptable_submission_statuses.is_some()
                || self.expected_timeline_windows.is_some())
        {
            return Err(invalid("v0.3 cannot contain v0.4 evaluation fields"));
        }
        if self.schema_version == V04
            && (self.acceptable_conclusions.is_empty()
                || self
                    .acceptable_submission_statuses
                    .as_ref()
                    .is_none_or(BTreeSet::is_empty)
                || self.expected_timeline_windows.is_none())
        {
            return Err(invalid(
                "v0.4 requires acceptable statuses and timeline windows",
            ));
        }
        self.validate_windows()
    }

    fn validate_limits(&self) -> Result<(), ContractValidationError> {
        if self.malicious_event_ids.len() > 100_000
            || self.malicious_entity_ids.len() > 100_000
            || self.expected_attack_path.len() > 100_000
            || self.expected_attack_techniques.len() > 10_000
            || self.acceptable_conclusions.len() > 1_024
            || self.minimum_evidence_items > 100_000
            || self
                .acceptable_conclusions
                .iter()
                .any(|value| value.is_empty() || value.len() > 4_096)
            || self
                .expected_attack_techniques
                .iter()
                .any(|value| value.is_empty() || value.len() > 128)
        {
            return Err(invalid("ground-truth collection or text limit exceeded"));
        }
        Ok(())
    }

    fn validate_windows(&self) -> Result<(), ContractValidationError> {
        if self
            .expected_timeline_windows
            .as_ref()
            .is_some_and(|windows| windows.len() > 100_000)
        {
            return Err(invalid("timeline window limit exceeded"));
        }
        let mut events = BTreeSet::new();
        for window in self.expected_timeline_windows.iter().flatten() {
            window.validate()?;
            if !self.malicious_event_ids.contains(&window.event_id)
                || !events.insert(&window.event_id)
            {
                return Err(invalid("timeline event must be unique and malicious"));
            }
        }
        Ok(())
    }
}

fn invalid(message: &'static str) -> ContractValidationError {
    ContractValidationError::new("ground_truth", message)
}
