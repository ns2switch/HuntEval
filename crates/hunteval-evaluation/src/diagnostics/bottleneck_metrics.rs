use std::collections::BTreeSet;

use hunteval_domain::{
    BottleneckMetric, DiagnosticApplicability, DiagnosticMetricDirection, DiagnosticMetricRange,
    DiagnosticMetricUnit, DiagnosticSourceReference, SchemaVersion,
};

pub(super) fn count(
    name: &str,
    value: u64,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    available(
        name,
        DiagnosticMetricUnit::Count,
        value as f64,
        value as f64,
        1.0,
        None,
        sources,
    )
}

pub(super) fn duration(
    name: &str,
    value: Option<u64>,
    reason: &str,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    match value {
        Some(value) => available(
            name,
            DiagnosticMetricUnit::Milliseconds,
            value as f64,
            value as f64,
            1.0,
            None,
            sources,
        ),
        None => unavailable(name, DiagnosticMetricUnit::Milliseconds, reason, sources),
    }
}

pub(super) fn ratio(
    name: &str,
    numerator: u64,
    denominator: u64,
    reason: &str,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    duration_ratio(name, numerator, Some(denominator), reason, sources)
}

pub(super) fn optional_ratio(
    name: &str,
    numerator: Option<u64>,
    denominator: Option<u64>,
    reason: &str,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    match numerator {
        Some(numerator) => duration_ratio(name, numerator, denominator, reason, sources),
        None => unavailable(name, DiagnosticMetricUnit::Ratio, reason, sources),
    }
}

fn duration_ratio(
    name: &str,
    numerator: u64,
    denominator: Option<u64>,
    reason: &str,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    match denominator.filter(|value| *value > 0) {
        Some(denominator) => available(
            name,
            DiagnosticMetricUnit::Ratio,
            (numerator as f64 / denominator as f64).min(1.0),
            numerator as f64,
            denominator as f64,
            Some(1.0),
            sources,
        ),
        None => unavailable(name, DiagnosticMetricUnit::Ratio, reason, sources),
    }
}

fn available(
    name: &str,
    unit: DiagnosticMetricUnit,
    value: f64,
    numerator: f64,
    denominator: f64,
    maximum: Option<f64>,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    BottleneckMetric {
        name: name.into(),
        version: SchemaVersion::new(0, 7),
        direction: DiagnosticMetricDirection::LowerIsBetter,
        unit,
        range: DiagnosticMetricRange {
            minimum: 0.0,
            maximum,
        },
        applicability: DiagnosticApplicability::Available,
        value: Some(value),
        numerator: Some(numerator),
        denominator: Some(denominator),
        reason_code: None,
        sources,
    }
}

pub(super) fn unavailable(
    name: &str,
    unit: DiagnosticMetricUnit,
    reason: &str,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    let maximum = (unit == DiagnosticMetricUnit::Ratio).then_some(1.0);
    BottleneckMetric {
        name: name.into(),
        version: SchemaVersion::new(0, 7),
        direction: DiagnosticMetricDirection::LowerIsBetter,
        unit,
        range: DiagnosticMetricRange {
            minimum: 0.0,
            maximum,
        },
        applicability: DiagnosticApplicability::Unavailable,
        value: None,
        numerator: None,
        denominator: None,
        reason_code: Some(reason.into()),
        sources,
    }
}
