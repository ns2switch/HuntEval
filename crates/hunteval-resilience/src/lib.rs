//! Seeded faults applied at logical trajectory boundaries, not timing races.

use hunteval_domain::{Applicability, MetricDirection, MetricRange, MetricValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultKind {
    AgentTimeout,
    MalformedResponse,
    WorkerFailure,
    UnavailableAgent,
    NoisyAgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultProfile {
    pub id: String,
    pub seed: u64,
    pub faults: Vec<FaultKind>,
    pub retry_budget: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultEvent {
    pub logical_sequence: u64,
    pub kind: FaultKind,
    pub attempt: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaultSchedule {
    pub seed: u64,
    pub events: Vec<FaultEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryOutcome {
    pub recovered: bool,
    pub retries_used: u32,
    pub reassigned: bool,
    pub reason_code: String,
}

pub fn schedule(
    profile: &FaultProfile,
    logical_boundaries: u64,
) -> Result<FaultSchedule, ResilienceError> {
    if profile.id.trim().is_empty() || logical_boundaries == 0 {
        return Err(ResilienceError::InvalidProfile);
    }
    let mut state = profile.seed.max(1);
    let events = profile
        .faults
        .iter()
        .enumerate()
        .map(|(index, kind)| {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            FaultEvent {
                logical_sequence: 1 + state % logical_boundaries,
                kind: *kind,
                attempt: index as u32 + 1,
            }
        })
        .collect();
    Ok(FaultSchedule {
        seed: profile.seed,
        events,
    })
}

/// Apply the fixed recovery policy to an injected logical fault.
#[must_use]
pub fn recover(event: &FaultEvent, retry_budget: u32, alternate_agents: u32) -> RecoveryOutcome {
    let retryable = matches!(
        event.kind,
        FaultKind::AgentTimeout | FaultKind::MalformedResponse | FaultKind::WorkerFailure
    );
    let reassigned = event.kind == FaultKind::UnavailableAgent && alternate_agents > 0;
    let retries_used = u32::from(retryable && retry_budget > 0);
    let recovered = retries_used > 0 || reassigned || event.kind == FaultKind::NoisyAgent;
    let reason_code = match (event.kind, recovered) {
        (FaultKind::UnavailableAgent, true) => "task_reassigned",
        (FaultKind::NoisyAgent, true) => "untrusted_output_discarded",
        (_, true) => "retry_succeeded",
        (_, false) => "recovery_budget_exhausted",
    };
    RecoveryOutcome {
        recovered,
        retries_used,
        reassigned,
        reason_code: reason_code.into(),
    }
}

/// Graceful-degradation ratio: faulted quality divided by paired baseline quality.
pub fn graceful_degradation(baseline: f64, faulted: f64) -> Result<MetricValue, ResilienceError> {
    if !baseline.is_finite()
        || !faulted.is_finite()
        || !(0.0..=1.0).contains(&baseline)
        || !(0.0..=1.0).contains(&faulted)
    {
        return Err(ResilienceError::InvalidScore);
    }
    if baseline == 0.0 {
        return Ok(MetricValue {
            value: None,
            applicability: Applicability::ZeroDenominator,
            direction: MetricDirection::HigherIsBetter,
            range: MetricRange {
                minimum: 0.0,
                maximum: 1.0,
            },
            numerator: None,
            denominator: None,
        });
    }
    Ok(MetricValue {
        value: Some((faulted / baseline).min(1.0)),
        applicability: Applicability::Applicable,
        direction: MetricDirection::HigherIsBetter,
        range: MetricRange {
            minimum: 0.0,
            maximum: 1.0,
        },
        numerator: None,
        denominator: None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ResilienceError {
    #[error("fault profile requires an identifier and logical boundary")]
    InvalidProfile,
    #[error("paired resilience scores must be finite and within zero and one")]
    InvalidScore,
}
