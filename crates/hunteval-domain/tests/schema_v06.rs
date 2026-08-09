use std::{collections::HashMap, fs, io, path::Path, path::PathBuf};

use jsonschema::{Retrieve, Uri};
use serde_json::Value;

const CONTRACTS: &[(&str, &str)] = &[
    (
        "episode-classification.schema.json",
        "episode-classification.json",
    ),
    (
        "dataset-review-record.schema.json",
        "dataset-review-record.json",
    ),
    ("statistical-policy.schema.json", "statistical-policy.json"),
    ("calibration-result.schema.json", "calibration-result.json"),
    (
        "deployment-topology.schema.json",
        "deployment-topology.json",
    ),
    (
        "topology-experiment.schema.json",
        "topology-experiment.json",
    ),
    (
        "topology-equivalence-result.schema.json",
        "topology-equivalence-result.json",
    ),
    ("topology-analysis.schema.json", "topology-analysis.json"),
    (
        "topology-ablation-observations.schema.json",
        "topology-ablation-observations.json",
    ),
    (
        "topology-comparison-report.schema.json",
        "topology-comparison-report.json",
    ),
    (
        "contributor-validation-result.schema.json",
        "contributor-validation-result.json",
    ),
    (
        "review-bundle-manifest.schema.json",
        "review-bundle-manifest.json",
    ),
];

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
                "schema reference is not in the repository",
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

fn load_schemas() -> Result<RepositorySchemas, Box<dyn std::error::Error>> {
    let directory = workspace_root()?.join("schemas/v0.6");
    let mut documents = HashMap::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let document: Value = serde_json::from_slice(&fs::read(&path)?)?;
        jsonschema::meta::validate(&document)
            .map_err(|error| io::Error::other(error.to_string()))?;
        let id = document
            .get("$id")
            .and_then(Value::as_str)
            .ok_or_else(|| io::Error::other("schema has no identifier"))?;
        documents.insert(id.to_owned(), document);
    }
    Ok(RepositorySchemas { documents })
}

#[test]
fn canonical_v06_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;
    let root = workspace_root()?;
    for (schema_name, example_name) in CONTRACTS {
        let id = format!("https://hunteval.dev/schemas/v0.6/{schema_name}");
        let schema = schemas
            .documents
            .get(&id)
            .ok_or_else(|| io::Error::other("required schema is missing"))?;
        let example: Value = serde_json::from_slice(&fs::read(
            root.join("examples/contracts/v0.6").join(example_name),
        )?)?;
        let validator = jsonschema::options()
            .with_retriever(schemas.clone())
            .should_validate_formats(true)
            .build(schema)?;
        validator
            .validate(&example)
            .map_err(|error| io::Error::other(format!("{example_name}: {error}")))?;
    }
    Ok(())
}

#[test]
fn public_v06_contracts_reject_private_and_unknown_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let schemas = load_schemas()?;
    let schema = schemas
        .documents
        .get("https://hunteval.dev/schemas/v0.6/episode-classification.schema.json")
        .ok_or_else(|| io::Error::other("classification schema is missing"))?;
    let validator = jsonschema::options()
        .with_retriever(schemas.clone())
        .build(schema)?;
    let mut example: Value = serde_json::from_slice(&fs::read(
        workspace_root()?.join("examples/contracts/v0.6/episode-classification.json"),
    )?)?;
    example["ground_truth"] = serde_json::json!({"malicious_event_ids": ["evt-answer"]});
    assert!(!validator.is_valid(&example));
    Ok(())
}
