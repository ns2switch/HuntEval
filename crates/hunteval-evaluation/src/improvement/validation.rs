use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    ConstraintKind, ConstraintResult, ConstraintStatus, ControlledValidationDecision,
    ImprovementConstraint, ImprovementEquivalenceResult, ImprovementEquivalenceStatus,
    ImprovementExperiment, ImprovementMetricApplicability, ImprovementPolicy, MetricDelta,
    MetricInterval, RequiredProvenance, ResourceProvenance, SchemaVersion, Sha256Digest,
    ValidationStatus,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct PairedMetricObservation {
    pub pair_id: String,
    pub metric: String,
    pub version: SchemaVersion,
    pub baseline: Option<f64>,
    pub candidate: Option<f64>,
    pub provenance: ResourceProvenance,
}

#[derive(Debug, Clone, Copy)]
pub struct ControlledValidationInput<'a> {
    pub id: &'a str,
    pub experiment: &'a ImprovementExperiment,
    pub experiment_sha256: Sha256Digest,
    pub equivalence: &'a ImprovementEquivalenceResult,
    pub equivalence_sha256: Sha256Digest,
    pub policy: &'a ImprovementPolicy,
    pub policy_sha256: Sha256Digest,
    pub observations: &'a [PairedMetricObservation],
}

pub fn decide_candidate(
    input: ControlledValidationInput<'_>,
) -> Result<ControlledValidationDecision, ValidationError> {
    if input.equivalence.status != ImprovementEquivalenceStatus::Eligible
        || input.equivalence.experiment_sha256 != input.experiment_sha256
        || input.policy.validate().is_err()
        || input.observations.is_empty()
    {
        return Err(ValidationError::Ineligible);
    }
    let mut grouped: BTreeMap<(&str, SchemaVersion), Vec<&PairedMetricObservation>> =
        BTreeMap::new();
    for observation in input.observations {
        if observation
            .baseline
            .into_iter()
            .chain(observation.candidate)
            .any(|value| !value.is_finite())
        {
            return Err(ValidationError::NonFinite);
        }
        grouped
            .entry((&observation.metric, observation.version))
            .or_default()
            .push(observation);
    }
    let metric_deltas = grouped
        .iter()
        .map(|((metric, version), values)| metric_delta(metric, *version, values))
        .collect::<Vec<_>>();
    let constraints = input
        .policy
        .constraints
        .iter()
        .map(|constraint| evaluate_constraint(constraint, input.observations))
        .collect::<Vec<_>>();
    let status = if constraints
        .iter()
        .all(|result| result.status == ConstraintStatus::Satisfied)
    {
        ValidationStatus::Passed
    } else {
        ValidationStatus::Failed
    };
    let paired = input
        .observations
        .iter()
        .filter(|item| item.baseline.is_some() && item.candidate.is_some())
        .map(|item| item.pair_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_pairs = input
        .observations
        .iter()
        .filter(|item| item.baseline.is_none() || item.candidate.is_none())
        .map(|item| item.pair_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(ControlledValidationDecision {
        schema_version: SchemaVersion::new(0, 8),
        id: input.id.to_owned(),
        experiment_id: input.experiment.id.clone(),
        experiment_sha256: input.experiment_sha256,
        equivalence_sha256: input.equivalence_sha256,
        improvement_policy_sha256: input.policy_sha256,
        status,
        paired_samples: u32::try_from(paired.len()).unwrap_or(u32::MAX),
        missing_pairs,
        metric_deltas,
        constraints,
        hidden_test_used_in_selection: false,
        human_review_required: true,
        limitations: BTreeSet::from(["experimental_topology_dependent".to_owned()]),
    })
}

fn metric_delta(
    metric: &str,
    version: SchemaVersion,
    observations: &[&PairedMetricObservation],
) -> MetricDelta {
    let pairs = observations
        .iter()
        .filter_map(|item| Some((item.baseline?, item.candidate?)))
        .collect::<Vec<_>>();
    if pairs.is_empty() {
        return MetricDelta {
            metric: metric.to_owned(),
            version,
            baseline: None,
            candidate: None,
            difference: None,
            interval: None,
            applicability: ImprovementMetricApplicability::Unavailable,
        };
    }
    let count = pairs.len() as f64;
    let baseline = pairs.iter().map(|pair| pair.0).sum::<f64>() / count;
    let candidate = pairs.iter().map(|pair| pair.1).sum::<f64>() / count;
    let difference = candidate - baseline;
    MetricDelta {
        metric: metric.to_owned(),
        version,
        baseline: Some(baseline),
        candidate: Some(candidate),
        difference: Some(difference),
        interval: Some(MetricInterval {
            lower: difference,
            upper: difference,
        }),
        applicability: ImprovementMetricApplicability::Available,
    }
}

fn evaluate_constraint(
    constraint: &ImprovementConstraint,
    observations: &[PairedMetricObservation],
) -> ConstraintResult {
    let values = observations
        .iter()
        .filter(|item| item.metric == constraint.metric)
        .collect::<Vec<_>>();
    let provenance_ok = match constraint.required_provenance {
        RequiredProvenance::None => true,
        RequiredProvenance::Measured => values
            .iter()
            .all(|item| item.provenance == ResourceProvenance::Measured),
        RequiredProvenance::VerifiedAdapter => values
            .iter()
            .all(|item| item.provenance == ResourceProvenance::VerifiedAdapter),
    };
    let pairs = values
        .iter()
        .filter_map(|item| Some((item.baseline?, item.candidate?)))
        .collect::<Vec<_>>();
    let status = if !provenance_ok || pairs.is_empty() || pairs.len() != values.len() {
        ConstraintStatus::Unverifiable
    } else {
        let count = pairs.len() as f64;
        let baseline = pairs.iter().map(|item| item.0).sum::<f64>() / count;
        let candidate = pairs.iter().map(|item| item.1).sum::<f64>() / count;
        let satisfied = match constraint.kind {
            ConstraintKind::MinimumMetric => candidate >= constraint.threshold,
            ConstraintKind::MaximumRegression => baseline - candidate <= constraint.threshold,
            ConstraintKind::MaximumVerifiedCostIncrease => {
                candidate - baseline <= constraint.threshold
            }
        };
        if satisfied {
            ConstraintStatus::Satisfied
        } else {
            ConstraintStatus::Violated
        }
    };
    ConstraintResult {
        code: format!("{:?}_{}", constraint.kind, constraint.metric).to_ascii_lowercase(),
        status,
        source_metric: constraint.metric.clone(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("controlled validation input is ineligible or empty")]
    Ineligible,
    #[error("controlled validation contains a non-finite metric")]
    NonFinite,
}
