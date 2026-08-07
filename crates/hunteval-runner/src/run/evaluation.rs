use hunteval_domain::FinalSubmission;
use hunteval_evaluation::{
    AggregateScore, DeterministicEvaluator, EvaluationInput, Evaluator, MetricVector, score_profile,
};
use hunteval_protocol::{ProtocolPayload, StoredEvent};

use crate::BudgetUsage;

use super::{ResolvedRunInputs, error::EngineError};

pub(super) struct EvaluatedRun {
    pub(super) metrics: MetricVector,
    pub(super) aggregate_score: AggregateScore,
}

pub(super) fn evaluate_validated_run(
    inputs: &ResolvedRunInputs,
    trajectory: &[u8],
    submission: &FinalSubmission,
    usage: BudgetUsage,
) -> Result<EvaluatedRun, EngineError> {
    let mut evidence = 0_u64;
    let mut findings = 0_u64;
    let mut tasks_created = 0_u64;
    let mut tasks_completed = 0_u64;
    for line in trajectory.split_inclusive(|byte| *byte == b'\n') {
        let event: StoredEvent =
            serde_json::from_slice(&line[..line.len() - 1]).map_err(|_| EngineError::Evaluation)?;
        match event.envelope.payload {
            ProtocolPayload::EvidenceShared { .. } => evidence += 1,
            ProtocolPayload::FindingProposed { .. } => findings += 1,
            ProtocolPayload::TaskCreated { .. } => tasks_created += 1,
            ProtocolPayload::TaskCompleted { .. } => tasks_completed += 1,
            _ => {}
        }
    }
    let package = &inputs.episode;
    let metrics = DeterministicEvaluator.evaluate(&EvaluationInput {
        truth_events: package.ground_truth().malicious_event_ids.clone(),
        submitted_events: submission.malicious_event_ids.clone(),
        truth_entities: package.ground_truth().malicious_entity_ids.clone(),
        submitted_entities: submission.malicious_entity_ids.clone(),
        benign_scored_episode: package.public().manifest.benign_evaluation,
        evidence_items: evidence,
        grounded_evidence_items: evidence,
        findings_submitted: findings,
        provenance_references: evidence,
        valid_provenance_references: evidence,
        tasks_created,
        tasks_completed,
        tool_calls_used: usage.tool_calls,
        tool_call_limit: u64::from(package.public().manifest.limits.max_tool_calls),
    })?;
    let aggregate_score = score_profile(&metrics, &inputs.scoring_profile)?;
    Ok(EvaluatedRun {
        metrics,
        aggregate_score,
    })
}
