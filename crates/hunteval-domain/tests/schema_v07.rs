use std::{collections::HashMap, fs, io, path::Path, path::PathBuf};

use jsonschema::{Retrieve, Uri};
use serde_json::{Value, json};

const CONTRACTS: &[(&str, &str)] = &[
    (
        "diagnostic-source-reference.schema.json",
        "diagnostic-source-reference.json",
    ),
    (
        "diagnostic-taxonomy.schema.json",
        "diagnostic-taxonomy.json",
    ),
    (
        "failure-classification.schema.json",
        "failure-classification.json",
    ),
    ("run-diagnosis.schema.json", "run-diagnosis.json"),
    (
        "diagnostic-recurrence.schema.json",
        "diagnostic-recurrence.json",
    ),
    (
        "bottleneck-observations.schema.json",
        "bottleneck-observations.json",
    ),
    (
        "bottleneck-analysis.schema.json",
        "bottleneck-analysis.json",
    ),
    (
        "contribution-analysis.schema.json",
        "contribution-analysis.json",
    ),
    ("diagnostic-report.schema.json", "diagnostic-report.json"),
    (
        "diagnostic-bundle-manifest.schema.json",
        "diagnostic-bundle-manifest.json",
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
    let directory = workspace_root()?.join("schemas/v0.7");
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

fn example(name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(
        workspace_root()?.join("examples/contracts/v0.7").join(name),
    )?)?)
}

fn validator(
    schemas: &RepositorySchemas,
    schema_name: &str,
) -> Result<jsonschema::Validator, Box<dyn std::error::Error>> {
    let id = format!("https://hunteval.dev/schemas/v0.7/{schema_name}");
    let schema = schemas
        .documents
        .get(&id)
        .ok_or_else(|| io::Error::other(format!("required schema is missing: {schema_name}")))?;
    Ok(jsonschema::options()
        .with_retriever(schemas.clone())
        .should_validate_formats(true)
        .build(schema)?)
}

#[test]
fn canonical_v07_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;
    assert_eq!(schemas.documents.len(), CONTRACTS.len() + 1);
    for (schema_name, example_name) in CONTRACTS {
        validator(&schemas, schema_name)?
            .validate(&example(example_name)?)
            .map_err(|error| io::Error::other(format!("{example_name}: {error}")))?;
    }
    Ok(())
}

#[test]
fn diagnostic_contracts_reject_unknown_private_and_future_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;

    let classification = validator(&schemas, "failure-classification.schema.json")?;
    let mut value = example("failure-classification.json")?;
    value["private_ground_truth"] = json!({"expected_finding": "answer"});
    assert!(!classification.is_valid(&value));

    let mut value = example("failure-classification.json")?;
    value["schema_version"] = json!("0.8");
    assert!(!classification.is_valid(&value));

    let source = validator(&schemas, "diagnostic-source-reference.schema.json")?;
    let mut value = example("diagnostic-source-reference.json")?;
    value["kind"] = json!("private_reasoning");
    assert!(!source.is_valid(&value));

    let mut value = example("diagnostic-source-reference.json")?;
    value["event_sequence"] = json!(0);
    assert!(!source.is_valid(&value));

    let taxonomy = validator(&schemas, "diagnostic-taxonomy.schema.json")?;
    let mut value = example("diagnostic-taxonomy.json")?;
    value["definitions"]
        .as_array_mut()
        .ok_or_else(|| io::Error::other("canonical taxonomy definitions must be an array"))?[5]["category"] =
        json!("investigation");
    assert!(!taxonomy.is_valid(&value));
    Ok(())
}

#[test]
fn applicability_and_safety_invariants_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;

    let bottlenecks = validator(&schemas, "bottleneck-analysis.schema.json")?;
    let mut value = example("bottleneck-analysis.json")?;
    value["metrics"][1]["value"] = json!(0.5);
    assert!(!bottlenecks.is_valid(&value));

    let contribution = validator(&schemas, "contribution-analysis.schema.json")?;
    let mut value = example("contribution-analysis.json")?;
    value["experimental"] = json!(false);
    assert!(!contribution.is_valid(&value));

    let run_diagnosis = validator(&schemas, "run-diagnosis.schema.json")?;
    let mut value = example("run-diagnosis.json")?;
    value["recommendation_hypotheses"][0]["validation_required"] = json!(false);
    assert!(!run_diagnosis.is_valid(&value));

    let bundle = validator(&schemas, "diagnostic-bundle-manifest.schema.json")?;
    let mut value = example("diagnostic-bundle-manifest.json")?;
    value["artifacts"][0]["path"] = json!("../private/ground-truth.json");
    assert!(!bundle.is_valid(&value));
    Ok(())
}
