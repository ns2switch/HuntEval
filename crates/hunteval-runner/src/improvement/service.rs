use std::{collections::BTreeSet, path::Path};

use hunteval_domain::{
    ImprovementEquivalenceResult, ImprovementEquivalenceStatus, ImprovementExperiment, Sha256Digest,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkRunSummary, BenchmarkService};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementRunSummary {
    pub experiment_id: String,
    pub benchmark: BenchmarkRunSummary,
    pub paired_cells: usize,
    pub all_pairs_terminal: bool,
    pub candidate_artifact_sha256: Sha256Digest,
}

#[derive(Debug)]
pub struct ImprovementService<'a> {
    benchmark: BenchmarkService<'a>,
}

#[derive(Debug, Clone, Copy)]
pub struct ImprovementRunRequest<'a> {
    pub experiment: &'a ImprovementExperiment,
    pub experiment_sha256: Sha256Digest,
    pub equivalence: &'a ImprovementEquivalenceResult,
    pub current_candidate_sha256: Sha256Digest,
    pub plan: &'a BenchmarkExecutionPlan,
    pub output_root: &'a Path,
    pub options: BenchmarkRunOptions,
}

impl<'a> ImprovementService<'a> {
    #[must_use]
    pub const fn new(benchmark: BenchmarkService<'a>) -> Self {
        Self { benchmark }
    }

    pub fn run(
        &self,
        request: ImprovementRunRequest<'_>,
    ) -> Result<ImprovementRunSummary, ImprovementServiceError> {
        validate_inputs(
            request.experiment,
            request.experiment_sha256,
            request.equivalence,
            request.current_candidate_sha256,
            request.plan,
        )?;
        let benchmark = self
            .benchmark
            .run(request.plan, request.output_root, request.options)?;
        Ok(ImprovementRunSummary {
            experiment_id: request.experiment.id.clone(),
            paired_cells: request.experiment.paired_cells.len(),
            all_pairs_terminal: benchmark.pending == 0,
            benchmark,
            candidate_artifact_sha256: request.current_candidate_sha256,
        })
    }
}

fn validate_inputs(
    experiment: &ImprovementExperiment,
    experiment_sha256: Sha256Digest,
    equivalence: &ImprovementEquivalenceResult,
    current_candidate_sha256: Sha256Digest,
    plan: &BenchmarkExecutionPlan,
) -> Result<(), ImprovementServiceError> {
    experiment
        .validate()
        .map_err(|_| ImprovementServiceError::InvalidExperiment)?;
    if equivalence.status != ImprovementEquivalenceStatus::Eligible
        || equivalence.experiment_sha256 != experiment_sha256
        || current_candidate_sha256 != experiment.candidate_artifact_sha256
    {
        return Err(ImprovementServiceError::IneligibleOrStale);
    }
    let cells = plan
        .definition
        .cells()
        .map_err(crate::BenchmarkServiceError::Definition)?;
    let cell_ids = cells
        .iter()
        .map(|cell| cell.cell_id.to_string())
        .collect::<BTreeSet<_>>();
    if experiment.paired_cells.iter().any(|pair| {
        !cell_ids.contains(&pair.baseline_cell_id) || !cell_ids.contains(&pair.candidate_cell_id)
    }) {
        return Err(ImprovementServiceError::UnpairedMatrix);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ImprovementServiceError {
    #[error("improvement experiment is invalid")]
    InvalidExperiment,
    #[error("improvement experiment is ineligible or candidate bytes are stale")]
    IneligibleOrStale,
    #[error("paired cells do not resolve in the canonical benchmark plan")]
    UnpairedMatrix,
    #[error("canonical benchmark service failed: {0}")]
    Benchmark(#[from] crate::BenchmarkServiceError),
}
