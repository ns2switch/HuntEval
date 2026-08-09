use hunteval_domain::{SchemaVersion, StatisticalPolicyId};
use hunteval_statistics::{
    CalibrationPolicy, ClaimStrength, ComparisonClass, EffectSizeMethod, IntervalMethod,
    MultiplicityMethod, MultiplicityPolicy, StatisticalPolicy, claim_strength, compare_with_policy,
    enforce_multiplicity_guard, holm_bonferroni_thresholds, paired_effect_size,
};

fn policy(class: ComparisonClass) -> Result<StatisticalPolicy, Box<dyn std::error::Error>> {
    Ok(StatisticalPolicy {
        schema_version: SchemaVersion::new(0, 6),
        id: StatisticalPolicyId::new("paired-validation-v1")?,
        comparison_class: class,
        minimum_paired_samples: 3,
        confidence_level: 0.95,
        interval_method: IntervalMethod::DeterministicPairedBootstrap,
        effect_size_method: EffectSizeMethod::PairedMeanDifference,
        multiplicity: MultiplicityPolicy {
            method: if class == ComparisonClass::Exploratory {
                MultiplicityMethod::ExploratoryUnadjusted
            } else {
                MultiplicityMethod::HolmBonferroni
            },
            family: "primary-quality".to_owned(),
        },
        calibration: CalibrationPolicy::ConfidenceBrierAndSeverityConfusion,
    })
}

#[test]
fn multiple_holm_comparisons_remain_descriptive_without_adjusted_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let policy = policy(ComparisonClass::Validation)?;
    let digest = hunteval_domain::Sha256Digest::from_bytes(b"policy");
    let comparison = compare_with_policy(
        &policy,
        digest,
        &[Some(1.0), Some(1.0), Some(1.0)],
        &[Some(0.0), Some(0.0), Some(0.0)],
        42,
    )?;
    let mut family = std::collections::BTreeMap::from([
        ("quality".to_owned(), comparison.clone()),
        ("resource".to_owned(), comparison),
    ]);
    assert!(enforce_multiplicity_guard(&policy, &mut family)?);
    assert!(family.values().all(|result| {
        result.claim_strength == ClaimStrength::Descriptive && !result.paired_difference.conclusive
    }));
    Ok(())
}

#[test]
fn policy_requires_samples_and_validated_claim_class() -> Result<(), Box<dyn std::error::Error>> {
    let validation = policy(ComparisonClass::Validation)?;
    validation.validate()?;
    assert_eq!(
        claim_strength(&validation, 2, true)?,
        ClaimStrength::Descriptive
    );
    assert_eq!(
        claim_strength(&validation, 3, false)?,
        ClaimStrength::Descriptive
    );
    assert_eq!(
        claim_strength(&validation, 3, true)?,
        ClaimStrength::Conclusive
    );
    assert_eq!(
        claim_strength(&policy(ComparisonClass::Exploratory)?, 3, true)?,
        ClaimStrength::Exploratory
    );
    Ok(())
}

#[test]
fn multiplicity_thresholds_are_bounded_and_monotonic() -> Result<(), Box<dyn std::error::Error>> {
    let thresholds = holm_bonferroni_thresholds(3, 0.05)?;
    assert_eq!(thresholds.len(), 3);
    assert!(thresholds.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(holm_bonferroni_thresholds(0, 0.05).is_err());
    Ok(())
}

#[test]
fn paired_effect_size_preserves_missingness_and_zero_variance()
-> Result<(), Box<dyn std::error::Error>> {
    let result = paired_effect_size(
        &[Some(0.8), None, Some(0.6)],
        &[Some(0.5), Some(0.2), Some(0.3)],
    )?;
    assert_eq!(result.count, 2);
    assert!(
        result
            .mean_difference
            .is_some_and(|value| (value - 0.3).abs() < f64::EPSILON)
    );
    assert_eq!(result.standardized_difference, None);
    Ok(())
}

#[test]
fn policy_bound_comparison_preserves_confidence_hash_and_claim_strength()
-> Result<(), Box<dyn std::error::Error>> {
    let mut policy = policy(ComparisonClass::Validation)?;
    policy.confidence_level = 0.9;
    let digest = hunteval_domain::Sha256Digest::from_bytes(b"policy");
    let result = compare_with_policy(
        &policy,
        digest,
        &[Some(1.0), Some(1.0), Some(1.0)],
        &[Some(0.0), Some(0.0), Some(0.0)],
        42,
    )?;
    assert_eq!(result.policy_sha256, digest);
    assert_eq!(result.claim_strength, ClaimStrength::Conclusive);
    assert_eq!(
        result
            .paired_difference
            .interval
            .map(|value| value.confidence),
        Some(0.9)
    );
    Ok(())
}
