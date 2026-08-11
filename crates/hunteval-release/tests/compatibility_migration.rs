use std::{collections::BTreeSet, fs, path::PathBuf};

use hunteval_release::{
    CompatibilityComponent, CompatibilityError, CompatibilityMatrix, CompatibilityStatus,
    InterfaceFreezeManifest, InterfaceInventory, MigrationAction, MigrationError,
    MigrationInventory, MigrationReceipt,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

fn root() -> Result<PathBuf, std::io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| std::io::Error::other("release crate is outside the workspace"))
}

fn load_freeze(root: &std::path::Path) -> Result<InterfaceFreezeManifest, std::io::Error> {
    serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/interface-freeze-manifest.json"),
    )?)
    .map_err(std::io::Error::other)
}

#[test]
fn canonical_matrix_is_deterministic_and_fixture_backed() -> Result<(), Box<dyn std::error::Error>>
{
    let root = root()?;
    let path = root.join("examples/contracts/v1.0/compatibility-matrix.json");
    let bytes = fs::read(path)?;
    let matrix: CompatibilityMatrix = serde_json::from_slice(&bytes)?;
    let freeze = load_freeze(&root)?;
    let interface_inventory: InterfaceInventory = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/release-interface-inventory.json"),
    )?)?;
    matrix.validate_against(&freeze)?;
    assert_eq!(matrix.normalized_json(&freeze)?, bytes);
    assert_eq!(
        matrix.markdown(&freeze)?,
        fs::read_to_string(root.join("docs/R8_COMPATIBILITY.md"))?
            .split("\n## Semantics")
            .next()
            .ok_or_else(|| std::io::Error::other("compatibility document has no projection"))?
            .to_owned()
    );

    for component in matrix.rules.iter().flat_map(|rule| &rule.components) {
        let interface = interface_inventory
            .interfaces
            .iter()
            .find(|entry| entry.interface_id == component.interface_id)
            .ok_or_else(|| std::io::Error::other("matrix interface is absent from inventory"))?;
        let fixture = interface
            .fixture_path
            .as_deref()
            .ok_or_else(|| std::io::Error::other("matrix interface has no fixture"))?;
        assert_eq!(
            component.fixture_sha256,
            hex::encode(Sha256::digest(fs::read(root.join(fixture))?))
        );
    }
    Ok(())
}

#[test]
fn matrix_rejects_ambiguity_ineligible_support_and_unknown_combinations()
-> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let freeze = load_freeze(&root)?;
    let mut matrix: CompatibilityMatrix = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/compatibility-matrix.json"),
    )?)?;
    matrix.rules.push(matrix.rules[0].clone());
    assert_eq!(
        matrix.validate_against(&freeze),
        Err(CompatibilityError::AmbiguousRule)
    );

    matrix.rules.pop();
    let preview = matrix
        .rules
        .iter_mut()
        .find(|rule| rule.combination_id == "framework-pack-preview")
        .ok_or_else(|| std::io::Error::other("preview rule is absent"))?;
    preview.status = CompatibilityStatus::Supported;
    preview.rejection_reason = None;
    preview.limitations.clear();
    assert_eq!(
        matrix.validate_against(&freeze),
        Err(CompatibilityError::IneligibleInterface)
    );

    let unknown = CompatibilityComponent {
        interface_id: "schema.unknown".to_owned(),
        version: "9.9".to_owned(),
        fixture_sha256: "0".repeat(64),
    };
    assert_eq!(
        matrix.rule_for(&[unknown]),
        Err(CompatibilityError::UnknownCombination)
    );
    Ok(())
}

#[test]
fn migration_inventory_is_exact_and_receipts_detect_changed_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let inventory: MigrationInventory = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/migration-inventory.json"),
    )?)?;
    inventory.validate()?;
    assert_eq!(
        inventory.decision("scoring-profile", "0.3")?.action,
        MigrationAction::AdaptInMemory
    );
    assert_eq!(
        inventory.decision("schema-future-major", "2.0")?.action,
        MigrationAction::Reject
    );
    assert_eq!(
        inventory.decision("unknown", "0.1"),
        Err(MigrationError::UndeclaredEdge)
    );

    let source = fs::read(root.join("examples/scoring-profile-balanced-v0.3.yaml"))?;
    let target = fs::read(root.join("examples/scoring-profile-balanced.yaml"))?;
    let receipt = inventory.receipt("scoring-profile-0.3-to-0.4", &source, &target)?;
    let expected: MigrationReceipt = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/migration-receipt.json"),
    )?)?;
    assert_eq!(receipt, expected);
    receipt.verify(&source, &target)?;
    let mut changed = target;
    changed.push(b' ');
    assert_eq!(
        receipt.verify(&source, &changed),
        Err(MigrationError::ReceiptMismatch)
    );
    Ok(())
}

#[test]
fn new_contracts_match_schemas_and_reject_unknown_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let root = root()?;
    for name in [
        "compatibility-matrix",
        "migration-inventory",
        "migration-receipt",
        "official-benchmark-pack",
        "platform-target-matrix",
        "r8-evidence-index",
    ] {
        let schema: Value = serde_json::from_slice(&fs::read(
            root.join(format!("schemas/v1.0/{name}.schema.json")),
        )?)?;
        jsonschema::meta::validate(&schema)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        let validator = jsonschema::validator_for(&schema)?;
        let value: Value = serde_json::from_slice(&fs::read(
            root.join(format!("examples/contracts/v1.0/{name}.json")),
        )?)?;
        assert!(validator.is_valid(&value), "invalid canonical {name}");
    }

    let bytes = fs::read(root.join("examples/contracts/v1.0/compatibility-matrix.json"))?;
    let mut value: Value = serde_json::from_slice(&bytes)?;
    value["unexpected"] = Value::Bool(true);
    assert!(serde_json::from_value::<CompatibilityMatrix>(value).is_err());

    let inventory: MigrationInventory = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/migration-inventory.json"),
    )?)?;
    let unique = inventory
        .edges
        .iter()
        .map(|edge| (&edge.artifact_family, &edge.source_version))
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), inventory.edges.len());
    Ok(())
}

#[test]
fn compatibility_matrix_hash_matches_migration_inventory() -> Result<(), Box<dyn std::error::Error>>
{
    let root = root()?;
    let matrix = fs::read(root.join("examples/contracts/v1.0/compatibility-matrix.json"))?;
    let inventory: MigrationInventory = serde_json::from_slice(&fs::read(
        root.join("examples/contracts/v1.0/migration-inventory.json"),
    )?)?;
    assert_eq!(
        inventory.compatibility_matrix_sha256,
        hex::encode(Sha256::digest(matrix))
    );
    Ok(())
}

#[test]
fn every_v1_schema_is_well_formed() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    for entry in fs::read_dir(root.join("schemas/v1.0"))? {
        let path = entry?.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
            continue;
        }
        let schema: Value = serde_json::from_slice(&fs::read(&path)?)?;
        jsonschema::meta::validate(&schema)
            .map_err(|error| std::io::Error::other(format!("{}: {error}", path.display())))?;
    }
    Ok(())
}
