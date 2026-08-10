use std::{collections::BTreeSet, path::Path};

use hunteval_domain::{
    ImprovementEquivalenceResult, ImprovementEquivalenceStatus, ImprovementExperiment, Sha256Digest,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BenchmarkExecutionPlan, BenchmarkRunOptions, BenchmarkRunSummary, BenchmarkService};

const MAXIMUM_IMPROVEMENT_INPUT_BYTES: u64 = 1024 * 1024;

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

pub fn validate_improvement_inputs(
    experiment_path: &Path,
    equivalence_path: &Path,
    candidate_path: &Path,
    benchmark: Option<(&Path, &Path)>,
) -> Result<(), ImprovementInputError> {
    let experiment_bytes = safe_read(experiment_path)?;
    let equivalence_bytes = safe_read(equivalence_path)?;
    let candidate_bytes = safe_read(candidate_path)?;
    let experiment: ImprovementExperiment = serde_json::from_slice(&experiment_bytes)
        .map_err(|_| ImprovementInputError::InvalidContract)?;
    experiment
        .validate()
        .map_err(|_| ImprovementInputError::InvalidContract)?;
    let equivalence: ImprovementEquivalenceResult = serde_json::from_slice(&equivalence_bytes)
        .map_err(|_| ImprovementInputError::InvalidContract)?;
    if equivalence.status != ImprovementEquivalenceStatus::Eligible
        || equivalence.experiment_sha256 != Sha256Digest::from_bytes(&experiment_bytes)
        || experiment.candidate_artifact_sha256 != Sha256Digest::from_bytes(&candidate_bytes)
    {
        return Err(ImprovementInputError::IneligibleOrStale);
    }
    if let Some((manifest, artifact_root)) = benchmark {
        let definition = crate::resolve_benchmark(manifest, artifact_root)
            .map_err(|_| ImprovementInputError::InvalidBenchmark)?;
        let cell_ids = definition
            .cells()
            .map_err(|_| ImprovementInputError::InvalidBenchmark)?
            .into_iter()
            .map(|cell| cell.cell_id.to_string())
            .collect::<BTreeSet<_>>();
        if experiment.paired_cells.iter().any(|pair| {
            !cell_ids.contains(&pair.baseline_cell_id)
                || !cell_ids.contains(&pair.candidate_cell_id)
        }) {
            return Err(ImprovementInputError::UnpairedMatrix);
        }
    }
    Ok(())
}

fn safe_read(path: &Path) -> Result<Vec<u8>, ImprovementInputError> {
    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| ImprovementInputError::UnsafeInput)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAXIMUM_IMPROVEMENT_INPUT_BYTES
    {
        return Err(ImprovementInputError::UnsafeInput);
    }
    std::fs::read(path).map_err(|_| ImprovementInputError::UnsafeInput)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ImprovementInputError {
    #[error("improvement input is not a bounded regular file")]
    UnsafeInput,
    #[error("improvement input does not satisfy the normative contract")]
    InvalidContract,
    #[error("improvement inputs are ineligible or stale")]
    IneligibleOrStale,
    #[error("benchmark manifest or its referenced artifacts are invalid")]
    InvalidBenchmark,
    #[error("paired improvement cells do not resolve in the benchmark matrix")]
    UnpairedMatrix,
}
