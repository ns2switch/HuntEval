use std::{collections::BTreeSet, fs, path::Path};

use hunteval_domain::{
    BenchmarkCell, BenchmarkCellId, BenchmarkDefinition, DeploymentId, EpisodeId, FinalSubmission,
    RunId, SchemaVersion, Sha256Digest,
};
use hunteval_statistics::{
    StabilityInput, StabilitySample, StabilitySummary, UnavailableRepetition,
    UnavailableRepetitionReason, evaluate_stability,
};
use serde::{Deserialize, Serialize};

use crate::benchmark::{BenchmarkCellState, BenchmarkCellStatus, BenchmarkJournal};

use super::{
    BenchmarkService, BenchmarkServiceError, production::BenchmarkCellResult,
    storage::verify_definition,
};

const MAX_RESULT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMetricGroup {
    pub deployment_id: DeploymentId,
    pub episode_id: EpisodeId,
    pub contributing_cell_ids: Vec<BenchmarkCellId>,
    pub stability: StabilitySummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMetrics {
    pub schema_version: SchemaVersion,
    pub groups: Vec<BenchmarkMetricGroup>,
}

impl BenchmarkService<'_> {
    pub fn metrics(
        output_root: &Path,
        definition: &BenchmarkDefinition,
    ) -> Result<BenchmarkMetrics, BenchmarkServiceError> {
        verify_definition(output_root, definition)?;
        let journal = BenchmarkJournal::open(output_root, definition.id.clone())?;
        let state = journal.state().ok_or(BenchmarkServiceError::MissingState)?;
        let cells = definition.cells()?;
        let mut groups = Vec::with_capacity(
            definition
                .deployments
                .len()
                .saturating_mul(definition.episodes.len()),
        );
        for deployment in &definition.deployments {
            for episode in &definition.episodes {
                groups.push(aggregate_group(
                    output_root,
                    definition,
                    &cells,
                    &state.cells,
                    &deployment.id,
                    &episode.id,
                )?);
            }
        }
        Ok(BenchmarkMetrics {
            schema_version: SchemaVersion::new(0, 4),
            groups,
        })
    }
}

fn aggregate_group(
    output_root: &Path,
    definition: &BenchmarkDefinition,
    cells: &[BenchmarkCell],
    states: &[BenchmarkCellState],
    deployment_id: &DeploymentId,
    episode_id: &EpisodeId,
) -> Result<BenchmarkMetricGroup, BenchmarkServiceError> {
    let mut samples = Vec::new();
    let mut unavailable = Vec::new();
    let mut contributing_cell_ids = Vec::new();
    for seed in &definition.seeds {
        let cell = cells.iter().find(|cell| {
            &cell.key.deployment.id == deployment_id
                && &cell.key.episode.id == episode_id
                && cell.key.seed == *seed
        });
        let Some(cell) = cell else {
            unavailable.push(unavailable_seed(
                *seed,
                UnavailableRepetitionReason::Missing,
            ));
            continue;
        };
        let state = states.iter().find(|state| state.cell_id == cell.cell_id);
        match load_sample(output_root, cell, state) {
            Ok(sample) => {
                contributing_cell_ids.push(cell.cell_id);
                samples.push(sample);
            }
            Err(reason) => unavailable.push(unavailable_seed(*seed, reason)),
        }
    }
    let stability = evaluate_stability(StabilityInput {
        required_seeds: definition.seeds.clone(),
        samples,
        unavailable,
    })?;
    Ok(BenchmarkMetricGroup {
        deployment_id: deployment_id.clone(),
        episode_id: episode_id.clone(),
        contributing_cell_ids,
        stability,
    })
}

fn load_sample(
    output_root: &Path,
    cell: &BenchmarkCell,
    state: Option<&BenchmarkCellState>,
) -> Result<StabilitySample, UnavailableRepetitionReason> {
    let state = state.ok_or(UnavailableRepetitionReason::Missing)?;
    if state.status != BenchmarkCellStatus::Completed {
        return Err(UnavailableRepetitionReason::Failed);
    }
    let run_id = state
        .run_id
        .as_ref()
        .ok_or(UnavailableRepetitionReason::InvalidArtifact)?;
    let expected = state
        .result_sha256
        .ok_or(UnavailableRepetitionReason::InvalidArtifact)?;
    let bytes = read_verified_result(output_root, run_id, expected)?;
    let result: BenchmarkCellResult =
        serde_json::from_slice(&bytes).map_err(|_| UnavailableRepetitionReason::InvalidArtifact)?;
    validate_result(&result, cell, run_id)?;
    Ok(StabilitySample {
        seed: cell.key.seed,
        submission_claims: submission_claims(&result.submission),
        metrics: result
            .metrics
            .0
            .into_iter()
            .filter_map(|(name, metric)| metric.value.map(|value| (name, value)))
            .collect(),
    })
}

fn read_verified_result(
    output_root: &Path,
    run_id: &RunId,
    expected: Sha256Digest,
) -> Result<Vec<u8>, UnavailableRepetitionReason> {
    let path = output_root
        .join("runs")
        .join(run_id.as_str())
        .join("result.json");
    let metadata =
        fs::symlink_metadata(&path).map_err(|_| UnavailableRepetitionReason::InvalidArtifact)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_RESULT_BYTES
    {
        return Err(UnavailableRepetitionReason::InvalidArtifact);
    }
    let bytes = fs::read(path).map_err(|_| UnavailableRepetitionReason::InvalidArtifact)?;
    if Sha256Digest::from_bytes(&bytes) != expected {
        return Err(UnavailableRepetitionReason::InvalidArtifact);
    }
    Ok(bytes)
}

fn validate_result(
    result: &BenchmarkCellResult,
    cell: &BenchmarkCell,
    run_id: &RunId,
) -> Result<(), UnavailableRepetitionReason> {
    if result.schema_version != SchemaVersion::new(0, 4)
        || result.cell_id != cell.cell_id
        || &result.run_id != run_id
        || &result.cell != cell
        || result.metrics.0.is_empty()
        || result
            .metrics
            .0
            .values()
            .any(|metric| metric.validate().is_err())
        || result.resource_usage.validate().is_err()
        || result.submission.validate().is_err()
    {
        Err(UnavailableRepetitionReason::InvalidArtifact)
    } else {
        Ok(())
    }
}

fn submission_claims(submission: &FinalSubmission) -> BTreeSet<String> {
    let mut claims = BTreeSet::new();
    claims.insert(format!("status:{}", status_name(submission.status)));
    claims.insert(format!("confidence:{}", submission.confidence.get()));
    claims.extend(
        submission
            .malicious_event_ids
            .iter()
            .map(|id| format!("event:{}", id.as_str())),
    );
    claims.extend(
        submission
            .malicious_entity_ids
            .iter()
            .map(|id| format!("entity:{id}")),
    );
    claims.extend(
        submission
            .attack_techniques
            .iter()
            .map(|id| format!("technique:{id}")),
    );
    claims.extend(
        submission
            .attack_path
            .iter()
            .enumerate()
            .map(|(index, id)| format!("path:{index}:{}", id.as_str())),
    );
    for (index, entry) in submission.timeline.iter().flatten().enumerate() {
        claims.insert(format!(
            "timeline:{index}:{}:{}",
            entry.event_id.as_str(),
            entry.observed_at
        ));
    }
    claims
}

const fn status_name(status: hunteval_domain::SubmissionStatus) -> &'static str {
    match status {
        hunteval_domain::SubmissionStatus::ConfirmedMaliciousActivity => {
            "confirmed_malicious_activity"
        }
        hunteval_domain::SubmissionStatus::SuspiciousActivity => "suspicious_activity",
        hunteval_domain::SubmissionStatus::NoMaliciousActivity => "no_malicious_activity",
        hunteval_domain::SubmissionStatus::Inconclusive => "inconclusive",
    }
}

const fn unavailable_seed(seed: u64, reason: UnavailableRepetitionReason) -> UnavailableRepetition {
    UnavailableRepetition { seed, reason }
}
