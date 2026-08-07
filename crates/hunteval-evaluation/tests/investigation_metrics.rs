use std::collections::BTreeSet;

use hunteval_domain::{
    Applicability, EventId, ExpectedTimelineWindow, SubmissionStatus, TimelineEntry,
};
use hunteval_evaluation::{
    DeterministicEvaluator, EfficiencyInput, EvaluationError, EvaluationInput, Evaluator,
    MetricVector,
};

#[test]
fn attack_path_scores_exact_partial_reordered_duplicate_and_empty_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    value.expected_attack_path = events(&["e1", "e2", "e3"])?;
    value.submitted_attack_path = value.expected_attack_path.clone();
    assert_values(&evaluate(&value)?, "attack_path", 1.0, 1.0);

    value.submitted_attack_path = events(&["e1", "e3", "e2"])?;
    assert_values(&evaluate(&value)?, "attack_path", 2.0 / 3.0, 2.0 / 3.0);

    value.expected_attack_path = events(&["e1", "e2"])?;
    value.submitted_attack_path = events(&["e1", "e1", "e2"])?;
    assert_values(&evaluate(&value)?, "attack_path", 2.0 / 3.0, 1.0);

    value.submitted_attack_path.clear();
    assert_values(&evaluate(&value)?, "attack_path", 0.0, 0.0);
    value.expected_attack_path.clear();
    value.benign_scored_episode = true;
    assert_values(&evaluate(&value)?, "attack_path", 1.0, 1.0);
    Ok(())
}

#[test]
fn attack_path_rejects_unbounded_duplicate_expansion() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    let duplicate = EventId::new("event-duplicate")?;
    value.expected_attack_path = vec![duplicate.clone(); 2_001];
    value.submitted_attack_path = vec![duplicate; 2_001];
    assert!(matches!(
        DeterministicEvaluator.evaluate(&value),
        Err(EvaluationError::AttackPathComparisonTooLarge)
    ));
    Ok(())
}

#[test]
fn timeline_uses_inclusive_windows_one_to_one_and_explicit_applicability()
-> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    value.expected_timeline_windows = Some(vec![window(
        "e1",
        "2026-08-06T17:59:55Z",
        "2026-08-06T18:00:05Z",
    )?]);
    value.submitted_timeline = Some(vec![entry("e1", "2026-08-06T18:00:05Z")?]);
    assert_values(&evaluate(&value)?, "timeline", 1.0, 1.0);

    value.submitted_timeline = Some(vec![entry("e1", "2026-08-06T18:00:06Z")?]);
    assert_values(&evaluate(&value)?, "timeline", 0.0, 0.0);

    value.expected_timeline_windows = Some(vec![
        window("e1", "2026-08-06T17:59:55Z", "2026-08-06T18:00:05Z")?,
        window("e2", "2026-08-06T18:00:10Z", "2026-08-06T18:00:20Z")?,
    ]);
    value.submitted_timeline = Some(vec![
        entry("e1", "2026-08-06T17:59:55Z")?,
        entry("e2", "2026-08-06T18:00:21Z")?,
    ]);
    assert_values(&evaluate(&value)?, "timeline", 0.5, 0.5);

    value.submitted_timeline = None;
    let metrics = evaluate(&value)?;
    assert_eq!(
        metrics.0["timeline_precision"].applicability,
        Applicability::TimelineNotSubmitted
    );
    value.submitted_timeline = Some(Vec::new());
    value.expected_timeline_windows = None;
    let metrics = evaluate(&value)?;
    assert_eq!(
        metrics.0["timeline_recall"].applicability,
        Applicability::TimelineTruthUnavailable
    );
    Ok(())
}

#[test]
fn timeline_rejects_duplicate_events() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    let duplicate = entry("e1", "2026-08-06T18:00:00Z")?;
    value.submitted_timeline = Some(vec![duplicate.clone(), duplicate]);
    value.expected_timeline_windows = Some(vec![window(
        "e1",
        "2026-08-06T17:59:55Z",
        "2026-08-06T18:00:05Z",
    )?]);
    assert!(matches!(
        DeterministicEvaluator.evaluate(&value),
        Err(EvaluationError::DuplicateTimelineEvent)
    ));
    Ok(())
}

#[test]
fn conclusion_scores_only_structured_statuses() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    let unavailable = evaluate(&value)?;
    assert_eq!(
        unavailable.0["conclusion_correctness"].applicability,
        Applicability::AcceptableStatusesUnavailable
    );
    value.acceptable_submission_statuses = Some(BTreeSet::from([
        SubmissionStatus::ConfirmedMaliciousActivity,
        SubmissionStatus::SuspiciousActivity,
    ]));
    value.submitted_status = SubmissionStatus::SuspiciousActivity;
    assert_eq!(
        evaluate(&value)?.0["conclusion_correctness"].value,
        Some(1.0)
    );
    value.submitted_status = SubmissionStatus::NoMaliciousActivity;
    assert_eq!(
        evaluate(&value)?.0["conclusion_correctness"].value,
        Some(0.0)
    );
    value.acceptable_submission_statuses = Some(BTreeSet::new());
    assert!(matches!(
        DeterministicEvaluator.evaluate(&value),
        Err(EvaluationError::EmptyAcceptableStatuses)
    ));
    Ok(())
}

#[test]
fn technique_metrics_are_exact_and_reject_unsupported_identifiers()
-> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    value.expected_attack_techniques = BTreeSet::from(["T1078".to_owned(), "T1098".to_owned()]);
    value.submitted_attack_techniques =
        BTreeSet::from(["T1078".to_owned(), "T1078.004".to_owned()]);
    assert_values(&evaluate(&value)?, "technique", 0.5, 0.5);
    value
        .submitted_attack_techniques
        .insert("T1078@15".to_owned());
    assert!(matches!(
        DeterministicEvaluator.evaluate(&value),
        Err(EvaluationError::UnsupportedTechniqueIdentifier(_))
    ));
    value.expected_attack_techniques.clear();
    value.submitted_attack_techniques.clear();
    value.benign_scored_episode = true;
    assert_values(&evaluate(&value)?, "technique", 1.0, 1.0);
    Ok(())
}

#[test]
fn investigation_metrics_remain_bounded_and_deterministic() -> Result<(), Box<dyn std::error::Error>>
{
    for submitted_length in 0..=5 {
        for expected_length in 0..=5 {
            let mut value = input();
            value.submitted_attack_path = generated_events(submitted_length)?;
            value.expected_attack_path = generated_events(expected_length)?;
            let first = evaluate(&value)?;
            let second = evaluate(&value)?;
            assert_eq!(first, second);
            for metric in first.0.values() {
                metric.validate()?;
                assert!(
                    metric
                        .value
                        .is_none_or(|score| (0.0..=1.0).contains(&score))
                );
            }
        }
    }
    Ok(())
}

fn input() -> EvaluationInput {
    EvaluationInput {
        truth_events: BTreeSet::new(),
        submitted_events: BTreeSet::new(),
        truth_entities: BTreeSet::new(),
        submitted_entities: BTreeSet::new(),
        expected_attack_path: Vec::new(),
        submitted_attack_path: Vec::new(),
        expected_timeline_windows: None,
        submitted_timeline: None,
        acceptable_submission_statuses: None,
        submitted_status: SubmissionStatus::Inconclusive,
        expected_attack_techniques: BTreeSet::new(),
        submitted_attack_techniques: BTreeSet::new(),
        grounded_evidence_events: BTreeSet::new(),
        grounded_evidence_entities: BTreeSet::new(),
        submitted_grounded_evidence_items: 0,
        minimum_evidence_items: 0,
        duplicate_tool_calls: 0,
        useful_messages: 0,
        operational_messages: 0,
        benign_scored_episode: false,
        evidence_items: 0,
        grounded_evidence_items: 0,
        findings_submitted: 0,
        provenance_references: 0,
        valid_provenance_references: 0,
        tasks_created: 0,
        tasks_completed: 0,
        tool_calls_used: 0,
        tool_call_limit: 0,
        resources: EfficiencyInput::default(),
    }
}

fn evaluate(input: &EvaluationInput) -> Result<MetricVector, EvaluationError> {
    DeterministicEvaluator.evaluate(input)
}

fn events(values: &[&str]) -> Result<Vec<EventId>, Box<dyn std::error::Error>> {
    values
        .iter()
        .map(|value| EventId::new(*value).map_err(Into::into))
        .collect()
}

fn generated_events(length: usize) -> Result<Vec<EventId>, Box<dyn std::error::Error>> {
    (0..length)
        .map(|index| EventId::new(format!("event-{index}")).map_err(Into::into))
        .collect()
}

fn window(
    event_id: &str,
    earliest: &str,
    latest: &str,
) -> Result<ExpectedTimelineWindow, Box<dyn std::error::Error>> {
    Ok(ExpectedTimelineWindow {
        event_id: EventId::new(event_id)?,
        earliest: earliest.parse()?,
        latest: latest.parse()?,
    })
}

fn entry(event_id: &str, observed_at: &str) -> Result<TimelineEntry, Box<dyn std::error::Error>> {
    Ok(TimelineEntry {
        event_id: EventId::new(event_id)?,
        observed_at: observed_at.parse()?,
        summary: "Observed event".to_owned(),
        evidence_ids: BTreeSet::new(),
    })
}

fn assert_values(metrics: &MetricVector, prefix: &str, precision: f64, recall: f64) {
    assert_eq!(
        metrics.0[&format!("{prefix}_precision")].value,
        Some(precision)
    );
    assert_eq!(metrics.0[&format!("{prefix}_recall")].value, Some(recall));
}
