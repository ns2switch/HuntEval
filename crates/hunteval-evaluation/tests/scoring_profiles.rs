use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{Applicability, MetricDirection, MetricRange, MetricValue, SchemaVersion};
use hunteval_evaluation::{
    MetricVector, MissingMetricPolicy, ScoringProfile, evaluate_constraints, score_profile,
};

fn metric(value: Option<f64>) -> MetricValue {
    MetricValue {
        value,
        applicability: if value.is_some() {
            Applicability::Applicable
        } else {
            Applicability::NotRequired
        },
        direction: MetricDirection::HigherIsBetter,
        range: MetricRange {
            minimum: 0.0,
            maximum: 1.0,
        },
        numerator: None,
        denominator: None,
    }
}

fn profile(policy: MissingMetricPolicy) -> ScoringProfile {
    ScoringProfile {
        schema_version: SchemaVersion::new(0, 3),
        id: "balanced-0.3".into(),
        missing_metric_policy: policy,
        weights: BTreeMap::from([("a".into(), 0.5), ("b".into(), 0.5)]),
        disqualifying_constraints: BTreeSet::new(),
    }
}

#[test]
fn renormalizes_only_when_profile_requests_it() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = MetricVector(BTreeMap::from([
        ("a".into(), metric(Some(0.8))),
        ("b".into(), metric(None)),
    ]));
    assert_eq!(
        score_profile(&metrics, &profile(MissingMetricPolicy::Renormalize))?.value,
        Some(0.8)
    );
    assert_eq!(
        score_profile(&metrics, &profile(MissingMetricPolicy::Zero))?.value,
        Some(0.4)
    );
    assert_eq!(
        score_profile(&metrics, &profile(MissingMetricPolicy::Reject))?.value,
        None
    );
    Ok(())
}

#[test]
fn rejects_invalid_weights_and_unknown_metrics() {
    let metrics = MetricVector(BTreeMap::from([("a".into(), metric(Some(1.0)))]));
    assert!(score_profile(&metrics, &profile(MissingMetricPolicy::Zero)).is_err());
    let mut invalid = profile(MissingMetricPolicy::Zero);
    invalid.weights.insert("a".into(), f64::NAN);
    assert!(score_profile(&metrics, &invalid).is_err());
}

#[test]
fn normalized_json_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let vector = MetricVector(BTreeMap::from([
        ("z".into(), metric(Some(0.2))),
        ("a".into(), metric(Some(0.9))),
    ]));
    assert_eq!(
        serde_json::to_string(&vector)?,
        serde_json::to_string(&vector)?
    );
    Ok(())
}

#[test]
fn marks_profile_constraints_as_disqualifying() {
    let mut profile = profile(MissingMetricPolicy::Reject);
    profile
        .disqualifying_constraints
        .insert("ground_truth_exposure".into());
    let observed = BTreeSet::from(["ground_truth_exposure".into(), "timeout".into()]);
    let constraints = evaluate_constraints(&observed, &profile);
    assert!(constraints[0].disqualifying);
    assert!(!constraints[1].disqualifying);
}
