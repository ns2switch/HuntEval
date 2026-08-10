use hunteval_domain::{
    ArtifactDiff, DiffOperationKind, ImprovementPolicy, SafetyStatus, StructuredArtifact,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

const FORBIDDEN_DIRECTIVES: &[&str] = &[
    "private chain of thought",
    "reveal hidden test",
    "disable network policy",
    "execute scored tool directly",
    "ignore authorization",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateSafetyResult {
    pub safety_status: SafetyStatus,
    pub leakage_status: SafetyStatus,
    pub reason_codes: Vec<String>,
}

pub fn evaluate_candidate_safety(
    policy: &ImprovementPolicy,
    baseline: &StructuredArtifact,
    candidate: &StructuredArtifact,
    diff: &ArtifactDiff,
    known_answer_fragments: &[String],
) -> Result<CandidateSafetyResult, CandidateSafetyError> {
    policy
        .validate()
        .map_err(|_| CandidateSafetyError::InvalidPolicy)?;
    baseline
        .validate()
        .map_err(|_| CandidateSafetyError::InvalidArtifact)?;
    candidate
        .validate()
        .map_err(|_| CandidateSafetyError::InvalidArtifact)?;
    let mut reasons = Vec::new();
    let baseline_immutable = immutable_classes(baseline);
    let candidate_immutable = immutable_classes(candidate);
    if baseline_immutable != policy.immutable_section_classes
        || candidate_immutable != policy.immutable_section_classes
    {
        reasons.push("immutable_coverage_incomplete".to_owned());
    }
    if diff.operations.iter().any(|operation| {
        !policy.allowed_targets.contains(&operation.section_class)
            || !policy.allowed_operations.contains(&operation.operation)
            || (operation.operation == DiffOperationKind::RemoveSection
                && operation.section_class == hunteval_domain::MutableSectionClass::OutputContract)
    }) {
        reasons.push("operation_not_allowed".to_owned());
    }
    let combined = candidate
        .sections
        .iter()
        .map(|section| section.content.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    if FORBIDDEN_DIRECTIVES
        .iter()
        .any(|directive| combined.contains(directive))
    {
        reasons.push("unsafe_instruction".to_owned());
    }
    let compact_candidate = compact(&combined);
    let leakage = known_answer_fragments.iter().any(|fragment| {
        let normalized = fragment.trim().to_ascii_lowercase();
        if normalized.len() < 8 {
            return false;
        }
        let compact_fragment = compact(&normalized);
        let hex_fragment = hex_text(normalized.as_bytes());
        combined.contains(&normalized)
            || compact_candidate.contains(&compact_fragment)
            || compact_candidate.contains(&hex_fragment)
    });
    if leakage {
        reasons.push("answer_leakage_detected".to_owned());
    }
    let size = candidate
        .sections
        .iter()
        .map(|section| section.content.len() as u64)
        .sum::<u64>();
    let baseline_size = baseline
        .sections
        .iter()
        .map(|section| section.content.len() as u64)
        .sum::<u64>();
    let allowed_growth = baseline_size.saturating_mul(u64::from(policy.max_growth_percent)) / 100;
    if size > policy.max_artifact_bytes || size > baseline_size.saturating_add(allowed_growth) {
        reasons.push("artifact_growth_exceeded".to_owned());
    }
    reasons.sort();
    reasons.dedup();
    Ok(CandidateSafetyResult {
        safety_status: if reasons
            .iter()
            .any(|reason| reason != "answer_leakage_detected")
        {
            SafetyStatus::Rejected
        } else {
            SafetyStatus::Passed
        },
        leakage_status: if leakage {
            SafetyStatus::Rejected
        } else {
            SafetyStatus::Passed
        },
        reason_codes: reasons,
    })
}

fn immutable_classes(
    artifact: &StructuredArtifact,
) -> BTreeSet<hunteval_domain::ImmutableSectionClass> {
    artifact
        .sections
        .iter()
        .filter_map(|section| match section.policy {
            hunteval_domain::SectionPolicy::Immutable(class) => Some(class),
            hunteval_domain::SectionPolicy::Mutable(_) => None,
        })
        .collect()
}

fn compact(value: &str) -> String {
    value
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(char::from)
        .collect()
}

fn hex_text(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum CandidateSafetyError {
    #[error("improvement policy is invalid")]
    InvalidPolicy,
    #[error("structured candidate is invalid")]
    InvalidArtifact,
}
