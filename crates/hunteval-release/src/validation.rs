use std::{collections::BTreeSet, path::Component, path::Path};

use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    FreezeExclusion, InterfaceEntry, InterfaceFreezeManifest, InterfaceInventory,
    PreconditionStatus, Projection, StabilityClass,
};

const MAX_INTERFACES: usize = 512;
const MAX_LIMITATIONS: usize = 32;
const MAX_TEXT_BYTES: usize = 256;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InventoryError {
    #[error("release inventory has an unsupported schema version")]
    UnsupportedVersion,
    #[error("release inventory contains an invalid bounded value")]
    InvalidValue,
    #[error("release inventory contains a duplicate interface")]
    DuplicateInterface,
    #[error("stable interface candidate does not satisfy its freeze prerequisites")]
    IneligibleStableCandidate,
    #[error("release inventory serialization failed")]
    Serialization,
}

impl InterfaceInventory {
    pub fn validate(&self) -> Result<(), InventoryError> {
        if self.schema_version != "1.0" {
            return Err(InventoryError::UnsupportedVersion);
        }
        if !identifier(&self.inventory_id)
            || !revision(&self.baseline_revision)
            || self.interfaces.is_empty()
            || self.interfaces.len() > MAX_INTERFACES
        {
            return Err(InventoryError::InvalidValue);
        }
        let mut identifiers = BTreeSet::new();
        for interface in &self.interfaces {
            validate_interface(interface)?;
            if !identifiers.insert(interface.interface_id.as_str()) {
                return Err(InventoryError::DuplicateInterface);
            }
        }
        if self.pre_r8_status == PreconditionStatus::Satisfied
            && self
                .interfaces
                .iter()
                .any(|interface| interface.precondition_status != PreconditionStatus::Satisfied)
        {
            return Err(InventoryError::InvalidValue);
        }
        Ok(())
    }

    pub fn freeze_manifest(&self) -> Result<InterfaceFreezeManifest, InventoryError> {
        self.validate()?;
        let mut normalized = self.clone();
        normalized
            .interfaces
            .sort_by(|left, right| left.interface_id.cmp(&right.interface_id));
        let bytes = serde_json::to_vec(&normalized).map_err(|_| InventoryError::Serialization)?;
        let inventory_sha256 = hex_digest(&bytes);
        let mut eligible_interfaces = Vec::new();
        let mut exclusions = Vec::new();
        for interface in normalized.interfaces {
            match interface.stability {
                StabilityClass::StableCandidate | StabilityClass::Retained => {
                    eligible_interfaces.push(interface.interface_id);
                }
                StabilityClass::Preview => exclusions.push(exclusion(interface, "preview")),
                StabilityClass::Experimental => {
                    exclusions.push(exclusion(interface, "experimental"));
                }
                StabilityClass::Blocked => exclusions.push(blocked_exclusion(interface)),
            }
        }
        Ok(InterfaceFreezeManifest {
            schema_version: "1.0".to_owned(),
            inventory_sha256,
            eligible_interfaces,
            exclusions,
        })
    }
}

fn validate_interface(interface: &InterfaceEntry) -> Result<(), InventoryError> {
    if !identifier(&interface.interface_id)
        || !identifier(&interface.owner)
        || !bounded(&interface.version_range)
        || !identifier(&interface.authority)
        || !identifier(&interface.trust_boundary)
        || interface.limitations.len() > MAX_LIMITATIONS
        || interface.limitations.iter().any(|value| !bounded(value))
        || interface
            .fixture_path
            .as_deref()
            .is_some_and(|value| !safe_relative_path(value))
        || interface
            .verification_gate
            .as_deref()
            .is_some_and(|value| !identifier(value))
    {
        return Err(InventoryError::InvalidValue);
    }
    let stable = matches!(
        interface.stability,
        StabilityClass::StableCandidate | StabilityClass::Retained
    );
    if stable
        && (interface.precondition_status != PreconditionStatus::Satisfied
            || interface.projection != Projection::Public
            || !interface.bounds_documented
            || !interface.parser_behavior_documented
            || interface.fixture_path.is_none()
            || interface.verification_gate.is_none())
    {
        return Err(InventoryError::IneligibleStableCandidate);
    }
    if matches!(interface.stability, StabilityClass::Blocked)
        && (interface.precondition_status == PreconditionStatus::Satisfied
            || interface.limitations.is_empty())
    {
        return Err(InventoryError::InvalidValue);
    }
    if matches!(
        interface.stability,
        StabilityClass::Preview | StabilityClass::Experimental
    ) && interface.limitations.is_empty()
    {
        return Err(InventoryError::InvalidValue);
    }
    Ok(())
}

fn exclusion(interface: InterfaceEntry, reason_code: &str) -> FreezeExclusion {
    FreezeExclusion {
        interface_id: interface.interface_id,
        reason_code: reason_code.to_owned(),
    }
}

fn blocked_exclusion(interface: InterfaceEntry) -> FreezeExclusion {
    let reason_code = if interface.precondition_status == PreconditionStatus::Unavailable {
        "precondition_unavailable"
    } else {
        "precondition_pending"
    };
    exclusion(interface, reason_code)
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

fn revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn safe_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !value.contains('\\')
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn hex_digest(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}
