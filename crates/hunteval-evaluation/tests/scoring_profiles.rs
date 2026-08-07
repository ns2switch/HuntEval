use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use hunteval_domain::{
    Applicability, MetricDirection, MetricRange, MetricValue, ResourceProvenance, SchemaVersion,
};
use hunteval_evaluation::{
    ConstraintInput, ConstraintStatus, LegacyScoringProfile, MetricReference, MetricSelection,
    MetricVector, MissingMetricPolicy, ProfileError, ResourceProvenanceRequirement,
    ScoringConstraint, ScoringProfile, ScoringProfileArtifact, ThresholdComparison,
    evaluate_constraints, metric_contracts, normalize_profile, score_profile,
};

const V03: SchemaVersion = SchemaVersion::new(0, 3);
const V04: SchemaVersion = SchemaVersion::new(0, 4);

#[test]
fn scoring_profiles_apply_every_missing_value_policy() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = MetricVector(BTreeMap::from([
        (
            "event_precision".into(),
            metric(Some(0.8), MetricDirection::HigherIsBetter),
        ),
        (
            "event_recall".into(),
            metric(None, MetricDirection::HigherIsBetter),
        ),
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
fn scoring_profiles_never_renormalize_missing_protected_metrics()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, version) in [
        ("resilience", V03),
        ("graceful_degradation", V03),
        ("verified_cost_utilization", V04),
        ("submission_stability", V04),
        ("metric_stability", V04),
    ] {
        let profile = ScoringProfile {
            schema_version: V04,
            id: "protected-metric".into(),
            missing_metric_policy: MissingMetricPolicy::Renormalize,
            metrics: BTreeMap::from([
                ("event_recall".into(), selection(V03, 0.5)),
                (name.into(), selection(version, 0.5)),
            ]),
            constraints: Vec::new(),
        };
        let metrics = MetricVector(BTreeMap::from([(
            "event_recall".into(),
            metric(Some(1.0), MetricDirection::HigherIsBetter),
        )]));
        assert_eq!(score_profile(&metrics, &profile)?.value, None, "{name}");

        let mut zero = profile;
        zero.missing_metric_policy = MissingMetricPolicy::Zero;
        assert_eq!(score_profile(&metrics, &zero)?.value, Some(0.5), "{name}");
    }
    Ok(())
}

#[test]
fn scoring_profiles_require_schema_aligned_ids_and_constraint_field() {
    let missing_constraints = b"schema_version: '0.4'\nid: valid\nmissing_metric_policy: reject\nmetrics:\n  event_recall: {version: '0.3', weight: 1.0}\n";
    assert!(serde_yaml_ng::from_slice::<ScoringProfileArtifact>(missing_constraints).is_err());

    let mut invalid_id = profile(MissingMetricPolicy::Reject);
    invalid_id.id = "-invalid".into();
    assert_eq!(
        score_profile(&MetricVector::default(), &invalid_id),
        Err(ProfileError::InvalidProfile)
    );
}

#[test]
fn scoring_profiles_validate_names_versions_weights_and_registered_direction() {
    let metrics = MetricVector(BTreeMap::from([(
        "event_precision".into(),
        metric(Some(1.0), MetricDirection::HigherIsBetter),
    )]));
    let mut invalid = profile(MissingMetricPolicy::Zero);
    if let Some(item) = invalid.metrics.get_mut("event_precision") {
        item.weight = f64::NAN;
    }
    assert_eq!(
        score_profile(&metrics, &invalid),
        Err(ProfileError::InvalidWeightsOrMetric)
    );

    let mut unknown = profile(MissingMetricPolicy::Zero);
    unknown.metrics.remove("event_recall");
    unknown
        .metrics
        .insert("unknown_metric".into(), selection(V04, 0.5));
    assert!(matches!(
        score_profile(&metrics, &unknown),
        Err(ProfileError::UnknownMetricVersion(_, _))
    ));

    let mut wrong_version = profile(MissingMetricPolicy::Zero);
    if let Some(item) = wrong_version.metrics.get_mut("event_recall") {
        item.version = V04;
    }
    assert!(score_profile(&metrics, &wrong_version).is_err());

    let forged_direction = MetricVector(BTreeMap::from([
        (
            "event_precision".into(),
            metric(Some(1.0), MetricDirection::LowerIsBetter),
        ),
        (
            "event_recall".into(),
            metric(Some(1.0), MetricDirection::HigherIsBetter),
        ),
    ]));
    assert!(matches!(
        score_profile(&forged_direction, &profile(MissingMetricPolicy::Zero)),
        Err(ProfileError::MetricContractMismatch(_))
    ));
}

#[test]
fn scoring_profiles_require_verified_cost_for_hard_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let profile = cost_constraint_profile(ResourceProvenanceRequirement::VerifiedAdapter);
    let metrics = MetricVector(BTreeMap::from([(
        "verified_cost_utilization".into(),
        metric(Some(0.4), MetricDirection::LowerIsBetter),
    )]));
    let observed = BTreeSet::new();
    let self_reported = BTreeMap::from([(
        "verified_cost_utilization".into(),
        ResourceProvenance::SelfReported,
    )]);
    let evaluations = evaluate_constraints(
        ConstraintInput {
            observed_violations: &observed,
            metrics: &metrics,
            resource_provenance: &self_reported,
        },
        &profile,
    )?;
    assert_eq!(evaluations[0].status, ConstraintStatus::Unverifiable);
    assert!(evaluations[0].disqualifying);

    let verified = BTreeMap::from([(
        "verified_cost_utilization".into(),
        ResourceProvenance::VerifiedAdapter,
    )]);
    let evaluations = evaluate_constraints(
        ConstraintInput {
            observed_violations: &observed,
            metrics: &metrics,
            resource_provenance: &verified,
        },
        &profile,
    )?;
    assert_eq!(evaluations[0].status, ConstraintStatus::Satisfied);

    let invalid = cost_constraint_profile(ResourceProvenanceRequirement::None);
    assert_eq!(
        evaluate_constraints(
            ConstraintInput {
                observed_violations: &observed,
                metrics: &metrics,
                resource_provenance: &verified,
            },
            &invalid,
        ),
        Err(ProfileError::InvalidConstraint)
    );
    Ok(())
}

#[test]
fn scoring_profiles_enforce_every_registered_resource_requirement()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, version, requirement) in [
        (
            "tool_call_utilization",
            V03,
            ResourceProvenanceRequirement::Measured,
        ),
        (
            "measured_duration_utilization",
            V04,
            ResourceProvenanceRequirement::Measured,
        ),
        (
            "verified_cost_utilization",
            V04,
            ResourceProvenanceRequirement::VerifiedAdapter,
        ),
    ] {
        let profile = threshold_profile(name, version, requirement);
        let metrics = MetricVector(BTreeMap::from([(
            name.into(),
            metric(Some(0.4), MetricDirection::LowerIsBetter),
        )]));
        let observed = BTreeSet::new();
        let missing = BTreeMap::new();
        let evaluations = evaluate_constraints(
            ConstraintInput {
                observed_violations: &observed,
                metrics: &metrics,
                resource_provenance: &missing,
            },
            &profile,
        )?;
        assert_eq!(evaluations[0].status, ConstraintStatus::Unverifiable);

        let actual = match requirement {
            ResourceProvenanceRequirement::Measured => ResourceProvenance::Measured,
            ResourceProvenanceRequirement::VerifiedAdapter => ResourceProvenance::VerifiedAdapter,
            ResourceProvenanceRequirement::None => unreachable!(),
        };
        let provenance = BTreeMap::from([(name.into(), actual)]);
        let evaluations = evaluate_constraints(
            ConstraintInput {
                observed_violations: &observed,
                metrics: &metrics,
                resource_provenance: &provenance,
            },
            &profile,
        )?;
        assert_eq!(evaluations[0].status, ConstraintStatus::Satisfied);
    }
    Ok(())
}

#[test]
fn scoring_profiles_mark_disqualifying_observed_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let mut profile = profile(MissingMetricPolicy::Reject);
    profile
        .constraints
        .push(ScoringConstraint::ObservedViolation {
            code: "ground_truth_exposure".into(),
            disqualifying: true,
        });
    let observed = BTreeSet::from(["ground_truth_exposure".into()]);
    let metrics = MetricVector::default();
    let provenance = BTreeMap::new();
    let constraints = evaluate_constraints(
        ConstraintInput {
            observed_violations: &observed,
            metrics: &metrics,
            resource_provenance: &provenance,
        },
        &profile,
    )?;
    assert_eq!(constraints[0].status, ConstraintStatus::Violated);
    assert!(constraints[0].disqualifying);
    Ok(())
}

#[test]
fn scoring_profiles_adapt_v03_without_rewriting_the_source()
-> Result<(), Box<dyn std::error::Error>> {
    let legacy = LegacyScoringProfile {
        schema_version: V03,
        id: "legacy".into(),
        missing_metric_policy: MissingMetricPolicy::Renormalize,
        weights: BTreeMap::from([("event_recall".into(), 1.0)]),
        disqualifying_constraints: BTreeSet::from(["ground_truth_exposure".into()]),
    };
    let normalized = normalize_profile(ScoringProfileArtifact::Legacy(legacy.clone()))?;
    assert_eq!(legacy.schema_version, V03);
    assert_eq!(normalized.schema_version, V04);
    assert_eq!(normalized.metrics["event_recall"].version, V03);
    assert_eq!(normalized.constraints.len(), 1);
    Ok(())
}

#[test]
fn scoring_profiles_normalized_json_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let profile = profile(MissingMetricPolicy::Renormalize);
    assert_eq!(
        serde_json::to_string(&profile)?,
        serde_json::to_string(&profile)?
    );
    Ok(())
}

#[test]
fn scoring_profiles_registry_is_unique_and_controls_lower_is_better_normalization()
-> Result<(), Box<dyn std::error::Error>> {
    let contracts = metric_contracts();
    let unique = contracts
        .iter()
        .map(|contract| (contract.name, contract.version))
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), contracts.len());

    let profile = ScoringProfile {
        schema_version: V04,
        id: "cost-only".into(),
        missing_metric_policy: MissingMetricPolicy::Reject,
        metrics: BTreeMap::from([("verified_cost_utilization".into(), selection(V04, 1.0))]),
        constraints: Vec::new(),
    };
    let metrics = MetricVector(BTreeMap::from([(
        "verified_cost_utilization".into(),
        metric(Some(0.25), MetricDirection::LowerIsBetter),
    )]));
    assert_eq!(score_profile(&metrics, &profile)?.value, Some(0.75));
    Ok(())
}

#[test]
fn scoring_profile_schema_metric_selections_match_the_registry()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or("evaluation crate is not inside the workspace")?;
    let schema: serde_json::Value = serde_json::from_slice(&fs::read(
        root.join("schemas/v0.4/scoring-profile.schema.json"),
    )?)?;
    let selections = schema["properties"]["metrics"]["properties"]
        .as_object()
        .ok_or("scoring profile metric properties are missing")?;
    let schema_names = selections
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let registry_names = metric_contracts()
        .iter()
        .map(|contract| contract.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(schema_names, registry_names);

    for contract in metric_contracts() {
        let reference = selections[contract.name]["$ref"]
            .as_str()
            .ok_or("metric selection reference is missing")?;
        let expected = match (contract.version.major(), contract.version.minor()) {
            (0, 3) => "#/$defs/metric_selection_v03",
            (0, 4) => "#/$defs/metric_selection_v04",
            _ => return Err("registry contains an unsupported schema version".into()),
        };
        assert_eq!(reference, expected, "{}", contract.name);
    }
    Ok(())
}

fn profile(policy: MissingMetricPolicy) -> ScoringProfile {
    ScoringProfile {
        schema_version: V04,
        id: "balanced-0.4".into(),
        missing_metric_policy: policy,
        metrics: BTreeMap::from([
            ("event_precision".into(), selection(V03, 0.5)),
            ("event_recall".into(), selection(V03, 0.5)),
        ]),
        constraints: Vec::new(),
    }
}

fn cost_constraint_profile(required: ResourceProvenanceRequirement) -> ScoringProfile {
    ScoringProfile {
        schema_version: V04,
        id: "cost-constraint".into(),
        missing_metric_policy: MissingMetricPolicy::Reject,
        metrics: BTreeMap::from([("event_recall".into(), selection(V03, 1.0))]),
        constraints: vec![ScoringConstraint::MetricThreshold {
            code: "maximum_verified_cost".into(),
            metric: MetricReference {
                name: "verified_cost_utilization".into(),
                version: V04,
            },
            comparison: ThresholdComparison::Maximum,
            threshold: 0.5,
            disqualifying: true,
            required_resource_provenance: required,
        }],
    }
}

fn threshold_profile(
    name: &str,
    version: SchemaVersion,
    requirement: ResourceProvenanceRequirement,
) -> ScoringProfile {
    ScoringProfile {
        schema_version: V04,
        id: "resource-threshold".into(),
        missing_metric_policy: MissingMetricPolicy::Reject,
        metrics: BTreeMap::from([("event_recall".into(), selection(V03, 1.0))]),
        constraints: vec![ScoringConstraint::MetricThreshold {
            code: "maximum_resource".into(),
            metric: MetricReference {
                name: name.into(),
                version,
            },
            comparison: ThresholdComparison::Maximum,
            threshold: 0.5,
            disqualifying: true,
            required_resource_provenance: requirement,
        }],
    }
}

const fn selection(version: SchemaVersion, weight: f64) -> MetricSelection {
    MetricSelection { version, weight }
}

fn metric(value: Option<f64>, direction: MetricDirection) -> MetricValue {
    MetricValue {
        value,
        applicability: if value.is_some() {
            Applicability::Applicable
        } else {
            Applicability::NotRequired
        },
        direction,
        range: MetricRange {
            minimum: 0.0,
            maximum: 1.0,
        },
        numerator: None,
        denominator: None,
    }
}
