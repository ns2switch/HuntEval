use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

/// A validated `major.minor` contract version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractVersion {
    major: u16,
    minor: u16,
}

impl ContractVersion {
    /// Creates a version from numeric components.
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Returns the compatibility-breaking component.
    #[must_use]
    pub const fn major(self) -> u16 {
        self.major
    }

    /// Returns the backward-compatible feature component.
    #[must_use]
    pub const fn minor(self) -> u16 {
        self.minor
    }

    /// Returns whether this implementation can consume `candidate`.
    #[must_use]
    pub const fn supports(self, candidate: Self) -> bool {
        self.major == candidate.major && candidate.minor <= self.minor
    }
}

impl fmt::Display for ContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for ContractVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value
            .split_once('.')
            .ok_or(VersionParseError::InvalidFormat)?;
        if minor.contains('.') || major.is_empty() || minor.is_empty() {
            return Err(VersionParseError::InvalidFormat);
        }
        let major = parse_component(major)?;
        let minor = parse_component(minor)?;
        Ok(Self::new(major, minor))
    }
}

fn parse_component(value: &str) -> Result<u16, VersionParseError> {
    let has_leading_zero = value.len() > 1 && value.starts_with('0');
    if has_leading_zero || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(VersionParseError::InvalidComponent);
    }
    value
        .parse::<u16>()
        .map_err(|_| VersionParseError::InvalidComponent)
}

impl Serialize for ContractVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ContractVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

/// Error returned for malformed contract versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum VersionParseError {
    /// The value is not exactly `major.minor`.
    #[error("contract version must use major.minor format")]
    InvalidFormat,
    /// A component is not an unsigned 16-bit integer.
    #[error("contract version contains an invalid numeric component")]
    InvalidComponent,
}

/// Version used by JSONL process messages.
pub type ProtocolVersion = ContractVersion;

/// Version used by persisted schemas and normalized artifacts.
pub type SchemaVersion = ContractVersion;

#[cfg(test)]
mod tests {
    use super::ContractVersion;

    #[test]
    fn parses_and_compares_compatible_versions() -> Result<(), Box<dyn std::error::Error>> {
        let supported: ContractVersion = "0.3".parse()?;
        let older: ContractVersion = "0.2".parse()?;
        let other_major: ContractVersion = "1.0".parse()?;

        assert!(supported.supports(older));
        assert!(!older.supports(supported));
        assert!(!supported.supports(other_major));
        assert_eq!(supported.to_string(), "0.3");
        Ok(())
    }

    #[test]
    fn rejects_non_contract_versions() {
        for value in ["", "0", "0.3.1", "v0.3", "0.-1", "00.3", "+1.0"] {
            assert!(
                value.parse::<ContractVersion>().is_err(),
                "accepted {value}"
            );
        }
    }
}
