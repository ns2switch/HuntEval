use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

const DEFAULT_MARKER: &str = "[REDACTED]";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactionPolicy {
    pub maximum_input_bytes: usize,
    pub maximum_output_bytes: usize,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            maximum_input_bytes: 64 * 1024,
            maximum_output_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactedText {
    pub text: String,
    pub truncated: bool,
    pub match_fingerprints: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Redactor {
    policy: RedactionPolicy,
    values: Vec<Vec<u8>>,
}

impl Redactor {
    pub fn new(
        policy: RedactionPolicy,
        values: impl IntoIterator<Item = String>,
    ) -> Result<Self, RedactionError> {
        if policy.maximum_input_bytes == 0 || policy.maximum_output_bytes < DEFAULT_MARKER.len() {
            return Err(RedactionError::InvalidPolicy);
        }
        let mut values = values
            .into_iter()
            .map(String::into_bytes)
            .collect::<Vec<_>>();
        if values.iter().any(Vec::is_empty) {
            return Err(RedactionError::EmptyValue);
        }
        values.sort_by_key(|value| std::cmp::Reverse(value.len()));
        values.dedup();
        Ok(Self { policy, values })
    }

    #[must_use]
    pub fn redact_bytes(&self, input: &[u8]) -> RedactedText {
        let input_truncated = input.len() > self.policy.maximum_input_bytes;
        let bounded = &input[..input.len().min(self.policy.maximum_input_bytes)];
        let mut text = String::from_utf8_lossy(bounded).into_owned();
        let mut fingerprints = Vec::new();
        for value in &self.values {
            let secret = String::from_utf8_lossy(value);
            if text.contains(secret.as_ref()) {
                text = text.replace(secret.as_ref(), DEFAULT_MARKER);
                fingerprints.push(fingerprint(value));
            }
        }
        fingerprints.sort();
        fingerprints.dedup();
        let output_truncated = text.len() > self.policy.maximum_output_bytes;
        if output_truncated {
            let mut boundary = self.policy.maximum_output_bytes - DEFAULT_MARKER.len();
            while !text.is_char_boundary(boundary) {
                boundary -= 1;
            }
            text.truncate(boundary);
            text.push_str(DEFAULT_MARKER);
        }
        RedactedText {
            text,
            truncated: input_truncated || output_truncated,
            match_fingerprints: fingerprints,
        }
    }
}

fn fingerprint(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}

#[derive(Debug, Error)]
pub enum RedactionError {
    #[error("redaction policy limits are invalid")]
    InvalidPolicy,
    #[error("redaction values must not be empty")]
    EmptyValue,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_overlapping_values_without_emitting_them() -> Result<(), RedactionError> {
        let redactor = Redactor::new(
            RedactionPolicy::default(),
            ["token-123".to_owned(), "token".to_owned()],
        )?;
        let result = redactor.redact_bytes(b"failed token-123 and token");
        assert_eq!(result.text, "failed [REDACTED] and [REDACTED]");
        assert_eq!(result.match_fingerprints.len(), 2);
        Ok(())
    }

    #[test]
    fn truncates_on_a_utf8_boundary() -> Result<(), RedactionError> {
        let policy = RedactionPolicy {
            maximum_input_bytes: 64,
            maximum_output_bytes: 16,
        };
        let result =
            Redactor::new(policy, std::iter::empty())?.redact_bytes("éééééééééé".as_bytes());
        assert!(result.truncated && result.text.is_char_boundary(result.text.len()));
        Ok(())
    }
}
