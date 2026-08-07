use std::collections::BTreeSet;

use hunteval_domain::EventId;
use hunteval_evaluation::{DeterministicEvaluator, EvaluationInput, Evaluator};

fn event(value: &str) -> Result<EventId, Box<dyn std::error::Error>> {
    Ok(EventId::new(value)?)
}

fn input() -> EvaluationInput {
    EvaluationInput {
        truth_events: BTreeSet::new(),
        submitted_events: BTreeSet::new(),
        truth_entities: BTreeSet::new(),
        submitted_entities: BTreeSet::new(),
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

#[test]
fn computes_exact_and_partial_set_metrics() -> Result<(), Box<dyn std::error::Error>> {
    let mut value = input();
    value.truth_events = [event("e1")?, event("e2")?].into_iter().collect();
    value.submitted_events = [event("e1")?, event("noise")?].into_iter().collect();
    let metrics = DeterministicEvaluator.evaluate(&value)?;
    assert_eq!(metrics.0["event_precision"].value, Some(0.5));
    assert_eq!(metrics.0["event_recall"].value, Some(0.5));
    Ok(())
}

#[test]
fn distinguishes_benign_empty_sets_and_zero_denominators() -> Result<(), Box<dyn std::error::Error>>
{
    let mut value = input();
    value.benign_scored_episode = true;
    let metrics = DeterministicEvaluator.evaluate(&value)?;
    assert_eq!(metrics.0["event_precision"].value, Some(1.0));
    assert_eq!(metrics.0["task_completion"].value, None);
    Ok(())
}

#[test]
fn rejects_forged_counts_and_budget_overrun() {
    let mut value = input();
    value.evidence_items = 1;
    value.grounded_evidence_items = 2;
    assert!(DeterministicEvaluator.evaluate(&value).is_err());
    value.grounded_evidence_items = 1;
    value.tool_calls_used = 1;
    assert!(DeterministicEvaluator.evaluate(&value).is_err());
}

#[test]
fn resilience_requires_a_paired_fault_run() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = DeterministicEvaluator.evaluate(&input())?;
    let resilience = &metrics.0["resilience"];
    assert_eq!(resilience.value, None);
    assert_eq!(
        resilience.applicability,
        hunteval_domain::Applicability::RequiresFaultPair
    );
    Ok(())
}
