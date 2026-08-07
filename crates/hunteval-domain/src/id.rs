use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

const MAX_IDENTIFIER_LENGTH: usize = 128;

/// Explains why an opaque HuntEval identifier was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdValidationError {
    /// Identifiers must contain at least one character.
    #[error("identifier must not be empty")]
    Empty,
    /// Identifiers are deliberately bounded before persistence or comparison.
    #[error("identifier exceeds the maximum length of {maximum} bytes")]
    TooLong { maximum: usize },
    /// A character falls outside the stable, path-safe identifier alphabet.
    #[error("identifier contains an invalid character at byte {index}")]
    InvalidCharacter { index: usize },
    /// The first character must be alphanumeric to keep textual forms unambiguous.
    #[error("identifier must start with an ASCII alphanumeric character")]
    InvalidStart,
}

fn validate_identifier(value: &str) -> Result<(), IdValidationError> {
    if value.is_empty() {
        return Err(IdValidationError::Empty);
    }
    if value.len() > MAX_IDENTIFIER_LENGTH {
        return Err(IdValidationError::TooLong {
            maximum: MAX_IDENTIFIER_LENGTH,
        });
    }

    let mut bytes = value.bytes();
    let first = bytes.next().ok_or(IdValidationError::Empty)?;
    if !first.is_ascii_alphanumeric() {
        return Err(IdValidationError::InvalidStart);
    }

    for (offset, byte) in bytes.enumerate() {
        let valid = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':');
        if !valid {
            return Err(IdValidationError::InvalidCharacter { index: offset + 1 });
        }
    }
    Ok(())
}

macro_rules! define_identifier {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier after validating its bounded textual form.
            pub fn new(value: impl Into<String>) -> Result<Self, IdValidationError> {
                let value = value.into();
                validate_identifier(&value)?;
                Ok(Self(value))
            }

            /// Returns the stable textual representation.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdValidationError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

define_identifier!(RunId, "Opaque identifier for one HuntEval run.");
define_identifier!(MessageId, "Opaque identifier for one protocol message.");
define_identifier!(
    DeploymentId,
    "Opaque identifier for an evaluated deployment."
);
define_identifier!(
    AgentId,
    "Opaque identifier for a registered deployment agent."
);
define_identifier!(
    TaskId,
    "Opaque identifier for an observable investigation task."
);
define_identifier!(ActionId, "Opaque identifier for a HuntEval-managed action.");
define_identifier!(EvidenceId, "Opaque identifier for submitted evidence.");
define_identifier!(
    HypothesisId,
    "Opaque identifier for an investigation hypothesis."
);
define_identifier!(FindingId, "Opaque identifier for a threat-hunting finding.");
define_identifier!(EpisodeId, "Opaque identifier for a benchmark episode.");
define_identifier!(EventId, "Opaque identifier for a scored telemetry event.");
define_identifier!(BenchmarkId, "Opaque identifier for a benchmark definition.");
define_identifier!(
    BenchmarkAttemptId,
    "Opaque identifier for one benchmark cell execution attempt."
);
define_identifier!(
    ScoringProfileId,
    "Opaque identifier for a versioned benchmark scoring profile."
);
define_identifier!(
    FaultProfileId,
    "Opaque identifier for a versioned benchmark fault profile."
);

#[cfg(test)]
mod tests {
    use super::{IdValidationError, RunId};

    #[test]
    fn accepts_stable_identifier_alphabet() -> Result<(), IdValidationError> {
        let identifier = RunId::new("run-001:attempt_2.v1")?;
        assert_eq!(identifier.as_str(), "run-001:attempt_2.v1");
        Ok(())
    }

    #[test]
    fn rejects_path_and_control_characters() {
        assert!(matches!(
            RunId::new("../private"),
            Err(IdValidationError::InvalidStart)
        ));
        assert!(matches!(
            RunId::new("run/001"),
            Err(IdValidationError::InvalidCharacter { .. })
        ));
        assert!(matches!(
            RunId::new("run\n001"),
            Err(IdValidationError::InvalidCharacter { .. })
        ));
    }

    #[test]
    fn serde_revalidates_identifiers() -> Result<(), Box<dyn std::error::Error>> {
        let identifier = RunId::new("run-001")?;
        let encoded = serde_json::to_string(&identifier)?;
        let decoded: RunId = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, identifier);
        assert!(serde_json::from_str::<RunId>(r#""../private""#).is_err());
        Ok(())
    }
}
