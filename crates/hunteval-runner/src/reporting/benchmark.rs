use std::{collections::BTreeMap, path::Path};

use hunteval_domain::{BenchmarkCell, SchemaVersion};
use hunteval_reporting::{
    ArtifactDigestReference, BenchmarkArtifact, BenchmarkCellSummary, BenchmarkConstraintSummary,
    BenchmarkResult,
};

use crate::benchmark::BenchmarkCellResult;
use crate::{
    BenchmarkCellState, BenchmarkCellStatus, BenchmarkJournal, hash_file, load_stored_definition,
};

use super::{
    ReportGenerationError,
    aggregation::{claims, comparisons, deployment_summaries, rankings},
    io::read_verified,
};

pub(super) fn build(root: &Path) -> Result<BenchmarkResult, ReportGenerationError> {
    let definition = load_stored_definition(root)?;
    definition.validate()?;
    let journal = BenchmarkJournal::open(root, definition.id.clone())?;
    let state = journal.state().ok_or(ReportGenerationError::MissingState)?;
    drop(journal);
    let definition_hash = hash_file(&root.join("benchmark-definition.json"))?;
    let state_hash = hash_file(&root.join("benchmark-state.json"))?;
    let cells = definition.cells()?;
    let states = state
        .cells
        .iter()
        .map(|item| (item.cell_id, item))
        .collect::<BTreeMap<_, _>>();
    let mut loaded = BTreeMap::new();
    let mut summaries = Vec::with_capacity(cells.len());
    let mut artifacts = base_artifacts(root)?;
    for cell in &cells {
        let state = states
            .get(&cell.cell_id)
            .copied()
            .ok_or(ReportGenerationError::InvalidInput)?;
        let result = load_cell(root, cell, state)?;
        if let Some(result) = result {
            let result_path = format!("runs/{}/result.json", result.run_id.as_str());
            let digest = state
                .result_sha256
                .ok_or(ReportGenerationError::InvalidInput)?;
            artifacts.push(BenchmarkArtifact {
                path: result_path,
                sha256: digest,
            });
            summaries.push(completed_summary(cell, state, &result));
            loaded.insert(cell.cell_id, result);
        } else {
            summaries.push(incomplete_summary(cell, state));
        }
    }
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let mut deployments = deployment_summaries(&definition, &summaries, &loaded)?;
    deployments.sort_by(|left, right| {
        left.deployment_id
            .as_str()
            .cmp(right.deployment_id.as_str())
    });
    let comparisons = comparisons(root, &definition, &cells, &loaded)?;
    let rankings = rankings(&deployments);
    let claims = claims(&definition, &deployments, &comparisons, &summaries);
    let mut limitations = vec![
        "Attribution is observational and does not establish causality.".to_owned(),
        "Statistical intervals describe only the configured benchmark repetitions.".to_owned(),
    ];
    let unavailable = summaries
        .iter()
        .filter(|cell| cell.status != "completed")
        .count();
    if unavailable > 0 {
        limitations.push(format!(
            "{unavailable} benchmark cells were unavailable and excluded from numeric summaries."
        ));
    }
    let result = BenchmarkResult {
        schema_version: SchemaVersion::new(0, 4),
        benchmark_id: definition.id,
        benchmark_definition_sha256: definition_hash,
        benchmark_state_sha256: state_hash,
        scoring_profile_sha256: definition.scoring_profile.sha256,
        cells: summaries,
        deployments,
        comparisons,
        rankings,
        claims,
        artifacts,
        limitations,
    };
    result.validate()?;
    Ok(result)
}

fn base_artifacts(root: &Path) -> Result<Vec<BenchmarkArtifact>, ReportGenerationError> {
    [
        "benchmark-definition.json",
        "benchmark-events.jsonl",
        "benchmark-state.json",
    ]
    .into_iter()
    .map(|path| {
        Ok(BenchmarkArtifact {
            path: path.to_owned(),
            sha256: hash_file(&root.join(path))?,
        })
    })
    .collect()
}

fn load_cell(
    root: &Path,
    cell: &BenchmarkCell,
    state: &BenchmarkCellState,
) -> Result<Option<BenchmarkCellResult>, ReportGenerationError> {
    if state.status != BenchmarkCellStatus::Completed {
        return Ok(None);
    }
    let run_id = state
        .run_id
        .as_ref()
        .ok_or(ReportGenerationError::InvalidInput)?;
    let expected = state
        .result_sha256
        .ok_or(ReportGenerationError::InvalidInput)?;
    let path = root.join("runs").join(run_id.as_str()).join("result.json");
    let bytes = read_verified(&path, expected)?;
    let result: BenchmarkCellResult = serde_json::from_slice(&bytes)?;
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
        return Err(ReportGenerationError::InvalidInput);
    }
    Ok(Some(result))
}

fn completed_summary(
    cell: &BenchmarkCell,
    state: &BenchmarkCellState,
    result: &BenchmarkCellResult,
) -> BenchmarkCellSummary {
    BenchmarkCellSummary {
        cell_id: cell.cell_id,
        deployment_id: cell.key.deployment.id.clone(),
        episode_id: cell.key.episode.id.clone(),
        seed: cell.key.seed,
        status: "completed".to_owned(),
        reason_code: None,
        run_id: Some(result.run_id.clone()),
        result_sha256: state.result_sha256,
        aggregate_score: result.aggregate_score.value,
        aggregate_score_omissions: result.aggregate_score.omitted_metrics.clone(),
        metrics: result.metrics.0.clone(),
        constraints: result
            .constraints
            .iter()
            .map(|constraint| BenchmarkConstraintSummary {
                code: constraint.code.clone(),
                status: constraint_status(constraint.status).to_owned(),
                disqualifying: constraint.disqualifying,
            })
            .collect(),
        resource_usage: Some(result.resource_usage.clone()),
        submitted_timeline: result.submission.timeline.clone().unwrap_or_default(),
        artifacts: result
            .artifact_hashes
            .iter()
            .map(|(artifact, sha256)| ArtifactDigestReference {
                artifact: artifact.clone(),
                sha256: *sha256,
            })
            .collect(),
    }
}

fn incomplete_summary(cell: &BenchmarkCell, state: &BenchmarkCellState) -> BenchmarkCellSummary {
    BenchmarkCellSummary {
        cell_id: cell.cell_id,
        deployment_id: cell.key.deployment.id.clone(),
        episode_id: cell.key.episode.id.clone(),
        seed: cell.key.seed,
        status: status_name(state.status).to_owned(),
        reason_code: state.reason_code.clone(),
        run_id: state.run_id.clone(),
        result_sha256: state.result_sha256,
        aggregate_score: None,
        aggregate_score_omissions: BTreeMap::new(),
        metrics: BTreeMap::new(),
        constraints: Vec::new(),
        resource_usage: None,
        submitted_timeline: Vec::new(),
        artifacts: Vec::new(),
    }
}

const fn constraint_status(status: hunteval_evaluation::ConstraintStatus) -> &'static str {
    match status {
        hunteval_evaluation::ConstraintStatus::Satisfied => "satisfied",
        hunteval_evaluation::ConstraintStatus::Violated => "violated",
        hunteval_evaluation::ConstraintStatus::Unverifiable => "unverifiable",
    }
}

const fn status_name(status: BenchmarkCellStatus) -> &'static str {
    match status {
        BenchmarkCellStatus::Pending => "pending",
        BenchmarkCellStatus::Running => "running",
        BenchmarkCellStatus::Completed => "completed",
        BenchmarkCellStatus::Failed => "failed",
        BenchmarkCellStatus::NonComparable => "non_comparable",
    }
}
