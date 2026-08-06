use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

/// An RFC 3339 timestamp whose offset is explicitly UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcTimestamp(OffsetDateTime);

impl UtcTimestamp {
    /// Wraps a timestamp only when its offset is UTC.
    pub fn new(value: OffsetDateTime) -> Result<Self, TimestampError> {
        if value.offset() != UtcOffset::UTC {
            return Err(TimestampError::NonUtcOffset);
        }
        Ok(Self(value))
    }

    /// Returns the underlying time value.
    #[must_use]
    pub const fn as_offset_date_time(self) -> OffsetDateTime {
        self.0
    }
}

impl FromStr for UtcTimestamp {
    type Err = TimestampError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let timestamp =
            OffsetDateTime::parse(value, &Rfc3339).map_err(|_| TimestampError::InvalidRfc3339)?;
        Self::new(timestamp)
    }
}

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encoded = self.0.format(&Rfc3339).map_err(|_| fmt::Error)?;
        formatter.write_str(&encoded)
    }
}

impl Serialize for UtcTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for UtcTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

/// Error returned for invalid or non-UTC timestamp input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TimestampError {
    /// The input is not a valid RFC 3339 timestamp.
    #[error("timestamp must be valid RFC 3339")]
    InvalidRfc3339,
    /// HuntEval contracts require an explicit zero UTC offset.
    #[error("timestamp offset must be UTC")]
    NonUtcOffset,
}

#[cfg(test)]
mod tests {
    use super::{TimestampError, UtcTimestamp};

    #[test]
    fn parses_and_serializes_utc_timestamp() -> Result<(), Box<dyn std::error::Error>> {
        let timestamp: UtcTimestamp = "2026-08-06T18:00:00Z".parse()?;
        let encoded = serde_json::to_string(&timestamp)?;
        assert_eq!(encoded, r#""2026-08-06T18:00:00Z""#);
        Ok(())
    }

    #[test]
    fn rejects_non_utc_offset() {
        assert_eq!(
            "2026-08-06T20:00:00+02:00".parse::<UtcTimestamp>(),
            Err(TimestampError::NonUtcOffset)
        );
    }
}
