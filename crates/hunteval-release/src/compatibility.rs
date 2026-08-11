use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{InterfaceFreezeManifest, StabilityClass};

const MAX_RULES: usize = 512;
const MAX_COMPONENTS: usize = 32;
const MAX_LIMITATIONS: usize = 32;
const MAX_TEXT_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityStatus {
    Supported,
    Retained,
    Preview,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityComponent {
    pub interface_id: String,
    pub version: String,
    pub fixture_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityRule {
    pub combination_id: String,
    pub components: Vec<CompatibilityComponent>,
    pub status: CompatibilityStatus,
    pub rejection_reason: Option<String>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityMatrix {
    pub schema_version: String,
    pub matrix_id: String,
    pub inventory_sha256: String,
    pub baseline_revision: String,
    pub rules: Vec<CompatibilityRule>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("compatibility matrix has an unsupported schema version")]
    UnsupportedVersion,
    #[error("compatibility matrix contains an invalid bounded value")]
    InvalidValue,
    #[error("compatibility matrix contains a duplicate or ambiguous rule")]
    AmbiguousRule,
    #[error("compatibility matrix references an interface outside the eligible freeze set")]
    IneligibleInterface,
    #[error("compatibility combination is not declared")]
    UnknownCombination,
    #[error("compatibility matrix serialization failed")]
    Serialization,
}

impl CompatibilityMatrix {
    pub fn validate_against(
        &self,
        freeze: &InterfaceFreezeManifest,
    ) -> Result<(), CompatibilityError> {
        if self.schema_version != "1.0" || freeze.schema_version != "1.0" {
            return Err(CompatibilityError::UnsupportedVersion);
        }
        if !identifier(&self.matrix_id)
            || !digest(&self.inventory_sha256)
            || self.inventory_sha256 != freeze.inventory_sha256
            || !revision(&self.baseline_revision)
            || self.rules.is_empty()
            || self.rules.len() > MAX_RULES
        {
            return Err(CompatibilityError::InvalidValue);
        }
        let eligible = freeze
            .eligible_interfaces
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let mut identifiers = BTreeSet::new();
        let mut signatures = BTreeSet::new();
        for rule in &self.rules {
            validate_rule(rule)?;
            if !identifiers.insert(rule.combination_id.as_str()) {
                return Err(CompatibilityError::AmbiguousRule);
            }
            let signature = rule
                .components
                .iter()
                .map(|component| (&component.interface_id, &component.version))
                .collect::<BTreeSet<_>>();
            if signature.len() != rule.components.len() || !signatures.insert(signature) {
                return Err(CompatibilityError::AmbiguousRule);
            }
            if matches!(
                rule.status,
                CompatibilityStatus::Supported | CompatibilityStatus::Retained
            ) && rule
                .components
                .iter()
                .any(|component| !eligible.contains(component.interface_id.as_str()))
            {
                return Err(CompatibilityError::IneligibleInterface);
            }
        }
        Ok(())
    }

    pub fn normalized_json(
        &self,
        freeze: &InterfaceFreezeManifest,
    ) -> Result<Vec<u8>, CompatibilityError> {
        self.validate_against(freeze)?;
        let mut normalized = self.clone();
        for rule in &mut normalized.rules {
            rule.components.sort();
            rule.limitations.sort();
        }
        normalized
            .rules
            .sort_by(|left, right| left.combination_id.cmp(&right.combination_id));
        let mut bytes = serde_json::to_vec_pretty(&normalized)
            .map_err(|_| CompatibilityError::Serialization)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    pub fn markdown(&self, freeze: &InterfaceFreezeManifest) -> Result<String, CompatibilityError> {
        let normalized: Self = serde_json::from_slice(&self.normalized_json(freeze)?)
            .map_err(|_| CompatibilityError::Serialization)?;
        let mut output = String::from(
            "# HuntEval compatibility matrix\n\nCompatibility does not grant runtime authority or certify deployment quality.\n\n| Combination | Status | Components | Reason or limitation |\n|---|---|---|---|\n",
        );
        for rule in normalized.rules {
            let components = rule
                .components
                .iter()
                .map(|component| format!("{}@{}", component.interface_id, component.version))
                .collect::<Vec<_>>()
                .join(", ");
            let reason = rule
                .rejection_reason
                .or_else(|| rule.limitations.first().cloned())
                .unwrap_or_else(|| "none".to_owned());
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                rule.combination_id,
                status_name(rule.status),
                components,
                reason
            ));
        }
        Ok(output)
    }

    pub fn rule_for(
        &self,
        components: &[CompatibilityComponent],
    ) -> Result<&CompatibilityRule, CompatibilityError> {
        let requested = components
            .iter()
            .map(|component| (&component.interface_id, &component.version))
            .collect::<BTreeSet<_>>();
        self.rules
            .iter()
            .find(|rule| {
                rule.components
                    .iter()
                    .map(|component| (&component.interface_id, &component.version))
                    .collect::<BTreeSet<_>>()
                    == requested
            })
            .ok_or(CompatibilityError::UnknownCombination)
    }
}

fn validate_rule(rule: &CompatibilityRule) -> Result<(), CompatibilityError> {
    if !identifier(&rule.combination_id)
        || rule.components.is_empty()
        || rule.components.len() > MAX_COMPONENTS
        || rule.limitations.len() > MAX_LIMITATIONS
        || rule.limitations.iter().any(|value| !bounded(value))
        || rule.components.iter().any(|component| {
            !identifier(&component.interface_id)
                || !bounded(&component.version)
                || !digest(&component.fixture_sha256)
        })
    {
        return Err(CompatibilityError::InvalidValue);
    }
    let compatible = matches!(
        rule.status,
        CompatibilityStatus::Supported | CompatibilityStatus::Retained
    );
    if compatible != rule.rejection_reason.is_none()
        || (!compatible && rule.limitations.is_empty())
        || rule
            .rejection_reason
            .as_deref()
            .is_some_and(|value| !identifier(value))
    {
        return Err(CompatibilityError::InvalidValue);
    }
    Ok(())
}

fn status_name(status: CompatibilityStatus) -> &'static str {
    match status {
        CompatibilityStatus::Supported => "supported",
        CompatibilityStatus::Retained => "retained",
        CompatibilityStatus::Preview => "preview",
        CompatibilityStatus::Unavailable => "unavailable",
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
}

fn bounded(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TEXT_BYTES && !value.chars().any(char::is_control)
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn stability_status(stability: StabilityClass) -> CompatibilityStatus {
    match stability {
        StabilityClass::StableCandidate => CompatibilityStatus::Supported,
        StabilityClass::Retained => CompatibilityStatus::Retained,
        StabilityClass::Preview | StabilityClass::Experimental => CompatibilityStatus::Preview,
        StabilityClass::Blocked => CompatibilityStatus::Unavailable,
    }
}
