use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const MAX_EDGES: usize = 256;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationAction {
    AdaptInMemory,
    ReadAsIs,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationEdge {
    pub edge_id: String,
    pub artifact_family: String,
    pub source_version: String,
    pub target_version: Option<String>,
    pub action: MigrationAction,
    pub implementation: Option<String>,
    pub fixture_path: Option<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationInventory {
    pub schema_version: String,
    pub inventory_id: String,
    pub compatibility_matrix_sha256: String,
    pub edges: Vec<MigrationEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationReceipt {
    pub schema_version: String,
    pub edge_id: String,
    pub source_sha256: String,
    pub target_sha256: String,
    pub implementation: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum MigrationError {
    #[error("migration inventory has an unsupported schema version")]
    UnsupportedVersion,
    #[error("migration inventory contains an invalid bounded value")]
    InvalidValue,
    #[error("migration inventory contains an ambiguous edge")]
    AmbiguousEdge,
    #[error("migration edge is not declared")]
    UndeclaredEdge,
    #[error("migration is explicitly rejected")]
    Rejected,
    #[error("migration input exceeds the byte limit")]
    Oversized,
    #[error("migration receipt does not match the supplied bytes")]
    ReceiptMismatch,
}

impl MigrationInventory {
    pub fn validate(&self) -> Result<(), MigrationError> {
        if self.schema_version != "1.0" {
            return Err(MigrationError::UnsupportedVersion);
        }
        if !identifier(&self.inventory_id)
            || !digest(&self.compatibility_matrix_sha256)
            || self.edges.is_empty()
            || self.edges.len() > MAX_EDGES
        {
            return Err(MigrationError::InvalidValue);
        }
        let mut identifiers = BTreeSet::new();
        let mut sources = BTreeSet::new();
        for edge in &self.edges {
            validate_edge(edge)?;
            if !identifiers.insert(edge.edge_id.as_str())
                || !sources.insert((edge.artifact_family.as_str(), edge.source_version.as_str()))
            {
                return Err(MigrationError::AmbiguousEdge);
            }
        }
        Ok(())
    }

    pub fn decision(
        &self,
        artifact_family: &str,
        source_version: &str,
    ) -> Result<&MigrationEdge, MigrationError> {
        self.validate()?;
        self.edges
            .iter()
            .find(|edge| {
                edge.artifact_family == artifact_family && edge.source_version == source_version
            })
            .ok_or(MigrationError::UndeclaredEdge)
    }

    pub fn receipt(
        &self,
        edge_id: &str,
        source: &[u8],
        target: &[u8],
    ) -> Result<MigrationReceipt, MigrationError> {
        self.validate()?;
        if source.len() > MAX_BYTES || target.len() > MAX_BYTES {
            return Err(MigrationError::Oversized);
        }
        let edge = self
            .edges
            .iter()
            .find(|edge| edge.edge_id == edge_id)
            .ok_or(MigrationError::UndeclaredEdge)?;
        if edge.action == MigrationAction::Reject {
            return Err(MigrationError::Rejected);
        }
        let implementation = edge
            .implementation
            .clone()
            .ok_or(MigrationError::InvalidValue)?;
        Ok(MigrationReceipt {
            schema_version: "1.0".to_owned(),
            edge_id: edge.edge_id.clone(),
            source_sha256: hash(source),
            target_sha256: hash(target),
            implementation,
        })
    }
}

impl MigrationReceipt {
    pub fn verify(&self, source: &[u8], target: &[u8]) -> Result<(), MigrationError> {
        if self.schema_version != "1.0" {
            return Err(MigrationError::UnsupportedVersion);
        }
        if source.len() > MAX_BYTES || target.len() > MAX_BYTES {
            return Err(MigrationError::Oversized);
        }
        if !identifier(&self.edge_id)
            || !identifier(&self.implementation)
            || !digest(&self.source_sha256)
            || !digest(&self.target_sha256)
        {
            return Err(MigrationError::InvalidValue);
        }
        if self.source_sha256 != hash(source) || self.target_sha256 != hash(target) {
            return Err(MigrationError::ReceiptMismatch);
        }
        Ok(())
    }
}

fn validate_edge(edge: &MigrationEdge) -> Result<(), MigrationError> {
    if !identifier(&edge.edge_id)
        || !identifier(&edge.artifact_family)
        || !version(&edge.source_version)
        || edge
            .target_version
            .as_deref()
            .is_some_and(|value| !version(value))
        || edge
            .implementation
            .as_deref()
            .is_some_and(|value| !identifier(value))
        || edge
            .fixture_path
            .as_deref()
            .is_some_and(|value| !safe_path(value))
        || edge
            .reason_code
            .as_deref()
            .is_some_and(|value| !identifier(value))
    {
        return Err(MigrationError::InvalidValue);
    }
    match edge.action {
        MigrationAction::AdaptInMemory => {
            if edge.target_version.is_none()
                || edge.implementation.is_none()
                || edge.fixture_path.is_none()
                || edge.reason_code.is_some()
                || edge.target_version.as_deref() == Some(edge.source_version.as_str())
            {
                return Err(MigrationError::InvalidValue);
            }
        }
        MigrationAction::ReadAsIs => {
            if edge.target_version.as_deref() != Some(edge.source_version.as_str())
                || edge.implementation.is_none()
                || edge.fixture_path.is_none()
                || edge.reason_code.is_some()
            {
                return Err(MigrationError::InvalidValue);
            }
        }
        MigrationAction::Reject => {
            if edge.target_version.is_some()
                || edge.implementation.is_some()
                || edge.fixture_path.is_some()
                || edge.reason_code.is_none()
            {
                return Err(MigrationError::InvalidValue);
            }
        }
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn safe_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn hash(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}
