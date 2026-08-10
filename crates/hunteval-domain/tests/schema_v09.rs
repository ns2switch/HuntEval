use std::{collections::HashMap, fs, io, path::Path, path::PathBuf};

use jsonschema::{Retrieve, Uri};
use serde_json::{Value, json};

const CONTRACTS: &[(&str, &str)] = &[
    (
        "analytical-corpus-manifest.schema.json",
        "analytical-corpus-manifest.json",
    ),
    (
        "analytical-index-manifest.schema.json",
        "analytical-index-manifest.json",
    ),
    ("analytical-query.schema.json", "analytical-query.json"),
    ("analytical-result.schema.json", "analytical-result.json"),
    (
        "managed-tool-adapter-request.schema.json",
        "managed-tool-adapter-request.json",
    ),
    (
        "managed-tool-adapter-response.schema.json",
        "managed-tool-adapter-response.json",
    ),
    ("extension-manifest.schema.json", "extension-manifest.json"),
    (
        "extension-capability-policy.schema.json",
        "extension-capability-policy.json",
    ),
    (
        "extension-resolution.schema.json",
        "extension-resolution.json",
    ),
    (
        "retrieval-audit-event.schema.json",
        "retrieval-audit-event.json",
    ),
    (
        "extension-conformance-result.schema.json",
        "extension-conformance-result.json",
    ),
    (
        "sdk-compatibility-index.schema.json",
        "sdk-compatibility-index.json",
    ),
];

#[derive(Clone)]
struct Schemas(HashMap<String, Value>);

impl Retrieve for Schemas {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.0.get(uri.as_str()).cloned().ok_or_else(|| {
            Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "schema is unavailable",
            )) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

fn root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))
}

fn schemas() -> Result<Schemas, Box<dyn std::error::Error>> {
    let mut documents = HashMap::new();
    for entry in fs::read_dir(root()?.join("schemas/v0.9"))? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document: Value = serde_json::from_slice(&fs::read(path)?)?;
        jsonschema::meta::validate(&document)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let id = document["$id"]
            .as_str()
            .ok_or_else(|| io::Error::other("schema identifier is missing"))?;
        documents.insert(id.to_owned(), document);
    }
    Ok(Schemas(documents))
}

fn validate(name: &str, example: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let schemas = schemas()?;
    let id = format!("https://hunteval.dev/schemas/v0.9/{name}");
    let schema = schemas
        .0
        .get(&id)
        .cloned()
        .ok_or_else(|| io::Error::other("required schema is missing"))?;
    let validator = jsonschema::options()
        .with_retriever(schemas)
        .should_validate_formats(true)
        .build(&schema)?;
    let value: Value = serde_json::from_slice(&fs::read(
        root()?.join("examples/contracts/v0.9").join(example),
    )?)?;
    Ok(validator.is_valid(&value))
}

#[test]
fn canonical_v09_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    for (schema, example) in CONTRACTS {
        assert!(validate(schema, example)?, "{example} must validate");
    }
    Ok(())
}

#[test]
fn v09_contracts_reject_unknown_private_and_future_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let schemas = schemas()?;
    let schema = schemas
        .0
        .get("https://hunteval.dev/schemas/v0.9/analytical-corpus-manifest.schema.json")
        .cloned()
        .ok_or_else(|| io::Error::other("corpus schema is missing"))?;
    let validator = jsonschema::options()
        .with_retriever(schemas)
        .build(&schema)?;
    let bytes = fs::read(root()?.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))?;
    let mut value: Value = serde_json::from_slice(&bytes)?;
    value["private_ground_truth"] = json!(true);
    assert!(!validator.is_valid(&value));
    value
        .as_object_mut()
        .ok_or_else(|| io::Error::other("example is not an object"))?
        .remove("private_ground_truth");
    value["schema_version"] = json!("1.0");
    assert!(!validator.is_valid(&value));
    Ok(())
}
