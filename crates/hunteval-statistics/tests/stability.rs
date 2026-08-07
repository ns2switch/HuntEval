use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::Applicability;
use hunteval_statistics::{
    StabilityError, StabilityInput, StabilitySample, UnavailableRepetition,
    UnavailableRepetitionReason, evaluate_stability,
};

#[test]
fn stability_identical_and_divergent_submissions_are_deterministic()
-> Result<(), Box<dyn std::error::Error>> {
    let first = sample(29, &["status:malicious", "event:e1"], [("recall", 1.0)]);
    let second = sample(11, &["status:malicious", "event:e1"], [("recall", 1.0)]);
    let identical = evaluate_stability(complete(vec![first.clone(), second]))?;
    assert_eq!(identical.submission_stability.value, Some(1.0));
    assert_eq!(identical.metric_stability.value, Some(1.0));

    let divergent = evaluate_stability(complete(vec![
        first,
        sample(11, &["status:benign", "event:e2"], [("recall", 0.0)]),
    ]))?;
    assert_eq!(divergent.submission_stability.value, Some(0.0));
    assert_eq!(divergent.metric_stability.value, Some(0.0));
    assert_eq!(divergent.comparable_pairs, 1);
    Ok(())
}

#[test]
fn stability_missing_or_failed_cells_are_explicit_and_never_imputed()
-> Result<(), Box<dyn std::error::Error>> {
    let summary = evaluate_stability(StabilityInput {
        required_seeds: vec![29, 11, 47],
        samples: vec![sample(11, &["event:e1"], [("recall", 1.0)])],
        unavailable: vec![
            UnavailableRepetition {
                seed: 47,
                reason: UnavailableRepetitionReason::Failed,
            },
            UnavailableRepetition {
                seed: 29,
                reason: UnavailableRepetitionReason::Missing,
            },
        ],
    })?;
    assert_eq!(summary.submission_stability.value, None);
    assert_eq!(
        summary.submission_stability.applicability,
        Applicability::RequiresComparableCells
    );
    assert_eq!(summary.unavailable[0].seed, 29);
    assert_eq!(summary.unavailable[1].seed, 47);
    Ok(())
}

#[test]
fn stability_one_declared_sample_requires_repetitions() -> Result<(), Box<dyn std::error::Error>> {
    let summary = evaluate_stability(StabilityInput {
        required_seeds: vec![11],
        samples: vec![sample(11, &["event:e1"], [("recall", 1.0)])],
        unavailable: Vec::new(),
    })?;
    assert_eq!(
        summary.metric_stability.applicability,
        Applicability::RequiresRepeatedRuns
    );
    Ok(())
}

#[test]
fn stability_rejects_duplicate_seeds_and_incomparable_or_invalid_metrics()
-> Result<(), Box<dyn std::error::Error>> {
    let duplicate = StabilityInput {
        required_seeds: vec![11, 11],
        samples: Vec::new(),
        unavailable: Vec::new(),
    };
    assert_eq!(
        evaluate_stability(duplicate),
        Err(StabilityError::InvalidSeedSet)
    );

    let inconsistent = complete(vec![
        sample(11, &[], [("recall", 1.0)]),
        sample(29, &[], [("precision", 1.0)]),
    ]);
    let inconsistent = evaluate_stability(inconsistent)?;
    assert_eq!(inconsistent.metric_stability.value, None);
    assert_eq!(
        inconsistent.metric_stability.applicability,
        Applicability::RequiresComparableCells
    );

    let invalid = complete(vec![
        sample(11, &[], [("recall", 1.1)]),
        sample(29, &[], [("recall", 1.0)]),
    ]);
    assert_eq!(
        evaluate_stability(invalid),
        Err(StabilityError::InvalidMetricValue)
    );
    Ok(())
}

#[test]
fn stability_rejects_unbounded_sample_sets() {
    let required_seeds = (0..10_001).collect::<Vec<_>>();
    let unavailable = required_seeds
        .iter()
        .map(|seed| UnavailableRepetition {
            seed: *seed,
            reason: UnavailableRepetitionReason::Missing,
        })
        .collect();
    assert_eq!(
        evaluate_stability(StabilityInput {
            required_seeds,
            samples: Vec::new(),
            unavailable,
        }),
        Err(StabilityError::ComparisonTooLarge)
    );
}

fn complete(samples: Vec<StabilitySample>) -> StabilityInput {
    StabilityInput {
        required_seeds: vec![11, 29],
        samples,
        unavailable: Vec::new(),
    }
}

fn sample<const N: usize>(
    seed: u64,
    claims: &[&str],
    metrics: [(&str, f64); N],
) -> StabilitySample {
    StabilitySample {
        seed,
        submission_claims: claims
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<BTreeSet<_>>(),
        metrics: metrics
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<BTreeMap<_, _>>(),
    }
}
