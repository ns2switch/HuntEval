use std::{collections::BTreeMap, path::Path};

use hunteval_domain::{BenchmarkCell, BenchmarkCellId, BenchmarkDefinition, DeploymentId};
use hunteval_evaluation::ConstraintStatus;
use hunteval_reporting::{
    BenchmarkCellSummary, BenchmarkClaim, BenchmarkClaimSource, BenchmarkDeploymentSummary,
    BenchmarkMetricSummary, BenchmarkPairwiseComparison, BenchmarkRankingGroup,
};
use hunteval_statistics::{RankingEntry, paired_difference, rank, summarize};

use crate::benchmark::BenchmarkCellResult;
use crate::{BenchmarkService, ComparisonStatus};

use super::ReportGenerationError;

pub(super) fn deployment_summaries(
    definition: &BenchmarkDefinition,
    cells: &[BenchmarkCellSummary],
    loaded: &BTreeMap<BenchmarkCellId, BenchmarkCellResult>,
) -> Result<Vec<BenchmarkDeploymentSummary>, ReportGenerationError> {
    definition
        .deployments
        .iter()
        .enumerate()
        .map(|(index, deployment)| summarize_deployment(deployment, index, cells, loaded))
        .collect()
}

fn summarize_deployment(
    deployment: &hunteval_domain::ResolvedDeployment,
    index: usize,
    cells: &[BenchmarkCellSummary],
    loaded: &BTreeMap<BenchmarkCellId, BenchmarkCellResult>,
) -> Result<BenchmarkDeploymentSummary, ReportGenerationError> {
    let selected = cells
        .iter()
        .filter(|cell| cell.deployment_id == deployment.id)
        .collect::<Vec<_>>();
    let values = selected
        .iter()
        .filter_map(|cell| cell.aggregate_score)
        .collect::<Vec<_>>();
    let metric_names = selected
        .iter()
        .flat_map(|cell| cell.metrics.keys().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    let mut metrics = BTreeMap::new();
    for metric in metric_names {
        let by_cell = selected
            .iter()
            .map(|cell| {
                (
                    cell.cell_id,
                    cell.metrics.get(&metric).and_then(|value| value.value),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let numeric = by_cell
            .values()
            .filter_map(|value| *value)
            .collect::<Vec<_>>();
        metrics.insert(
            metric.clone(),
            BenchmarkMetricSummary {
                metric,
                values: by_cell,
                statistics: summarize(&numeric, index as u64 + 1)?,
            },
        );
    }
    let disqualifying_constraints = selected
        .iter()
        .filter_map(|cell| loaded.get(&cell.cell_id))
        .flat_map(|result| &result.constraints)
        .filter(|constraint| {
            constraint.disqualifying && constraint.status != ConstraintStatus::Satisfied
        })
        .count();
    Ok(BenchmarkDeploymentSummary {
        deployment_id: deployment.id.clone(),
        completed_cells: count_status(&selected, "completed"),
        failed_cells: count_status(&selected, "failed"),
        pending_cells: count_status(&selected, "pending") + count_status(&selected, "running"),
        non_comparable_cells: count_status(&selected, "non_comparable"),
        disqualifying_constraints,
        aggregate_score: summarize(&values, index as u64 + 1)?,
        metrics,
    })
}

fn count_status(cells: &[&BenchmarkCellSummary], status: &str) -> usize {
    cells.iter().filter(|cell| cell.status == status).count()
}

pub(super) fn comparisons(
    root: &Path,
    definition: &BenchmarkDefinition,
    cells: &[BenchmarkCell],
    loaded: &BTreeMap<BenchmarkCellId, BenchmarkCellResult>,
) -> Result<Vec<BenchmarkPairwiseComparison>, ReportGenerationError> {
    let mut comparisons = Vec::new();
    for (left_index, left) in definition.deployments.iter().enumerate() {
        for right in definition.deployments.iter().skip(left_index + 1) {
            let eligibility = BenchmarkService::compare(root, definition, &left.id, &right.id)?;
            let (left_values, right_values) = paired_values(cells, loaded, &left.id, &right.id);
            comparisons.push(BenchmarkPairwiseComparison {
                comparison_id: eligibility.comparison_id,
                left: left.id.clone(),
                right: right.id.clone(),
                eligible: eligibility.status == ComparisonStatus::Eligible,
                reasons: eligibility
                    .reasons
                    .iter()
                    .map(|reason| format!("{reason:?}").to_ascii_lowercase())
                    .collect(),
                cell_ids: eligibility.cell_ids,
                aggregate_difference: paired_difference(
                    &left_values,
                    &right_values,
                    left_index as u64 + 1,
                )?,
            });
        }
    }
    comparisons.sort_by(|left, right| left.comparison_id.cmp(&right.comparison_id));
    Ok(comparisons)
}

fn paired_values(
    cells: &[BenchmarkCell],
    loaded: &BTreeMap<BenchmarkCellId, BenchmarkCellResult>,
    left: &DeploymentId,
    right: &DeploymentId,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let keys = cells
        .iter()
        .map(|cell| (cell.key.episode.id.clone(), cell.key.seed))
        .collect::<std::collections::BTreeSet<_>>();
    let values_for = |deployment: &DeploymentId| {
        keys.iter()
            .map(|(episode, seed)| {
                cells
                    .iter()
                    .find(|cell| {
                        &cell.key.deployment.id == deployment
                            && &cell.key.episode.id == episode
                            && cell.key.seed == *seed
                    })
                    .and_then(|cell| loaded.get(&cell.cell_id))
                    .and_then(|result| result.aggregate_score.value)
            })
            .collect()
    };
    (values_for(left), values_for(right))
}

pub(super) fn rankings(deployments: &[BenchmarkDeploymentSummary]) -> Vec<BenchmarkRankingGroup> {
    let entries = deployments
        .iter()
        .map(|deployment| RankingEntry {
            deployment: deployment.deployment_id.as_str().to_owned(),
            disqualifying_violations: deployment.disqualifying_constraints,
            aggregate_score: deployment.aggregate_score.mean,
            raw_metrics: deployment
                .metrics
                .iter()
                .map(|(name, metric)| (name.clone(), metric.statistics.mean))
                .collect(),
        })
        .collect();
    let mut groups: Vec<BenchmarkRankingGroup> = Vec::new();
    for entry in rank(entries) {
        let tie = groups.last_mut().filter(|group| {
            group.disqualifying_constraints == entry.disqualifying_violations
                && optional_score_equal(group.aggregate_score, entry.aggregate_score)
        });
        if let Some(group) = tie {
            if let Ok(id) = DeploymentId::new(entry.deployment) {
                group.deployments.push(id);
            }
        } else if let Ok(id) = DeploymentId::new(entry.deployment) {
            groups.push(BenchmarkRankingGroup {
                rank: groups.len() + 1,
                deployments: vec![id],
                disqualifying_constraints: entry.disqualifying_violations,
                aggregate_score: entry.aggregate_score,
            });
        }
    }
    groups
}

fn optional_score_equal(left: Option<f64>, right: Option<f64>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left.total_cmp(&right).is_eq(),
        (None, None) => true,
        _ => false,
    }
}

pub(super) fn claims(
    definition: &BenchmarkDefinition,
    deployments: &[BenchmarkDeploymentSummary],
    comparisons: &[BenchmarkPairwiseComparison],
    cells: &[BenchmarkCellSummary],
) -> Vec<BenchmarkClaim> {
    let deployment_claims = deployments.iter().map(|deployment| BenchmarkClaim {
        claim_id: format!("deployment:{}", deployment.deployment_id.as_str()),
        text: format!(
            "Deployment {} completed {} cells with mean aggregate score {}.",
            deployment.deployment_id.as_str(),
            deployment.completed_cells,
            deployment
                .aggregate_score
                .mean
                .map_or_else(|| "unavailable".to_owned(), |value| format!("{value:.6}"))
        ),
        sources: cells
            .iter()
            .filter(|cell| cell.deployment_id == deployment.deployment_id)
            .map(|cell| BenchmarkClaimSource::BenchmarkCell {
                benchmark_id: definition.id.clone(),
                cell_id: cell.cell_id,
            })
            .collect(),
    });
    let comparison_claims = comparisons.iter().map(|comparison| BenchmarkClaim {
        claim_id: comparison.comparison_id.clone(),
        text: format!(
            "Comparison of {} and {} used {} paired observations.",
            comparison.left.as_str(),
            comparison.right.as_str(),
            comparison.aggregate_difference.count
        ),
        sources: vec![BenchmarkClaimSource::StatisticalComparison {
            comparison_id: comparison.comparison_id.clone(),
        }],
    });
    deployment_claims.chain(comparison_claims).collect()
}
