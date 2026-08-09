use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    DiagnosticRecurrenceGroup, DiagnosticSourceReference, ExcludedDiagnosticCell,
    FailureClassification, RecurrenceClaimStrength, RunId, SchemaVersion, Sha256Digest,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparableDiagnosticCell {
    pub cell_id: String,
    pub run_id: Option<RunId>,
    pub deployment_sha256: Sha256Digest,
    pub configuration_sha256: Sha256Digest,
    pub topology_sha256: Sha256Digest,
    pub cell_artifact_sha256: Sha256Digest,
    pub classifications: Vec<FailureClassification>,
    pub exclusion_reason: Option<String>,
}

pub fn reduce_recurrence(
    cells: &[ComparableDiagnosticCell],
) -> Result<Vec<DiagnosticRecurrenceGroup>, RecurrenceError> {
    if cells.len() > 1_000_000 {
        return Err(RecurrenceError::TooManyCells);
    }
    let mut ids = BTreeSet::new();
    if cells.iter().any(|cell| !ids.insert(&cell.cell_id)) {
        return Err(RecurrenceError::DuplicateCell);
    }
    type CohortKey = (Sha256Digest, Sha256Digest, Sha256Digest);
    let mut cohorts: BTreeMap<CohortKey, Vec<&ComparableDiagnosticCell>> = BTreeMap::new();
    for cell in cells {
        cohorts
            .entry((
                cell.deployment_sha256,
                cell.configuration_sha256,
                cell.topology_sha256,
            ))
            .or_default()
            .push(cell);
    }
    let mut groups = Vec::new();
    for ((deployment, configuration, topology), cohort) in cohorts {
        let eligible: Vec<_> = cohort
            .iter()
            .filter(|cell| cell.exclusion_reason.is_none())
            .copied()
            .collect();
        let codes: BTreeSet<_> = eligible
            .iter()
            .flat_map(|cell| cell.classifications.iter().map(|item| item.code.clone()))
            .collect();
        for code in codes {
            let affected: Vec<_> = eligible
                .iter()
                .filter(|cell| cell.classifications.iter().any(|item| item.code == code))
                .copied()
                .collect();
            let taxonomy_sha256 = affected
                .first()
                .and_then(|cell| cell.classifications.iter().find(|item| item.code == code))
                .map(|item| item.taxonomy_sha256)
                .ok_or(RecurrenceError::InvalidClassification)?;
            if affected.iter().any(|cell| {
                cell.classifications
                    .iter()
                    .find(|item| item.code == code)
                    .is_none_or(|item| item.taxonomy_sha256 != taxonomy_sha256)
            }) {
                return Err(RecurrenceError::TaxonomyDrift);
            }
            let sources = affected
                .iter()
                .map(|cell| DiagnosticSourceReference::BenchmarkCell {
                    cell_id: cell.cell_id.clone(),
                    artifact_sha256: cell.cell_artifact_sha256,
                })
                .collect();
            let affected_run_ids = affected
                .iter()
                .filter_map(|cell| cell.run_id.clone())
                .collect();
            let affected_cell_ids = affected.iter().map(|cell| cell.cell_id.clone()).collect();
            let excluded_cells = cohort
                .iter()
                .filter_map(|cell| {
                    cell.exclusion_reason
                        .as_ref()
                        .map(|reason| ExcludedDiagnosticCell {
                            cell_id: cell.cell_id.clone(),
                            reason_code: reason.clone(),
                        })
                })
                .collect();
            let id = recurrence_id(
                &code,
                deployment,
                configuration,
                topology,
                &affected_cell_ids,
            );
            groups.push(DiagnosticRecurrenceGroup {
                schema_version: SchemaVersion::new(0, 7),
                id,
                taxonomy_sha256,
                code,
                deployment_sha256: deployment,
                configuration_sha256: configuration,
                topology_sha256: topology,
                eligible_samples: eligible.len(),
                occurrences: affected.len(),
                affected_run_ids,
                affected_cell_ids,
                excluded_cells,
                claim_strength: RecurrenceClaimStrength::Descriptive,
                sources,
                limitations: ["recurrence_is_not_causality".to_owned()]
                    .into_iter()
                    .collect(),
            });
        }
    }
    groups.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(groups)
}

fn recurrence_id(
    code: &str,
    deployment: Sha256Digest,
    configuration: Sha256Digest,
    topology: Sha256Digest,
    cells: &BTreeSet<String>,
) -> String {
    let mut bytes = format!("{code}\n{deployment}\n{configuration}\n{topology}\n").into_bytes();
    for cell in cells {
        bytes.extend_from_slice(cell.as_bytes());
        bytes.push(b'\n');
    }
    format!("recurrence:{}", Sha256Digest::from_bytes(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RecurrenceError {
    #[error("diagnostic recurrence input exceeds its cell bound")]
    TooManyCells,
    #[error("diagnostic recurrence input contains duplicate cells")]
    DuplicateCell,
    #[error("diagnostic recurrence contains an invalid classification")]
    InvalidClassification,
    #[error("diagnostic recurrence cannot combine taxonomy versions")]
    TaxonomyDrift,
}
