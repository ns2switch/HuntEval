use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use hunteval_domain::{
    BenchmarkCell, BenchmarkCellId, BenchmarkDefinition, DeploymentId, SchemaVersion,
};
use serde::{Deserialize, Serialize};

use crate::benchmark::{BenchmarkCellState, BenchmarkCellStatus, BenchmarkState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonReason {
    MissingCell,
    CellNotCompleted,
    EpisodeHashMismatch,
    ScoringProfileMismatch,
    SchemaVersionMismatch,
    ProtocolVersionMismatch,
    BudgetMismatch,
    ConfigurationMismatch,
    SeedNotPaired,
    FaultPairMismatch,
    ArtifactVerificationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonStatus {
    Eligible,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonEligibility {
    pub schema_version: SchemaVersion,
    pub comparison_id: String,
    pub status: ComparisonStatus,
    pub cell_ids: Vec<BenchmarkCellId>,
    pub reasons: BTreeSet<ComparisonReason>,
}

impl ComparisonEligibility {
    pub(crate) fn evaluate(
        definition: &BenchmarkDefinition,
        state: &BenchmarkState,
        output_root: &Path,
        left: &DeploymentId,
        right: &DeploymentId,
    ) -> Result<Self, hunteval_domain::BenchmarkDefinitionError> {
        let cells = definition.cells()?;
        let states = state
            .cells
            .iter()
            .map(|cell| (cell.cell_id, cell))
            .collect::<BTreeMap<_, _>>();
        let mut selected = Vec::new();
        let mut reasons = BTreeSet::new();
        for episode in &definition.episodes {
            for seed in &definition.seeds {
                for deployment in [left, right] {
                    match find_cell(&cells, deployment, &episode.id, *seed) {
                        Some(cell) => {
                            selected.push(cell.cell_id);
                            check_state(
                                states.get(&cell.cell_id).copied(),
                                output_root,
                                &mut reasons,
                            );
                        }
                        None => {
                            reasons.insert(ComparisonReason::MissingCell);
                        }
                    }
                }
            }
        }
        if left == right || selected.len() < 2 {
            reasons.insert(ComparisonReason::ConfigurationMismatch);
        }
        let status = if reasons.is_empty() {
            ComparisonStatus::Eligible
        } else {
            ComparisonStatus::Ineligible
        };
        let comparison_key = format!("{}:{}", left.as_str(), right.as_str());
        Ok(Self {
            schema_version: SchemaVersion::new(0, 4),
            comparison_id: format!(
                "comparison:{}",
                hunteval_domain::Sha256Digest::from_bytes(comparison_key.as_bytes())
            ),
            status,
            cell_ids: selected,
            reasons,
        })
    }
}

fn find_cell<'a>(
    cells: &'a [BenchmarkCell],
    deployment: &DeploymentId,
    episode: &hunteval_domain::EpisodeId,
    seed: u64,
) -> Option<&'a BenchmarkCell> {
    cells.iter().find(|cell| {
        &cell.key.deployment.id == deployment
            && &cell.key.episode.id == episode
            && cell.key.seed == seed
    })
}

fn check_state(
    state: Option<&BenchmarkCellState>,
    output_root: &Path,
    reasons: &mut BTreeSet<ComparisonReason>,
) {
    match state {
        None => {
            reasons.insert(ComparisonReason::MissingCell);
        }
        Some(cell) if cell.status != BenchmarkCellStatus::Completed => {
            reasons.insert(ComparisonReason::CellNotCompleted);
        }
        Some(cell) if cell.result_sha256.is_none() || cell.run_id.is_none() => {
            reasons.insert(ComparisonReason::ArtifactVerificationFailed);
        }
        Some(cell) if !artifact_matches(cell, output_root) => {
            reasons.insert(ComparisonReason::ArtifactVerificationFailed);
        }
        Some(_) => {}
    }
}

fn artifact_matches(cell: &BenchmarkCellState, output_root: &Path) -> bool {
    let (Some(run_id), Some(expected)) = (&cell.run_id, cell.result_sha256) else {
        return false;
    };
    let path = output_root
        .join("runs")
        .join(run_id.as_str())
        .join("result.json");
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 16 * 1024 * 1024
    {
        return false;
    }
    fs::read(path)
        .map(hunteval_domain::Sha256Digest::from_bytes)
        .is_ok_and(|actual| actual == expected)
}
