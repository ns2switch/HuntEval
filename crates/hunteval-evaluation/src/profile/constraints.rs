use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::ResourceProvenance;

use crate::{
    ConstraintEvaluation, ConstraintStatus, MetricReference, MetricVector,
    ResourceProvenanceRequirement, ScoringConstraint, ScoringProfile, ThresholdComparison,
    metric_contract,
};

use super::{
    ProfileError,
    validation::{validate_metric_value, validate_profile},
};

#[derive(Debug)]
pub struct ConstraintInput<'a> {
    pub observed_violations: &'a BTreeSet<String>,
    pub metrics: &'a MetricVector,
    pub resource_provenance: &'a BTreeMap<String, ResourceProvenance>,
}

pub fn evaluate_constraints(
    input: ConstraintInput<'_>,
    profile: &ScoringProfile,
) -> Result<Vec<ConstraintEvaluation>, ProfileError> {
    validate_profile(profile)?;
    profile
        .constraints
        .iter()
        .map(|constraint| evaluate_constraint(&input, constraint))
        .collect()
}

fn evaluate_constraint(
    input: &ConstraintInput<'_>,
    constraint: &ScoringConstraint,
) -> Result<ConstraintEvaluation, ProfileError> {
    match constraint {
        ScoringConstraint::ObservedViolation {
            code,
            disqualifying,
        } => Ok(ConstraintEvaluation {
            code: code.clone(),
            status: if input.observed_violations.contains(code) {
                ConstraintStatus::Violated
            } else {
                ConstraintStatus::Satisfied
            },
            disqualifying: *disqualifying,
        }),
        ScoringConstraint::MetricThreshold {
            code,
            metric,
            comparison,
            threshold,
            disqualifying,
            required_resource_provenance,
        } => Ok(ConstraintEvaluation {
            code: code.clone(),
            status: threshold_status(
                input,
                metric,
                *comparison,
                *threshold,
                *required_resource_provenance,
            )?,
            disqualifying: *disqualifying,
        }),
    }
}

fn threshold_status(
    input: &ConstraintInput<'_>,
    reference: &MetricReference,
    comparison: ThresholdComparison,
    threshold: f64,
    required_provenance: ResourceProvenanceRequirement,
) -> Result<ConstraintStatus, ProfileError> {
    let contract = metric_contract(&reference.name, reference.version).ok_or_else(|| {
        ProfileError::UnknownMetricVersion(reference.name.clone(), reference.version)
    })?;
    if !provenance_matches(
        required_provenance,
        input.resource_provenance.get(&reference.name).copied(),
    ) {
        return Ok(ConstraintStatus::Unverifiable);
    }
    let Some(metric) = input.metrics.0.get(&reference.name) else {
        return Ok(ConstraintStatus::Unverifiable);
    };
    validate_metric_value(&reference.name, metric, contract.direction)?;
    let Some(value) = metric.value else {
        return Ok(ConstraintStatus::Unverifiable);
    };
    let satisfied = match comparison {
        ThresholdComparison::Minimum => value >= threshold,
        ThresholdComparison::Maximum => value <= threshold,
    };
    Ok(if satisfied {
        ConstraintStatus::Satisfied
    } else {
        ConstraintStatus::Violated
    })
}

const fn provenance_matches(
    requirement: ResourceProvenanceRequirement,
    actual: Option<ResourceProvenance>,
) -> bool {
    match requirement {
        ResourceProvenanceRequirement::None => true,
        ResourceProvenanceRequirement::Measured => {
            matches!(actual, Some(ResourceProvenance::Measured))
        }
        ResourceProvenanceRequirement::VerifiedAdapter => {
            matches!(actual, Some(ResourceProvenance::VerifiedAdapter))
        }
    }
}
