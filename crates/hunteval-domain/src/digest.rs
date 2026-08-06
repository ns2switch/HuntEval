use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use sha2::{Digest, Sha256};
use thiserror::Error;

const SHA256_BYTES: usize = 32;
const SHA256_HEX_LENGTH: usize = SHA256_BYTES * 2;

/// A fixed-size SHA-256 digest used for reproducible artifact identity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest([u8; SHA256_BYTES]);

impl Sha256Digest {
    /// Hashes the exact supplied bytes.
    #[must_use]
    pub fn from_bytes(value: impl AsRef<[u8]>) -> Self {
        let bytes: [u8; SHA256_BYTES] = Sha256::digest(value.as_ref()).into();
        Self(bytes)
    }

    /// Returns the raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; SHA256_BYTES] {
        &self.0
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Sha256Digest")
            .field(&self.to_string())
            .finish()
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0))
    }
}

impl FromStr for Sha256Digest {
    type Err = DigestParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != SHA256_HEX_LENGTH {
            return Err(DigestParseError::InvalidLength);
        }
        let mut bytes = [0_u8; SHA256_BYTES];
        hex::decode_to_slice(value, &mut bytes).map_err(|_| DigestParseError::InvalidHex)?;
        Ok(Self(bytes))
    }
}

impl Serialize for Sha256Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

/// Error returned for malformed SHA-256 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DigestParseError {
    /// SHA-256 text must contain exactly 64 hexadecimal characters.
    #[error("SHA-256 digest must contain exactly 64 hexadecimal characters")]
    InvalidLength,
    /// The text contains a non-hexadecimal character.
    #[error("SHA-256 digest contains invalid hexadecimal data")]
    InvalidHex,
}

#[cfg(test)]
mod tests {
    use super::{DigestParseError, Sha256Digest};

    #[test]
    fn hashes_and_round_trips_exact_bytes() -> Result<(), Box<dyn std::error::Error>> {
        let digest = Sha256Digest::from_bytes(b"HuntEval");
        let text = digest.to_string();
        let parsed: Sha256Digest = text.parse()?;
        assert_eq!(parsed, digest);
        assert_eq!(text.len(), 64);
        Ok(())
    }

    #[test]
    fn rejects_invalid_digest_text() {
        assert_eq!(
            "abc".parse::<Sha256Digest>(),
            Err(DigestParseError::InvalidLength)
        );
        assert!("z".repeat(64).parse::<Sha256Digest>().is_err());
    }
}
