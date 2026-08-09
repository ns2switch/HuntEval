use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkCellId, ContractValidationError, SchemaVersion, Sha256Digest, TopologyExperimentId,
};

const SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 6);
const MAX_CHANGES: usize = 32;
const MAX_PAIRED_CELLS: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlHashes {
    pub episodes: Sha256Digest,
    pub seeds: Sha256Digest,
    pub budgets: Sha256Digest,
    pub models: Sha256Digest,
    pub managed_tool_policy: Sha256Digest,
    pub scoring_profile: Sha256Digest,
    pub execution_policy: Sha256Digest,
    pub schemas: Sha256Digest,
    pub binaries: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyExperiment {
    pub schema_version: SchemaVersion,
    pub id: TopologyExperimentId,
    pub baseline_topology_sha256: Sha256Digest,
    pub candidate_topology_sha256: Sha256Digest,
    pub changed_variables: BTreeSet<String>,
    pub control_hashes: ControlHashes,
    pub paired_cell_ids: BTreeSet<BenchmarkCellId>,
}

impl TopologyExperiment {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        if self.baseline_topology_sha256 == self.candidate_topology_sha256 {
            return Err(invalid(
                "candidate_topology_sha256",
                "candidate topology must differ from baseline",
            ));
        }
        if self.changed_variables.is_empty() || self.changed_variables.len() > MAX_CHANGES {
            return Err(invalid(
                "changed_variables",
                "changed-variable count is outside the supported bound",
            ));
        }
        if self
            .changed_variables
            .iter()
            .any(|path| !valid_pointer(path))
        {
            return Err(invalid(
                "changed_variables",
                "changed variable is not a bounded JSON pointer",
            ));
        }
        if self.paired_cell_ids.len() < 2 || self.paired_cell_ids.len() > MAX_PAIRED_CELLS {
            return Err(invalid(
                "paired_cell_ids",
                "paired-cell count is outside the supported bound",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquivalenceStatus {
    Eligible,
    Ineligible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyEquivalenceResult {
    pub schema_version: SchemaVersion,
    pub experiment_sha256: Sha256Digest,
    pub status: EquivalenceStatus,
    pub declared_changes: BTreeSet<String>,
    pub observed_changes: BTreeSet<String>,
    pub mismatch_reason_codes: BTreeSet<String>,
}

impl TopologyEquivalenceResult {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        if self.declared_changes.len() > MAX_CHANGES
            || self.observed_changes.len() > MAX_CHANGES
            || self
                .declared_changes
                .iter()
                .chain(&self.observed_changes)
                .any(|path| !valid_pointer(path))
        {
            return Err(invalid(
                "changes",
                "change inventory is malformed or unbounded",
            ));
        }
        let equivalent = self.declared_changes == self.observed_changes;
        match self.status {
            EquivalenceStatus::Eligible
                if !equivalent || !self.mismatch_reason_codes.is_empty() =>
            {
                Err(invalid("status", "eligible result contains a mismatch"))
            }
            EquivalenceStatus::Ineligible
                if equivalent && self.mismatch_reason_codes.is_empty() =>
            {
                Err(invalid("status", "ineligible result requires a mismatch"))
            }
            _ => Ok(()),
        }
    }
}

fn require_version(version: SchemaVersion) -> Result<(), ContractValidationError> {
    if version != SCHEMA_VERSION {
        return Err(invalid("schema_version", "schema version is unsupported"));
    }
    Ok(())
}

fn valid_pointer(value: &str) -> bool {
    value.starts_with('/')
        && value.len() <= 1_024
        && !value.contains(['\0', '\n', '\r'])
        && !value.split('/').skip(1).any(str::is_empty)
}

fn invalid(field: &'static str, reason: &'static str) -> ContractValidationError {
    ContractValidationError::new(field, reason)
}
