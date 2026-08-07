use std::collections::BTreeMap;

use hunteval_domain::{Applicability, EventId, MetricDirection, MetricValue};

use crate::{EvaluationError, sets};

const MAX_MATCHING_PAIRS: usize = 4_000_000;

pub(super) fn evaluate(
    submitted: &[EventId],
    expected: &[EventId],
    benign: bool,
) -> Result<(MetricValue, MetricValue), EvaluationError> {
    let matched = longest_common_subsequence(submitted, expected)? as u64;
    let precision = if submitted.is_empty() {
        empty_precision(expected.is_empty(), benign)
    } else {
        sets::ratio(
            matched,
            submitted.len() as u64,
            MetricDirection::HigherIsBetter,
        )
    };
    let recall = if expected.is_empty() {
        if benign && submitted.is_empty() {
            exact_empty_match()
        } else {
            sets::unavailable(Applicability::NotRequired, MetricDirection::HigherIsBetter)
        }
    } else {
        sets::ratio(
            matched,
            expected.len() as u64,
            MetricDirection::HigherIsBetter,
        )
    };
    Ok((precision, recall))
}

fn empty_precision(expected_empty: bool, benign: bool) -> MetricValue {
    if expected_empty && benign {
        exact_empty_match()
    } else if expected_empty {
        sets::unavailable(Applicability::NotRequired, MetricDirection::HigherIsBetter)
    } else {
        MetricValue {
            value: Some(0.0),
            applicability: Applicability::Applicable,
            direction: MetricDirection::HigherIsBetter,
            range: sets::unit_range(),
            numerator: Some(0),
            denominator: Some(0),
        }
    }
}

fn exact_empty_match() -> MetricValue {
    MetricValue {
        value: Some(1.0),
        applicability: Applicability::Applicable,
        direction: MetricDirection::HigherIsBetter,
        range: sets::unit_range(),
        numerator: Some(0),
        denominator: Some(0),
    }
}

fn longest_common_subsequence(
    left: &[EventId],
    right: &[EventId],
) -> Result<usize, EvaluationError> {
    let mut positions = BTreeMap::<&EventId, Vec<usize>>::new();
    for (index, event) in right.iter().enumerate() {
        positions.entry(event).or_default().push(index);
    }
    let mut matching_pairs = 0_usize;
    let mut tails = Vec::<usize>::new();
    for event in left {
        if let Some(indices) = positions.get(event) {
            matching_pairs = matching_pairs
                .checked_add(indices.len())
                .ok_or(EvaluationError::AttackPathComparisonTooLarge)?;
            if matching_pairs > MAX_MATCHING_PAIRS {
                return Err(EvaluationError::AttackPathComparisonTooLarge);
            }
            for &index in indices.iter().rev() {
                let insertion = tails.partition_point(|tail| *tail < index);
                if insertion == tails.len() {
                    tails.push(index);
                } else {
                    tails[insertion] = index;
                }
            }
        }
    }
    Ok(tails.len())
}
