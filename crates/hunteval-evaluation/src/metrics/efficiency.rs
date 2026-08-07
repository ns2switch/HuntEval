use std::collections::BTreeMap;

use hunteval_domain::{
    Applicability, MetricDirection, MetricRange, MetricValue, ResourceProvenance,
};

use crate::{EvaluationError, EvaluationInput};

const RANGE: MetricRange = MetricRange {
    minimum: 0.0,
    maximum: 1.0,
};

pub(super) fn evaluate(
    input: &EvaluationInput,
    metrics: &mut BTreeMap<String, MetricValue>,
) -> Result<(), EvaluationError> {
    metrics.insert(
        "measured_duration_utilization".into(),
        duration_utilization(input),
    );
    metrics.insert("verified_cost_utilization".into(), cost_utilization(input)?);
    Ok(())
}

fn duration_utilization(input: &EvaluationInput) -> MetricValue {
    let resources = &input.resources;
    if resources.duration_cap_ms == 0 {
        return unavailable(Applicability::ZeroDenominator);
    }
    MetricValue {
        value: Some((resources.duration_ms as f64 / resources.duration_cap_ms as f64).min(1.0)),
        applicability: Applicability::Applicable,
        direction: MetricDirection::LowerIsBetter,
        range: RANGE,
        numerator: Some(resources.duration_ms.min(resources.duration_cap_ms)),
        denominator: Some(resources.duration_cap_ms),
    }
}

fn cost_utilization(input: &EvaluationInput) -> Result<MetricValue, EvaluationError> {
    let resources = &input.resources;
    validate_cost(resources.estimated_cost, resources.cost_provenance)?;
    if resources
        .estimated_cost_cap
        .is_some_and(|cap| !cap.is_finite() || cap < 0.0)
    {
        return Err(EvaluationError::InvalidResourceUsage);
    }
    if resources.cost_provenance != ResourceProvenance::VerifiedAdapter {
        return Ok(unavailable(Applicability::RequiresVerifiedResourceUsage));
    }
    let cost = resources
        .estimated_cost
        .ok_or(EvaluationError::InvalidResourceUsage)?;
    let Some(cap) = resources.estimated_cost_cap else {
        return Ok(unavailable(Applicability::UnavailableResource));
    };
    if cap == 0.0 {
        return Ok(unavailable(Applicability::ZeroDenominator));
    }
    Ok(MetricValue {
        value: Some((cost / cap).min(1.0)),
        applicability: Applicability::Applicable,
        direction: MetricDirection::LowerIsBetter,
        range: RANGE,
        numerator: None,
        denominator: None,
    })
}

fn validate_cost(
    value: Option<f64>,
    provenance: ResourceProvenance,
) -> Result<(), EvaluationError> {
    if value.is_some_and(|cost| !cost.is_finite() || cost < 0.0)
        || matches!(provenance, ResourceProvenance::Unavailable) != value.is_none()
        || provenance == ResourceProvenance::Measured
    {
        Err(EvaluationError::InvalidResourceUsage)
    } else {
        Ok(())
    }
}

const fn unavailable(applicability: Applicability) -> MetricValue {
    MetricValue {
        value: None,
        applicability,
        direction: MetricDirection::LowerIsBetter,
        range: RANGE,
        numerator: None,
        denominator: None,
    }
}
