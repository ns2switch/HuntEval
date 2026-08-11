use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{CommercialError, CommercialOperation, CommercialPlatform};

const MAX_ORIGIN_BYTES: usize = 512;

/// Opaque secret identity. Secret values are never part of this contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SecretReference(String);

impl TryFrom<String> for SecretReference {
    type Error = CommercialError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let valid = !value.is_empty()
            && value.len() <= 128
            && value.bytes().enumerate().all(|(index, byte)| {
                byte.is_ascii_alphanumeric() || (index > 0 && b"._:-".contains(&byte))
            });
        if !valid {
            return Err(CommercialError::InvalidSecretReference);
        }
        Ok(Self(value))
    }
}

impl From<SecretReference> for String {
    fn from(value: SecretReference) -> Self {
        value.0
    }
}

impl SecretReference {
    /// Resolver-facing opaque name; it is an identity, never secret material.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// One-way identity safe for public audit records.
    #[must_use]
    pub fn identity_sha256(&self) -> String {
        hex_digest(self.0.as_bytes())
    }
}

/// Address resolved immediately before connection establishment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedAddress(pub IpAddr);

impl ResolvedAddress {
    /// Reject local, private, documentation, multicast, and metadata destinations.
    pub fn validate(self) -> Result<(), CommercialError> {
        let denied = match self.0 {
            IpAddr::V4(value) => denied_v4(value),
            IpAddr::V6(value) => denied_v6(value),
        };
        if denied {
            Err(CommercialError::DeniedAddress)
        } else {
            Ok(())
        }
    }
}

/// Available connector modes before production scored execution exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialMode {
    FixtureReplay,
    LiveReadOnly,
}

/// Runner-owned authorization for one exact connector and operation inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommercialPolicy {
    pub policy_version: String,
    pub mode: CommercialMode,
    pub platform: CommercialPlatform,
    pub origin: String,
    pub operations: Vec<CommercialOperation>,
    pub secret_reference: Option<SecretReference>,
    pub max_requests: u32,
    pub max_response_bytes: u64,
    pub max_records: u32,
    pub timeout_ms: u64,
}

impl CommercialPolicy {
    /// Validate bounded HTTPS-only policy without resolving or using a secret.
    pub fn validate(&self) -> Result<(), CommercialError> {
        if self.policy_version != "0.1"
            || self.operations.is_empty()
            || self.operations.len() > 64
            || self.max_requests == 0
            || self.max_requests > 1_024
            || self.max_response_bytes == 0
            || self.max_response_bytes > 64 * 1024 * 1024
            || self.max_records == 0
            || self.max_records > 100_000
            || self.timeout_ms == 0
            || self.timeout_ms > 300_000
        {
            return Err(CommercialError::InvalidPolicy);
        }
        if !matches!(
            (self.mode, &self.secret_reference),
            (CommercialMode::FixtureReplay, None) | (CommercialMode::LiveReadOnly, Some(_))
        ) {
            return Err(CommercialError::InvalidPolicy);
        }
        validate_origin(&self.origin)?;
        let mut operations = self.operations.clone();
        operations.sort_unstable();
        operations.dedup();
        if operations.len() != self.operations.len()
            || operations
                .iter()
                .any(|operation| !self.platform.supports(*operation))
        {
            return Err(CommercialError::InvalidPolicy);
        }
        Ok(())
    }

    /// Content identity excluding all secret values by construction.
    pub fn sha256(&self) -> Result<String, CommercialError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| CommercialError::InvalidPolicy)?;
        Ok(hex_digest(&bytes))
    }
}

fn validate_origin(value: &str) -> Result<(), CommercialError> {
    let host = value
        .strip_prefix("https://")
        .ok_or(CommercialError::InvalidOrigin)?;
    let valid = !host.is_empty()
        && value.len() <= MAX_ORIGIN_BYTES
        && !host.contains(['/', '?', '#', '@'])
        && !host.starts_with('.')
        && !host.ends_with('.')
        && host.contains('.')
        && host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
        && host.parse::<IpAddr>().is_err();
    if !valid {
        return Err(CommercialError::InvalidOrigin);
    }
    Ok(())
}

fn denied_v4(value: Ipv4Addr) -> bool {
    let octets = value.octets();
    value.is_private()
        || value.is_loopback()
        || value.is_link_local()
        || value.is_multicast()
        || value.is_unspecified()
        || octets[0] == 0
        || octets[0] >= 224
        || octets == [169, 254, 169, 254]
        || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
}

fn denied_v6(value: Ipv6Addr) -> bool {
    let segments = value.segments();
    value.is_loopback()
        || value.is_unspecified()
        || value.is_multicast()
        || (segments[0] & 0xfe00) == 0xfc00
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
        || value.to_ipv4_mapped().is_some_and(denied_v4)
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
