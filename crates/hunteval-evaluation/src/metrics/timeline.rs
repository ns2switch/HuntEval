use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    Applicability, EventId, ExpectedTimelineWindow, MetricDirection, MetricValue, TimelineEntry,
};

use crate::{EvaluationError, sets};

pub(super) fn evaluate(
    submitted: Option<&[TimelineEntry]>,
    expected: Option<&[ExpectedTimelineWindow]>,
    benign: bool,
) -> Result<(MetricValue, MetricValue), EvaluationError> {
    let Some(submitted) = submitted else {
        return Ok((
            unavailable(Applicability::TimelineNotSubmitted),
            unavailable(if expected.is_some() {
                Applicability::TimelineNotSubmitted
            } else {
                Applicability::TimelineTruthUnavailable
            }),
        ));
    };
    let Some(expected) = expected else {
        return Ok((
            unavailable(Applicability::TimelineTruthUnavailable),
            unavailable(Applicability::TimelineTruthUnavailable),
        ));
    };
    require_unique(submitted.iter().map(|entry| &entry.event_id))?;
    require_unique(expected.iter().map(|window| &window.event_id))?;
    let windows: BTreeMap<_, _> = expected
        .iter()
        .map(|window| (&window.event_id, window))
        .collect();
    let matched = submitted
        .iter()
        .filter(|entry| {
            windows.get(&entry.event_id).is_some_and(|window| {
                entry.observed_at >= window.earliest && entry.observed_at <= window.latest
            })
        })
        .count() as u64;
    Ok((
        precision(matched, submitted.len(), expected.is_empty(), benign),
        recall(matched, expected.len(), submitted.is_empty(), benign),
    ))
}

fn precision(matched: u64, submitted: usize, expected_empty: bool, benign: bool) -> MetricValue {
    if submitted == 0 && expected_empty {
        if benign {
            return empty_match();
        }
        return unavailable(Applicability::NotRequired);
    }
    if submitted == 0 {
        return explicit_zero();
    }
    sets::ratio(matched, submitted as u64, MetricDirection::HigherIsBetter)
}

fn recall(matched: u64, expected: usize, submitted_empty: bool, benign: bool) -> MetricValue {
    if expected == 0 {
        if benign && submitted_empty {
            return empty_match();
        }
        return unavailable(Applicability::NotRequired);
    }
    sets::ratio(matched, expected as u64, MetricDirection::HigherIsBetter)
}

fn require_unique<'a>(
    mut events: impl Iterator<Item = &'a EventId>,
) -> Result<(), EvaluationError> {
    let mut seen = BTreeSet::new();
    if events.any(|event| !seen.insert(event)) {
        return Err(EvaluationError::DuplicateTimelineEvent);
    }
    Ok(())
}

fn explicit_zero() -> MetricValue {
    explicit_value(0.0)
}

fn empty_match() -> MetricValue {
    explicit_value(1.0)
}

fn explicit_value(value: f64) -> MetricValue {
    MetricValue {
        value: Some(value),
        applicability: Applicability::Applicable,
        direction: MetricDirection::HigherIsBetter,
        range: sets::unit_range(),
        numerator: Some(0),
        denominator: Some(0),
    }
}

fn unavailable(applicability: Applicability) -> MetricValue {
    sets::unavailable(applicability, MetricDirection::HigherIsBetter)
}
