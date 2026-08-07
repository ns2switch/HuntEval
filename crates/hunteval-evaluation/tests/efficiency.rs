use std::collections::BTreeSet;

use hunteval_domain::{Applicability, ResourceProvenance, SubmissionStatus};
use hunteval_evaluation::{
    DeterministicEvaluator, EfficiencyInput, EvaluationError, EvaluationInput, Evaluator,
};

#[test]
fn efficiency_measures_duration_and_caps_overruns() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = baseline();
    input.resources.duration_ms = 250;
    input.resources.duration_cap_ms = 1_000;
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(metrics.0["measured_duration_utilization"].value, Some(0.25));

    input.resources.duration_ms = 1_500;
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(metrics.0["measured_duration_utilization"].value, Some(1.0));
    assert_eq!(
        metrics.0["measured_duration_utilization"].numerator,
        Some(1_000)
    );
    Ok(())
}

#[test]
fn efficiency_distinguishes_verified_self_reported_and_unavailable_cost()
-> Result<(), Box<dyn std::error::Error>> {
    let mut input = baseline();
    input.resources.estimated_cost = Some(1.5);
    input.resources.estimated_cost_cap = Some(2.0);
    input.resources.cost_provenance = ResourceProvenance::VerifiedAdapter;
    let verified = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(verified.0["verified_cost_utilization"].value, Some(0.75));

    input.resources.estimated_cost = Some(3.0);
    let exceeded = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(exceeded.0["verified_cost_utilization"].value, Some(1.0));

    input.resources.cost_provenance = ResourceProvenance::SelfReported;
    let reported = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(
        reported.0["verified_cost_utilization"].applicability,
        Applicability::RequiresVerifiedResourceUsage
    );

    input.resources.estimated_cost = None;
    input.resources.cost_provenance = ResourceProvenance::Unavailable;
    let unavailable = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(
        unavailable.0["verified_cost_utilization"].applicability,
        Applicability::RequiresVerifiedResourceUsage
    );
    Ok(())
}

#[test]
fn efficiency_handles_zero_or_missing_caps_and_rejects_invalid_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let mut input = baseline();
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(
        metrics.0["measured_duration_utilization"].applicability,
        Applicability::ZeroDenominator
    );

    input.resources.estimated_cost = Some(0.0);
    input.resources.cost_provenance = ResourceProvenance::VerifiedAdapter;
    input.resources.estimated_cost_cap = Some(0.0);
    let metrics = DeterministicEvaluator.evaluate(&input)?;
    assert_eq!(
        metrics.0["verified_cost_utilization"].applicability,
        Applicability::ZeroDenominator
    );

    input.resources.cost_provenance = ResourceProvenance::Unavailable;
    assert_eq!(
        DeterministicEvaluator.evaluate(&input),
        Err(EvaluationError::InvalidResourceUsage)
    );
    Ok(())
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
        resources: EfficiencyInput::default(),
    }
}
