use std::{collections::HashMap, fs, io, path::Path, path::PathBuf};

use jsonschema::{Retrieve, Uri};
use serde_json::{Value, json};

const CONTRACTS: &[(&str, &str)] = &[
    (
        "registered-artifact.schema.json",
        "registered-artifact.json",
    ),
    (
        "structured-artifact.schema.json",
        "structured-artifact.json",
    ),
    ("artifact-registry.schema.json", "artifact-registry.json"),
    ("artifact-diff.schema.json", "artifact-diff.json"),
    ("improvement-policy.schema.json", "improvement-policy.json"),
    (
        "improvement-experiment.schema.json",
        "improvement-experiment.json",
    ),
    (
        "improvement-equivalence-result.schema.json",
        "improvement-equivalence-result.json",
    ),
    (
        "validation-decision.schema.json",
        "validation-decision.json",
    ),
    ("recommendation.schema.json", "recommendation.json"),
    (
        "recommendation-event.schema.json",
        "recommendation-event.json",
    ),
    (
        "recommendation-state.schema.json",
        "recommendation-state.json",
    ),
    ("human-decision.schema.json", "human-decision.json"),
    ("adoption-record.schema.json", "adoption-record.json"),
    (
        "prompt-failure-taxonomy.schema.json",
        "prompt-failure-taxonomy.json",
    ),
    ("improvement-report.schema.json", "improvement-report.json"),
    (
        "improvement-bundle-manifest.schema.json",
        "improvement-bundle-manifest.json",
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
    let mut documents = HashMap::new();
    for version in ["v0.7", "v0.8"] {
        for entry in fs::read_dir(workspace_root()?.join("schemas").join(version))? {
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
    }
    Ok(RepositorySchemas { documents })
}

fn example(version: &str, name: &str) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(
        workspace_root()?
            .join("examples/contracts")
            .join(version)
            .join(name),
    )?)?)
}

fn validator(
    schemas: &RepositorySchemas,
    version: &str,
    schema_name: &str,
) -> Result<jsonschema::Validator, Box<dyn std::error::Error>> {
    let id = format!("https://hunteval.dev/schemas/{version}/{schema_name}");
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
fn canonical_v08_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;
    let v08_count = schemas
        .documents
        .keys()
        .filter(|id| id.contains("/schemas/v0.8/"))
        .count();
    assert_eq!(v08_count, CONTRACTS.len() + 1);
    for (schema_name, example_name) in CONTRACTS {
        validator(&schemas, "v0.8", schema_name)?
            .validate(&example("v0.8", example_name)?)
            .map_err(|error| io::Error::other(format!("{example_name}: {error}")))?;
    }
    Ok(())
}

#[test]
fn v08_contracts_reject_private_future_and_unknown_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let schemas = load_schemas()?;
    let recommendation = validator(&schemas, "v0.8", "recommendation.schema.json")?;
    let mut value = example("v0.8", "recommendation.json")?;
    value["private_ground_truth"] = json!({"answer": "hidden"});
    assert!(!recommendation.is_valid(&value));

    let mut value = example("v0.8", "recommendation.json")?;
    value["schema_version"] = json!("0.9");
    assert!(!recommendation.is_valid(&value));

    let report = validator(&schemas, "v0.8", "improvement-report.schema.json")?;
    let mut value = example("v0.8", "improvement-report.json")?;
    value["hidden_test_results"] = json!([0.99]);
    assert!(!report.is_valid(&value));
    Ok(())
}

#[test]
fn immutable_hidden_test_and_adoption_rules_fail_closed() -> Result<(), Box<dyn std::error::Error>>
{
    let schemas = load_schemas()?;

    let structured = validator(&schemas, "v0.8", "structured-artifact.schema.json")?;
    let mut value = example("v0.8", "structured-artifact.json")?;
    value["sections"][0]["mutability"] = json!("mutable");
    assert!(!structured.is_valid(&value));

    let policy = validator(&schemas, "v0.8", "improvement-policy.schema.json")?;
    let mut value = example("v0.8", "improvement-policy.json")?;
    value["immutable_section_classes"]
        .as_array_mut()
        .ok_or_else(|| io::Error::other("immutable classes must be an array"))?
        .pop();
    assert!(!policy.is_valid(&value));
    let mut value = example("v0.8", "improvement-policy.json")?;
    value["autonomous_adoption"] = json!(true);
    assert!(!policy.is_valid(&value));

    let decision = validator(&schemas, "v0.8", "validation-decision.schema.json")?;
    let mut value = example("v0.8", "validation-decision.json")?;
    value["hidden_test_used_in_selection"] = json!(true);
    assert!(!decision.is_valid(&value));
    let mut value = example("v0.8", "validation-decision.json")?;
    value["constraints"][0]["status"] = json!("violated");
    assert!(!decision.is_valid(&value));
    let mut value = example("v0.8", "validation-decision.json")?;
    value["metric_deltas"][0]["applicability"] = json!("unavailable");
    assert!(!decision.is_valid(&value));

    let human = validator(&schemas, "v0.8", "human-decision.schema.json")?;
    let mut value = example("v0.8", "human-decision.json")?;
    value["explicit_confirmation"] = json!(false);
    assert!(!human.is_valid(&value));

    let adoption = validator(&schemas, "v0.8", "adoption-record.schema.json")?;
    let mut value = example("v0.8", "adoption-record.json")?;
    value["external_adoption_confirmed"] = json!(false);
    assert!(!adoption.is_valid(&value));
    Ok(())
}

#[test]
fn lifecycle_taxonomy_and_bundle_rules_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;

    let event = validator(&schemas, "v0.8", "recommendation-event.schema.json")?;
    let mut value = example("v0.8", "recommendation-event.json")?;
    value["validation_decision_sha256"] = Value::Null;
    assert!(!event.is_valid(&value));

    let state = validator(&schemas, "v0.8", "recommendation-state.schema.json")?;
    let mut value = example("v0.8", "recommendation-state.json")?;
    value["status"] = json!("adopted");
    assert!(!state.is_valid(&value));

    let taxonomy = validator(&schemas, "v0.8", "prompt-failure-taxonomy.schema.json")?;
    let mut value = example("v0.8", "prompt-failure-taxonomy.json")?;
    value["definitions"][0]["executable_rule"] = json!("read('/private')");
    assert!(!taxonomy.is_valid(&value));

    let bundle = validator(&schemas, "v0.8", "improvement-bundle-manifest.schema.json")?;
    let mut value = example("v0.8", "improvement-bundle-manifest.json")?;
    value["artifacts"][0]["path"] = json!("../private/ground-truth.json");
    assert!(!bundle.is_valid(&value));
    Ok(())
}

#[test]
fn schema_v07_remains_valid_and_is_not_relabelled() -> Result<(), Box<dyn std::error::Error>> {
    let schemas = load_schemas()?;
    let legacy = example("v0.7", "run-diagnosis.json")?;
    validator(&schemas, "v0.7", "run-diagnosis.schema.json")?
        .validate(&legacy)
        .map_err(|error| io::Error::other(error.to_string()))?;

    let registered = validator(&schemas, "v0.8", "registered-artifact.schema.json")?;
    assert!(!registered.is_valid(&legacy));
    Ok(())
}
