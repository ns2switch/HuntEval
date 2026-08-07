use std::collections::BTreeSet;

use hunteval_domain::{Applicability, MetricDirection, MetricRange, MetricValue};

pub(crate) fn precision<T: Ord>(
    submitted: &BTreeSet<T>,
    truth: &BTreeSet<T>,
    benign: bool,
) -> MetricValue {
    if submitted.is_empty() {
        if truth.is_empty() && benign {
            return ratio(1, 1, MetricDirection::HigherIsBetter);
        }
        if truth.is_empty() {
            return unavailable(Applicability::NotRequired, MetricDirection::HigherIsBetter);
        }
        return ratio(0, 1, MetricDirection::HigherIsBetter);
    }
    ratio(
        submitted.intersection(truth).count() as u64,
        submitted.len() as u64,
        MetricDirection::HigherIsBetter,
    )
}

pub(crate) fn recall<T: Ord>(
    submitted: &BTreeSet<T>,
    truth: &BTreeSet<T>,
    benign: bool,
) -> MetricValue {
    if truth.is_empty() {
        if benign && submitted.is_empty() {
            return ratio(1, 1, MetricDirection::HigherIsBetter);
        }
        return unavailable(Applicability::NotRequired, MetricDirection::HigherIsBetter);
    }
    ratio(
        submitted.intersection(truth).count() as u64,
        truth.len() as u64,
        MetricDirection::HigherIsBetter,
    )
}

pub(crate) fn counted(numerator: u64, denominator: u64, direction: MetricDirection) -> MetricValue {
    if denominator == 0 {
        unavailable(Applicability::ZeroDenominator, direction)
    } else {
        ratio(numerator.min(denominator), denominator, direction)
    }
}

pub(crate) fn ratio(numerator: u64, denominator: u64, direction: MetricDirection) -> MetricValue {
    MetricValue {
        value: Some(numerator as f64 / denominator as f64),
        applicability: Applicability::Applicable,
        direction,
        range: unit_range(),
        numerator: Some(numerator),
        denominator: Some(denominator),
    }
}

pub(crate) fn unavailable(applicability: Applicability, direction: MetricDirection) -> MetricValue {
    MetricValue {
        value: None,
        applicability,
        direction,
        range: unit_range(),
        numerator: None,
        denominator: None,
    }
}

pub(crate) const fn unit_range() -> MetricRange {
    MetricRange {
        minimum: 0.0,
        maximum: 1.0,
    }
}
