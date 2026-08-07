use std::collections::BTreeMap;

use hunteval_statistics::{RankingEntry, paired_difference, rank, summarize};

#[test]
fn bootstrap_is_deterministic_and_reports_sample_count() -> Result<(), Box<dyn std::error::Error>> {
    let first = summarize(&[0.2, 0.4, 0.8], 17)?;
    assert_eq!(first, summarize(&[0.2, 0.4, 0.8], 17)?);
    assert_eq!(first.count, 3);
    Ok(())
}

#[test]
fn ranking_applies_constraints_before_aggregate_score() {
    let entries = vec![
        RankingEntry {
            deployment: "unsafe".into(),
            disqualifying_violations: 1,
            aggregate_score: Some(1.0),
            raw_metrics: BTreeMap::new(),
        },
        RankingEntry {
            deployment: "safe".into(),
            disqualifying_violations: 0,
            aggregate_score: Some(0.4),
            raw_metrics: BTreeMap::new(),
        },
    ];
    assert_eq!(rank(entries)[0].deployment, "safe");
}

#[test]
fn pairs_only_present_cells_and_labels_inconclusive() -> Result<(), Box<dyn std::error::Error>> {
    let result = paired_difference(
        &[Some(0.7), None, Some(0.4)],
        &[Some(0.5), Some(0.9), Some(0.5)],
        9,
    )?;
    assert_eq!(result.count, 2);
    assert_eq!((result.wins, result.losses), (1, 1));
    assert!(!result.conclusive);
    Ok(())
}
