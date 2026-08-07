use hunteval_evaluation::{
    AggregateScore, DeterministicEvaluator, Evaluator, MetricVector, score_profile,
};

use std::path::Path;

use hunteval_domain::RunId;

use super::{ResolvedRunInputs, StoredEvaluationHashes, error::EngineError, load_trusted_run_view};

pub(super) struct EvaluatedRun {
    pub(super) metrics: MetricVector,
    pub(super) aggregate_score: AggregateScore,
    pub(super) stored_hashes: StoredEvaluationHashes,
}

pub(super) fn evaluate_stored_run(
    inputs: &ResolvedRunInputs,
    run_root: &Path,
    run_id: &RunId,
    maximum_line_bytes: usize,
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
    let metrics = DeterministicEvaluator.evaluate(&view.evaluation_input())?;
    let aggregate_score = score_profile(&metrics, &inputs.scoring_profile)?;
    Ok(EvaluatedRun {
        metrics,
        aggregate_score,
        stored_hashes: expected_hashes,
    })
}
