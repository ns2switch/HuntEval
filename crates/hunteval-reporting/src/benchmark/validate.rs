use std::{collections::BTreeSet, path::Path};

use hunteval_domain::SchemaVersion;

use super::types::{BenchmarkClaimSource, BenchmarkResult, BenchmarkResultError};

impl BenchmarkResult {
    pub fn validate(&self) -> Result<(), BenchmarkResultError> {
        if self.schema_version != SchemaVersion::new(0, 4)
            || self.cells.len() > 1_000_000
            || self.deployments.is_empty()
            || self.claims.len() > 100_000
            || self.limitations.len() > 1_024
            || self.artifacts.iter().any(|artifact| {
                !safe_relative(&artifact.path) || !valid_label(&artifact.path, 4_096)
            })
        {
            return Err(BenchmarkResultError::InvalidContract);
        }
        let cells = self
            .cells
            .iter()
            .map(|cell| cell.cell_id)
            .collect::<BTreeSet<_>>();
        if cells.len() != self.cells.len()
            || self
                .cells
                .iter()
                .any(|cell| !valid_cell(cell) || cell.artifacts.len() > 256)
            || self
                .deployments
                .windows(2)
                .any(|pair| pair[0].deployment_id.as_str() >= pair[1].deployment_id.as_str())
            || self
                .comparisons
                .windows(2)
                .any(|pair| pair[0].comparison_id >= pair[1].comparison_id)
        {
            return Err(BenchmarkResultError::InvalidContract);
        }
        let deployments = self
            .deployments
            .iter()
            .map(|item| &item.deployment_id)
            .collect::<BTreeSet<_>>();
        if self.comparisons.iter().any(|comparison| {
            !deployments.contains(&comparison.left)
                || !deployments.contains(&comparison.right)
                || comparison.cell_ids.iter().any(|cell| !cells.contains(cell))
        }) || self.rankings.iter().any(|group| {
            group.rank == 0
                || group.deployments.is_empty()
                || group
                    .deployments
                    .iter()
                    .any(|deployment| !deployments.contains(deployment))
        }) {
            return Err(BenchmarkResultError::InvalidSource);
        }
        validate_claims(self, &cells)
    }
}

fn valid_cell(cell: &super::types::BenchmarkCellSummary) -> bool {
    matches!(
        cell.status.as_str(),
        "pending" | "running" | "completed" | "failed" | "non_comparable"
    ) && cell
        .reason_code
        .as_ref()
        .is_none_or(|reason| valid_label(reason, 256))
        && cell.aggregate_score.is_none_or(valid_score)
        && cell.constraints.iter().all(|constraint| {
            valid_identifier(&constraint.code)
                && matches!(
                    constraint.status.as_str(),
                    "satisfied" | "violated" | "unverifiable"
                )
        })
        && cell
            .aggregate_score_omissions
            .iter()
            .all(|(metric, reason)| valid_identifier(metric) && valid_label(reason, 4_096))
        && cell
            .metrics
            .values()
            .all(|metric| metric.validate().is_ok())
}

fn validate_claims(
    report: &BenchmarkResult,
    cells: &BTreeSet<hunteval_domain::BenchmarkCellId>,
) -> Result<(), BenchmarkResultError> {
    let comparisons = report
        .comparisons
        .iter()
        .map(|item| item.comparison_id.as_str())
        .collect::<BTreeSet<_>>();
    let runs = report
        .cells
        .iter()
        .filter_map(|cell| cell.run_id.as_ref())
        .collect::<BTreeSet<_>>();
    let constraints = report
        .cells
        .iter()
        .flat_map(|cell| {
            cell.constraints
                .iter()
                .map(move |constraint| (cell.cell_id.to_string(), constraint.code.as_str()))
        })
        .collect::<BTreeSet<_>>();
    let mut identifiers = BTreeSet::new();
    for claim in &report.claims {
        if !valid_identifier(&claim.claim_id)
            || !identifiers.insert(claim.claim_id.as_str())
            || !valid_label(&claim.text, 16_384)
            || claim.sources.is_empty()
            || claim.sources.len() > 1_024
        {
            return Err(BenchmarkResultError::InvalidSource);
        }
        for source in &claim.sources {
            let valid = match source {
                BenchmarkClaimSource::BenchmarkCell {
                    benchmark_id,
                    cell_id,
                } => benchmark_id == &report.benchmark_id && cells.contains(cell_id),
                BenchmarkClaimSource::MetricPointer { run_id, pointer } => {
                    runs.contains(run_id) && valid_pointer(pointer)
                }
                BenchmarkClaimSource::Constraint {
                    scope_id,
                    constraint_id,
                } => constraints.contains(&(scope_id.clone(), constraint_id.as_str())),
                BenchmarkClaimSource::StatisticalComparison { comparison_id } => {
                    comparisons.contains(comparison_id.as_str())
                }
                BenchmarkClaimSource::ArtifactDigest { artifact, sha256 } => report
                    .artifacts
                    .iter()
                    .any(|item| &item.path == artifact && &item.sha256 == sha256),
            };
            if !valid {
                return Err(BenchmarkResultError::InvalidSource);
            }
        }
    }
    Ok(())
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && !Path::new(value)
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir))
}

fn valid_score(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn valid_label(value: &str, maximum: usize) -> bool {
    !value.trim().is_empty() && value.len() <= maximum && !value.contains('\0')
}

fn valid_identifier(value: &str) -> bool {
    value.len() <= 128
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
}

fn valid_pointer(value: &str) -> bool {
    value.starts_with('/') && value.len() <= 4_096 && !value.contains('~')
}
