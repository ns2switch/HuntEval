use std::{fs, path::Path};

use hunteval_sandbox::ResolvedExecutionPolicy;

const CONTRACTS: &[(&str, &str)] = &[
    ("execution-policy.schema.json", "execution-policy.json"),
    (
        "sandbox-capability-report.schema.json",
        "sandbox-capability-report.json",
    ),
    (
        "protocol-conformance-result.schema.json",
        "protocol-conformance-result.json",
    ),
    (
        "run-verification-result.schema.json",
        "run-verification-result.json",
    ),
    ("secret-scan-result.schema.json", "secret-scan-result.json"),
];

#[test]
fn canonical_v05_examples_validate_offline() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (schema_name, example_name) in CONTRACTS {
        let schema: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("schemas/v0.5").join(schema_name))?)?;
        let example: serde_json::Value = serde_json::from_slice(&fs::read(
            root.join("examples/contracts/v0.5").join(example_name),
        )?)?;
        let validator = jsonschema::validator_for(&schema)?;
        assert!(
            validator.is_valid(&example),
            "invalid example: {example_name}"
        );
    }
    Ok(())
}

#[test]
fn execution_policy_example_matches_rust_contract() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = fs::read(root.join("examples/contracts/v0.5/execution-policy.json"))?;
    let parsed: ResolvedExecutionPolicy = serde_json::from_slice(&bytes)?;
    parsed.validate()?;
    assert_eq!(parsed, ResolvedExecutionPolicy::hardened_default());
    Ok(())
}
