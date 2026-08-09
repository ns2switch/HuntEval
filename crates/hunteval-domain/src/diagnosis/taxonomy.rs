use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{DiagnosticSourceKind, SourceFamily};
use crate::{SchemaVersion, Sha256Digest};

const MAX_DEFINITIONS: usize = 256;
const MAX_DESCRIPTION_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    Investigation,
    Evidence,
    ToolUse,
    Coordination,
    Resilience,
    Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceConfidence {
    Direct,
    Corroborated,
    Controlled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureDefinition {
    pub code: String,
    pub category: FailureCategory,
    pub safe_description: String,
    pub required_source_kinds: BTreeSet<DiagnosticSourceKind>,
    pub required_source_families: BTreeSet<SourceFamily>,
    pub minimum_sources: usize,
    pub minimum_confidence: EvidenceConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticTaxonomy {
    pub schema_version: SchemaVersion,
    pub taxonomy_version: SchemaVersion,
    pub id: String,
    pub definitions: Vec<FailureDefinition>,
}

impl DiagnosticTaxonomy {
    pub fn validate(&self) -> Result<(), TaxonomyValidationError> {
        if self.schema_version != SchemaVersion::new(0, 7) {
            return Err(TaxonomyValidationError::UnsupportedSchema);
        }
        if self.definitions.is_empty() || self.definitions.len() > MAX_DEFINITIONS {
            return Err(TaxonomyValidationError::DefinitionCount);
        }
        let mut codes = BTreeSet::new();
        let mut categories = BTreeSet::new();
        for definition in &self.definitions {
            validate_definition(definition)?;
            if !codes.insert(definition.code.as_str()) {
                return Err(TaxonomyValidationError::DuplicateCode);
            }
            categories.insert(definition.category);
        }
        if categories.len() != 6 {
            return Err(TaxonomyValidationError::MissingCategory);
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Sha256Digest, TaxonomyValidationError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| TaxonomyValidationError::Serialize)?;
        Ok(Sha256Digest::from_bytes(bytes))
    }

    #[must_use]
    pub fn definition(&self, code: &str) -> Option<&FailureDefinition> {
        self.definitions.iter().find(|item| item.code == code)
    }
}

fn validate_definition(definition: &FailureDefinition) -> Result<(), TaxonomyValidationError> {
    if !valid_reason_code(&definition.code) {
        return Err(TaxonomyValidationError::InvalidCode);
    }
    if definition.safe_description.trim().is_empty()
        || definition.safe_description.len() > MAX_DESCRIPTION_BYTES
        || definition.safe_description.chars().any(char::is_control)
    {
        return Err(TaxonomyValidationError::InvalidDescription);
    }
    if definition.required_source_kinds.is_empty()
        || definition.required_source_families.is_empty()
        || definition.minimum_sources == 0
        || definition.minimum_sources > 32
    {
        return Err(TaxonomyValidationError::InvalidRequirement);
    }
    Ok(())
}

fn valid_reason_code(value: &str) -> bool {
    value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || (index > 0 && (byte.is_ascii_digit() || byte == b'_'))
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TaxonomyValidationError {
    #[error("diagnostic taxonomy schema version is unsupported")]
    UnsupportedSchema,
    #[error("diagnostic taxonomy definition count is outside its bound")]
    DefinitionCount,
    #[error("diagnostic taxonomy contains a duplicate code")]
    DuplicateCode,
    #[error("diagnostic taxonomy must cover every failure category")]
    MissingCategory,
    #[error("diagnostic taxonomy code is invalid")]
    InvalidCode,
    #[error("diagnostic taxonomy description is invalid")]
    InvalidDescription,
    #[error("diagnostic taxonomy source requirement is invalid")]
    InvalidRequirement,
    #[error("diagnostic taxonomy serialization failed")]
    Serialize,
}
