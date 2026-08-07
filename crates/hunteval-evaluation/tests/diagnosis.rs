use std::collections::BTreeSet;

use hunteval_domain::RunId;
use hunteval_evaluation::{
    DiagnosticInput, FailureKind, ObservableFailure, RecommendationStatus, diagnose, recommend,
};

#[test]
fn diagnosis_cites_only_observable_events_and_affected_runs()
-> Result<(), Box<dyn std::error::Error>> {
    let input = DiagnosticInput {
        run_id: RunId::new("run-observed")?,
        failures: vec![ObservableFailure {
            reason_code: "low_event_recall".into(),
            event_sequences: [12].into_iter().collect(),
            metric_references: ["result.json#/raw_metrics/event_recall".into()]
                .into_iter()
                .collect(),
        }],
    };
    let classifications = diagnose(&input);
    assert_eq!(classifications.len(), 1);
    assert_eq!(classifications[0].kind, FailureKind::LowEventRecall);
    assert_eq!(
        classifications[0].evidence.event_sequences,
        BTreeSet::from([12])
    );

    let recommendations = recommend(&classifications);
    assert_eq!(recommendations[0].status, RecommendationStatus::Unvalidated);
    assert!(recommendations[0].human_review_required);
    assert!(recommendations[0].affected_runs.contains(&input.run_id));
    Ok(())
}

#[test]
fn diagnosis_omits_unknown_or_unsupported_claims() -> Result<(), Box<dyn std::error::Error>> {
    let input = DiagnosticInput {
        run_id: RunId::new("run-unknown")?,
        failures: vec![
            ObservableFailure {
                reason_code: "model_was_confused".into(),
                event_sequences: [1].into_iter().collect(),
                metric_references: BTreeSet::new(),
            },
            ObservableFailure {
                reason_code: "task_incomplete".into(),
                event_sequences: BTreeSet::new(),
                metric_references: BTreeSet::new(),
            },
        ],
    };
    assert!(diagnose(&input).is_empty());
    Ok(())
}
