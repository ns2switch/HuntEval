use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::CommercialError;

const MAX_QUERY_BYTES: usize = 32 * 1024;
const MAX_TARGET_VALUE_BYTES: usize = 512;

pub(crate) fn expand_path(
    template: &str,
    values: &BTreeMap<String, String>,
) -> Result<String, CommercialError> {
    let mut output = template.to_owned();
    while let Some(start) = output.find('{') {
        let tail = &output[start + 1..];
        let end = tail.find('}').ok_or(CommercialError::InvalidPolicy)? + start + 1;
        let key = &output[start + 1..end];
        let value = values.get(key).ok_or(CommercialError::InvalidPolicy)?;
        output.replace_range(start..=end, value);
    }
    if output.contains(['{', '}'])
        || !output.starts_with('/')
        || output.contains("://")
        || output.split('/').any(|part| part == "." || part == "..")
    {
        return Err(CommercialError::InvalidPolicy);
    }
    Ok(output)
}

pub(crate) fn allow(
    arguments: &BTreeMap<String, Value>,
    allowed: &[&str],
) -> Result<(), CommercialError> {
    let allowed: BTreeSet<_> = allowed.iter().copied().collect();
    if arguments.keys().all(|key| allowed.contains(key.as_str())) {
        Ok(())
    } else {
        Err(CommercialError::DeniedOperation)
    }
}

pub(crate) fn require_keys(
    arguments: &BTreeMap<String, Value>,
    required: &[&str],
) -> Result<(), CommercialError> {
    if required.iter().all(|key| arguments.contains_key(*key)) {
        Ok(())
    } else {
        Err(CommercialError::InvalidRequest)
    }
}

pub(crate) fn text_query(
    arguments: &BTreeMap<String, Value>,
    names: &[&str],
) -> Result<Vec<(String, String)>, CommercialError> {
    names.iter().try_fold(Vec::new(), |mut output, name| {
        if let Some(value) = optional_text(arguments, name)? {
            output.push(((*name).to_owned(), value));
        }
        Ok(output)
    })
}

pub(crate) fn required_text(
    arguments: &BTreeMap<String, Value>,
    name: &str,
) -> Result<String, CommercialError> {
    optional_text(arguments, name)?.ok_or(CommercialError::InvalidRequest)
}

pub(crate) fn optional_text(
    arguments: &BTreeMap<String, Value>,
    name: &str,
) -> Result<Option<String>, CommercialError> {
    arguments
        .get(name)
        .map(|value| match value {
            Value::String(value)
                if !value.is_empty() && value.len() <= MAX_QUERY_BYTES && !value.contains('\0') =>
            {
                Ok(value.clone())
            }
            _ => Err(CommercialError::InvalidRequest),
        })
        .transpose()
}

pub(crate) fn required_segment(value: &Value) -> Result<&str, CommercialError> {
    match value {
        Value::String(value) if valid_segment(value) => Ok(value),
        _ => Err(CommercialError::InvalidRequest),
    }
}

pub(crate) fn required_string_array(
    arguments: &BTreeMap<String, Value>,
    name: &str,
    maximum: usize,
) -> Result<Vec<String>, CommercialError> {
    match arguments.get(name) {
        Some(Value::Array(values)) if !values.is_empty() && values.len() <= maximum => values
            .iter()
            .map(|value| required_segment(value).map(str::to_owned))
            .collect(),
        _ => Err(CommercialError::InvalidRequest),
    }
}

pub(crate) fn limit(
    arguments: &BTreeMap<String, Value>,
    name: &str,
    maximum: u32,
) -> Result<u64, CommercialError> {
    Ok(optional_u64(arguments, name, u64::from(maximum))?.unwrap_or(u64::from(maximum)))
}

pub(crate) fn push_limit(
    output: &mut Vec<(String, String)>,
    arguments: &BTreeMap<String, Value>,
    name: &str,
    maximum: u32,
) -> Result<(), CommercialError> {
    push_limit_named(output, arguments, name, name, maximum)
}

pub(crate) fn push_limit_named(
    output: &mut Vec<(String, String)>,
    arguments: &BTreeMap<String, Value>,
    name: &str,
    output_name: &str,
    maximum: u32,
) -> Result<(), CommercialError> {
    output.push((
        output_name.to_owned(),
        limit(arguments, name, maximum)?.to_string(),
    ));
    Ok(())
}

pub(crate) fn push_u64(
    output: &mut Vec<(String, String)>,
    arguments: &BTreeMap<String, Value>,
    name: &str,
    maximum: u64,
) -> Result<(), CommercialError> {
    if let Some(value) = optional_u64(arguments, name, maximum)? {
        output.push((name.to_owned(), value.to_string()));
    }
    Ok(())
}

pub(crate) fn optional_u64(
    arguments: &BTreeMap<String, Value>,
    name: &str,
    maximum: u64,
) -> Result<Option<u64>, CommercialError> {
    arguments
        .get(name)
        .map(|value| {
            value
                .as_u64()
                .filter(|value| (1..=maximum).contains(value))
                .ok_or(CommercialError::InvalidRequest)
        })
        .transpose()
}

pub(crate) fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_lowercase())
}

pub(crate) fn valid_target_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_TARGET_VALUE_BYTES
        && !value.contains(['?', '#', '\0'])
        && !value.contains("://")
        && value.split('/').all(valid_segment)
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}
