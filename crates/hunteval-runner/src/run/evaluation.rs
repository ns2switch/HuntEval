use hunteval_evaluation::{
    AggregateScore, ConstraintEvaluation, ConstraintInput, DeterministicEvaluator, Evaluator,
    MetricVector, evaluate_constraints, score_profile,
};

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use hunteval_domain::{ResourceUsage, RunId};

use super::{ResolvedRunInputs, StoredEvaluationHashes, error::EngineError, load_trusted_run_view};

pub(super) struct EvaluatedRun {
    pub(super) metrics: MetricVector,
    pub(super) aggregate_score: AggregateScore,
    pub(super) constraints: Vec<ConstraintEvaluation>,
    pub(super) stored_hashes: StoredEvaluationHashes,
}

pub(super) fn evaluate_stored_run(
    inputs: &ResolvedRunInputs,
    run_root: &Path,
    run_id: &RunId,
    maximum_line_bytes: usize,
    resource_usage: &ResourceUsage,
) -> Result<EvaluatedRun, EngineError> {
    let package = &inputs.episode;
    let expected_hashes = StoredEvaluationHashes {
        trajectory: crate::hash_file(&run_root.join("trajectory.jsonl"))
            .map_err(|_| EngineError::Evaluation)?,
        submission: crate::hash_file(&run_root.join("submission.json"))
            .map_err(|_| EngineError::Evaluation)?,
        ground_truth: package.digests().private_ground_truth,
    };
    let view = load_trusted_run_view(
        run_root,
        run_id,
        package.ground_truth().clone(),
        expected_hashes,
        maximum_line_bytes,
        u64::from(package.public().manifest.limits.max_tool_calls),
        package.public().manifest.benign_evaluation,
    )
    .map_err(|_| EngineError::Evaluation)?;
    let mut evaluation_input = view.evaluation_input();
    evaluation_input.resources = hunteval_evaluation::EfficiencyInput {
        duration_ms: resource_usage.duration_ms,
        duration_cap_ms: package
            .public()
            .manifest
            .limits
            .max_duration_seconds
            .saturating_mul(1_000),
        estimated_cost: resource_usage.estimated_cost.value,
        cost_provenance: resource_usage.estimated_cost.provenance,
        estimated_cost_cap: package.public().manifest.limits.max_estimated_cost,
    };
    let metrics = DeterministicEvaluator.evaluate(&evaluation_input)?;
    let aggregate_score = score_profile(&metrics, &inputs.scoring_profile)?;
    let resource_provenance = BTreeMap::from([
        (
            "tool_call_utilization".to_owned(),
            hunteval_domain::ResourceProvenance::Measured,
        ),
        (
            "measured_duration_utilization".to_owned(),
            hunteval_domain::ResourceProvenance::Measured,
        ),
        (
            "verified_cost_utilization".to_owned(),
            resource_usage.estimated_cost.provenance,
        ),
    ]);
    let observed_violations = BTreeSet::new();
    let constraints = evaluate_constraints(
        ConstraintInput {
            observed_violations: &observed_violations,
            metrics: &metrics,
            resource_provenance: &resource_provenance,
        },
        &inputs.scoring_profile,
    )?;
    Ok(EvaluatedRun {
        metrics,
        aggregate_score,
        constraints,
        stored_hashes: expected_hashes,
    })
}
