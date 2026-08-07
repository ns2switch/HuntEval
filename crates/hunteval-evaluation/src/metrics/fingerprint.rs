use hunteval_domain::Sha256Digest;

use crate::EvaluationError;

const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 100_000;
const MAX_CANONICAL_BYTES: usize = 128 * 1024;

pub(super) fn canonical_tool_fingerprint(
    tool: &str,
    arguments: &serde_json::Value,
) -> Result<Sha256Digest, EvaluationError> {
    if !valid_tool_name(tool) {
        return Err(EvaluationError::InvalidToolName);
    }
    let mut canonical = Vec::new();
    let mut nodes = 0;
    write_value(arguments, 0, &mut nodes, &mut canonical)?;
    if canonical.len() > MAX_CANONICAL_BYTES {
        return Err(EvaluationError::InvalidToolArguments);
    }
    let mut input = Vec::with_capacity(tool.len() + canonical.len() + 1);
    input.extend_from_slice(tool.as_bytes());
    input.push(0);
    input.extend_from_slice(&canonical);
    Ok(Sha256Digest::from_bytes(input))
}

fn valid_tool_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && value.len() <= 128
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn write_value(
    value: &serde_json::Value,
    depth: usize,
    nodes: &mut usize,
    output: &mut Vec<u8>,
) -> Result<(), EvaluationError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(EvaluationError::InvalidToolArguments)?;
    if depth > MAX_DEPTH || *nodes > MAX_NODES {
        return Err(EvaluationError::InvalidToolArguments);
    }
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::Number(value) => output.extend_from_slice(value.to_string().as_bytes()),
        serde_json::Value::String(value) => write_primitive(value, output)?,
        serde_json::Value::Array(values) => write_array(values, depth, nodes, output)?,
        serde_json::Value::Object(values) => write_object(values, depth, nodes, output)?,
    }
    if output.len() > MAX_CANONICAL_BYTES {
        return Err(EvaluationError::InvalidToolArguments);
    }
    Ok(())
}

fn write_primitive(value: &str, output: &mut Vec<u8>) -> Result<(), EvaluationError> {
    let encoded = serde_json::to_vec(value).map_err(|_| EvaluationError::InvalidToolArguments)?;
    output.extend_from_slice(&encoded);
    Ok(())
}

fn write_array(
    values: &[serde_json::Value],
    depth: usize,
    nodes: &mut usize,
    output: &mut Vec<u8>,
) -> Result<(), EvaluationError> {
    output.push(b'[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(b',');
        }
        write_value(value, depth + 1, nodes, output)?;
    }
    output.push(b']');
    Ok(())
}

fn write_object(
    values: &serde_json::Map<String, serde_json::Value>,
    depth: usize,
    nodes: &mut usize,
    output: &mut Vec<u8>,
) -> Result<(), EvaluationError> {
    let mut entries: Vec<_> = values.iter().collect();
    entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
    output.push(b'{');
    for (index, (key, value)) in entries.into_iter().enumerate() {
        if index > 0 {
            output.push(b',');
        }
        write_primitive(key, output)?;
        output.push(b':');
        write_value(value, depth + 1, nodes, output)?;
    }
    output.push(b'}');
    Ok(())
}
