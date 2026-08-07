use std::collections::BTreeSet;

use hunteval_domain::{Applicability, EventId, SubmissionStatus};
use hunteval_evaluation::{DeterministicEvaluator, EvaluationError, EvaluationInput, Evaluator};

#[test]
fn evidence_metrics_score_grounded_coverage_and_minimum() -> Result<(), Box<dyn std::error::Error>>
{
    let mut input = baseline();
    input.truth_events = events(&["event-1", "event-2"])?;
    input.grounded_evidence_events = events(&["event-1", "event-noise"])?;
    input.truth_entities = BTreeSet::from(["entity-1".to_owned(), "entity-2".to_owned()]);
    input.grounded_evidence_entities =
        BTreeSet::from(["entity-1".to_owned(), "entity-noise".to_owned()]);
    input.submitted_grounded_evidence_items = 1;
    input.evidence_items = 1;
    input.grounded_evidence_items = 1;
    input.minimum_evidence_items = 2;
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(metrics.0["evidence_event_coverage"].value, Some(0.5));
    assert_eq!(metrics.0["evidence_entity_coverage"].value, Some(0.5));
    assert_eq!(metrics.0["evidence_sufficiency"].value, Some(0.5));
    Ok(())
}

#[test]
fn evidence_metrics_ignore_forged_facts_and_preserve_zero_denominators()
-> Result<(), Box<dyn std::error::Error>> {
    let mut input = baseline();
    input.truth_events = events(&["event-truth"])?;
    input.grounded_evidence_events = events(&["event-forged"])?;
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(metrics.0["evidence_event_coverage"].value, Some(0.0));
    assert_eq!(
        metrics.0["evidence_sufficiency"].applicability,
        Applicability::InsufficientEvidenceRequirements
    );
    assert_eq!(
        metrics.0["duplicate_tool_work"].applicability,
        Applicability::ZeroDenominator
    );
    assert_eq!(
        metrics.0["useful_communication"].applicability,
        Applicability::ZeroDenominator
    );
    Ok(())
}

#[test]
fn coordination_metrics_preserve_raw_counts_and_direction() -> Result<(), Box<dyn std::error::Error>>
{
    let mut input = baseline();
    input.tool_calls_used = 3;
    input.tool_call_limit = 3;
    input.duplicate_tool_calls = 1;
    input.operational_messages = 4;
    input.useful_messages = 3;
    let first = DeterministicEvaluator.evaluate(&input)?;
    let second = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(first, second);
    assert_eq!(first.0["duplicate_tool_work"].numerator, Some(1));
    assert_eq!(first.0["duplicate_tool_work"].denominator, Some(3));
    assert_eq!(first.0["useful_communication"].value, Some(0.75));
    Ok(())
}

#[test]
fn coordination_metrics_reject_forged_raw_counts() {
    let mut input = baseline();
    input.duplicate_tool_calls = 1;
    assert!(matches!(
        DeterministicEvaluator.evaluate(&input),
        Err(EvaluationError::InvalidCount("duplicate tool calls"))
    ));
    input.duplicate_tool_calls = 0;
    input.useful_messages = 1;
    assert!(matches!(
        DeterministicEvaluator.evaluate(&input),
        Err(EvaluationError::InvalidCount("useful operational messages"))
    ));
}

fn baseline() -> EvaluationInput {
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
    }
}

fn events(values: &[&str]) -> Result<BTreeSet<EventId>, Box<dyn std::error::Error>> {
    values
        .iter()
        .map(|value| EventId::new(*value).map_err(Into::into))
        .collect()
}
