use hunteval_domain::{DiagnosticTaxonomy, FailureCategory, Sha256Digest, TaxonomyValidationError};
use std::collections::BTreeMap;
use thiserror::Error;

const TAXONOMY_BYTES: &[u8] = include_bytes!("../../../../taxonomies/diagnostic-failures-v1.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassifierRule {
    pub code: &'static str,
    pub category: FailureCategory,
}

const RULES: &[ClassifierRule] = &[
    ClassifierRule {
        code: "agent_unavailable",
        category: FailureCategory::Resilience,
    },
    ClassifierRule {
        code: "duplicate_task_creation",
        category: FailureCategory::Coordination,
    },
    ClassifierRule {
        code: "evidence_ungrounded",
        category: FailureCategory::Evidence,
    },
    ClassifierRule {
        code: "task_incomplete",
        category: FailureCategory::Investigation,
    },
    ClassifierRule {
        code: "policy_violation_observed",
        category: FailureCategory::Policy,
    },
    ClassifierRule {
        code: "tool_action_failed",
        category: FailureCategory::ToolUse,
    },
];

pub fn canonical_taxonomy() -> Result<DiagnosticTaxonomy, DiagnosticRegistryError> {
    let taxonomy: DiagnosticTaxonomy =
        serde_json::from_slice(TAXONOMY_BYTES).map_err(DiagnosticRegistryError::Parse)?;
    taxonomy
        .validate()
        .map_err(DiagnosticRegistryError::Invalid)?;
    validate_registry(&taxonomy)?;
    Ok(taxonomy)
}

pub fn validate_registry(taxonomy: &DiagnosticTaxonomy) -> Result<(), DiagnosticRegistryError> {
    let expected: BTreeMap<_, _> = RULES
        .iter()
        .map(|rule| (rule.code, rule.category))
        .collect();
    let actual: BTreeMap<_, _> = taxonomy
        .definitions
        .iter()
        .map(|definition| (definition.code.as_str(), definition.category))
        .collect();
    if actual != expected {
        return Err(DiagnosticRegistryError::Drift);
    }
    Ok(())
}

#[must_use]
pub fn classifier_registry_digest() -> Sha256Digest {
    let canonical = RULES
        .iter()
        .map(|rule| format!("{}:{:?}\n", rule.code, rule.category))
        .collect::<String>();
    Sha256Digest::from_bytes(canonical)
}

#[derive(Debug, Error)]
pub enum DiagnosticRegistryError {
    #[error("canonical diagnostic taxonomy is malformed: {0}")]
    Parse(serde_json::Error),
    #[error("canonical diagnostic taxonomy is invalid: {0}")]
    Invalid(TaxonomyValidationError),
    #[error("diagnostic taxonomy and compiled classifier registry differ")]
    Drift,
}
