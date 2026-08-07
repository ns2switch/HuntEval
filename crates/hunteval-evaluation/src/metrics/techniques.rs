use std::collections::BTreeSet;

use hunteval_domain::MetricValue;

use crate::{EvaluationError, sets};

pub(super) fn evaluate(
    submitted: &BTreeSet<String>,
    expected: &BTreeSet<String>,
    benign: bool,
) -> Result<(MetricValue, MetricValue), EvaluationError> {
    for identifier in submitted.iter().chain(expected) {
        if !is_supported_identifier(identifier) {
            return Err(EvaluationError::UnsupportedTechniqueIdentifier(
                identifier.clone(),
            ));
        }
    }
    Ok((
        sets::precision(submitted, expected, benign),
        sets::recall(submitted, expected, benign),
    ))
}

fn is_supported_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    (bytes.len() == 5 || bytes.len() == 9)
        && bytes.first() == Some(&b'T')
        && bytes[1..5].iter().all(u8::is_ascii_digit)
        && (bytes.len() == 5
            || (bytes.get(5) == Some(&b'.') && bytes[6..].iter().all(u8::is_ascii_digit)))
}
