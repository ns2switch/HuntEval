use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::DiagnosticSourceReference;
use crate::{RunId, SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludedDiagnosticCell {
    pub cell_id: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRecurrenceGroup {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub taxonomy_sha256: Sha256Digest,
    pub code: String,
    pub deployment_sha256: Sha256Digest,
    pub configuration_sha256: Sha256Digest,
    pub topology_sha256: Sha256Digest,
    pub eligible_samples: usize,
    pub occurrences: usize,
    pub affected_run_ids: BTreeSet<RunId>,
    pub affected_cell_ids: BTreeSet<String>,
    pub excluded_cells: Vec<ExcludedDiagnosticCell>,
    pub claim_strength: RecurrenceClaimStrength,
    pub sources: BTreeSet<DiagnosticSourceReference>,
    pub limitations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrenceClaimStrength {
    Descriptive,
}
