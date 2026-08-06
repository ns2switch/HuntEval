use std::{fs, io, path::PathBuf};

use hunteval_domain::{ContractVersion, RunId, Sha256Digest, UtcTimestamp};
use serde_json::Value;

fn common_schema() -> Result<Value, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))?;
    let schema = fs::read_to_string(workspace_root.join("schemas/v0.3/common.schema.json"))?;
    Ok(serde_json::from_str(&schema)?)
}

fn examples<'a>(schema: &'a Value, definition: &str) -> Result<&'a [Value], io::Error> {
    schema
        .get("$defs")
        .and_then(|definitions| definitions.get(definition))
        .and_then(|value| value.get("examples"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| io::Error::other(format!("missing examples for {definition}")))
}

fn example_text(value: &Value) -> Result<&str, io::Error> {
    value
        .as_str()
        .ok_or_else(|| io::Error::other("schema example is not a string"))
}

#[test]
fn common_schema_examples_match_domain_validation() -> Result<(), Box<dyn std::error::Error>> {
    let schema = common_schema()?;

    for value in examples(&schema, "contract_version")? {
        example_text(value)?.parse::<ContractVersion>()?;
    }
    for value in examples(&schema, "identifier")? {
        example_text(value)?.parse::<RunId>()?;
    }
    for value in examples(&schema, "sha256")? {
        example_text(value)?.parse::<Sha256Digest>()?;
    }
    for value in examples(&schema, "utc_timestamp")? {
        example_text(value)?.parse::<UtcTimestamp>()?;
    }

    Ok(())
}
