use std::{collections::HashMap, fs, io, path::Path, path::PathBuf};

use hunteval_domain::Sha256Digest;
use jsonschema::{Retrieve, Uri};
use serde_json::Value;

#[derive(Clone)]
struct RepositorySchemas {
    documents: HashMap<String, Value>,
}

impl Retrieve for RepositorySchemas {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.documents.get(uri.as_str()).cloned().ok_or_else(|| {
            Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!("schema reference is not in the repository: {uri}"),
            )) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))
}

fn load_schemas(version: &str) -> Result<RepositorySchemas, Box<dyn std::error::Error>> {
    let directory = workspace_root()?.join("schemas").join(version);
    let mut documents = HashMap::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document: Value = serde_json::from_slice(&fs::read(&path)?)?;
        jsonschema::meta::validate(&document).map_err(|error| {
            io::Error::other(format!("invalid schema {}: {error}", path.display()))
        })?;
        let id = document
            .get("$id")
            .and_then(Value::as_str)
            .ok_or_else(|| io::Error::other(format!("schema {} has no $id", path.display())))?;
        documents.insert(id.to_owned(), document);
    }
    Ok(RepositorySchemas { documents })
}

fn json_example(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(
        workspace_root()?.join(path),
    )?)?)
}

fn yaml_example(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_yaml_ng::from_slice(&fs::read(
        workspace_root()?.join(path),
    )?)?)
}

fn validate(
    schemas: &RepositorySchemas,
    version: &str,
    schema_name: &str,
    instance: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = format!("https://hunteval.dev/schemas/{version}/{schema_name}");
    let schema = schemas
        .documents
        .get(&id)
        .ok_or_else(|| io::Error::other(format!("missing schema {id}")))?;
    let validator = jsonschema::options()
        .with_retriever(schemas.clone())
        .should_validate_formats(true)
        .build(schema)
        .map_err(|error| io::Error::other(error.to_string()))?;
    validator
        .validate(instance)
        .map_err(|error| io::Error::other(error.to_string()))?;
    Ok(())
}

#[test]
fn canonical_v04_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas("v0.4")?;
    let cases = [
        ("benchmark-cell.schema.json", "benchmark-cell.json"),
        ("benchmark-event.schema.json", "benchmark-event.json"),
        ("benchmark-state.schema.json", "benchmark-state.json"),
        (
            "comparison-eligibility.schema.json",
            "comparison-eligibility.json",
        ),
        (
            "deployment-registration.schema.json",
            "deployment-registration.json",
        ),
        ("submission.schema.json", "submission.json"),
        ("ground-truth.schema.json", "ground-truth.json"),
        ("report-claim.schema.json", "report-claim.json"),
    ];
    for (schema, example) in cases {
        let instance = json_example(&format!("examples/contracts/v0.4/{example}"))?;
        validate(&schemas, "v0.4", schema, &instance)?;
    }
    validate(
        &schemas,
        "v0.4",
        "benchmark-manifest.schema.json",
        &yaml_example("examples/contracts/v0.4/benchmark-manifest.yaml")?,
    )?;
    Ok(())
}

#[test]
fn canonical_cell_example_has_its_derived_identifier() -> Result<(), Box<dyn std::error::Error>> {
    let cell = json_example("examples/contracts/v0.4/benchmark-cell.json")?;
    let canonical_key = serde_json::to_vec(&cell["key"])?;
    let expected = format!("cell:{}", Sha256Digest::from_bytes(canonical_key));
    assert_eq!(cell["cell_id"].as_str(), Some(expected.as_str()));
    Ok(())
}

#[test]
fn v03_examples_remain_valid_and_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas("v0.3")?;
    let cases = [
        (
            "deployment-registration.schema.json",
            "examples/contracts/deployment-registration.json",
        ),
        (
            "ground-truth.schema.json",
            "examples/contracts/ground-truth.json",
        ),
        ("result.schema.json", "examples/contracts/result.json"),
    ];
    for (schema, path) in cases {
        validate(&schemas, "v0.3", schema, &json_example(path)?)?;
    }
    validate(
        &schemas,
        "v0.3",
        "episode-manifest.schema.json",
        &yaml_example("examples/contracts/episode-manifest.yaml")?,
    )?;
    Ok(())
}

#[test]
fn public_v04_contracts_reject_private_and_unknown_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let schemas = load_schemas("v0.4")?;
    let mut manifest = yaml_example("examples/contracts/v0.4/benchmark-manifest.yaml")?;
    manifest["ground_truth"] = Value::String("private/ground-truth.json".into());
    assert!(
        validate(
            &schemas,
            "v0.4",
            "benchmark-manifest.schema.json",
            &manifest
        )
        .is_err()
    );

    let mut deployment = json_example("examples/contracts/v0.4/deployment-registration.json")?;
    deployment["process"]["environment_values"] = serde_json::json!({"SECRET": "value"});
    assert!(
        validate(
            &schemas,
            "v0.4",
            "deployment-registration.schema.json",
            &deployment
        )
        .is_err()
    );

    let mut submission = json_example("examples/contracts/v0.4/submission.json")?;
    submission["expected_timeline_windows"] = Value::Array(Vec::new());
    assert!(validate(&schemas, "v0.4", "submission.schema.json", &submission).is_err());
    Ok(())
}

#[test]
fn v04_contracts_reject_unsupported_versions() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas("v0.4")?;
    let mut manifest = yaml_example("examples/contracts/v0.4/benchmark-manifest.yaml")?;
    manifest["schema_version"] = Value::String("0.5".into());
    assert!(
        validate(
            &schemas,
            "v0.4",
            "benchmark-manifest.schema.json",
            &manifest
        )
        .is_err()
    );
    Ok(())
}
