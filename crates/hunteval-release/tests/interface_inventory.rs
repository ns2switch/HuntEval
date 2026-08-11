use hunteval_release::{
    InterfaceEntry, InterfaceFreezeManifest, InterfaceInventory, InterfaceKind, InventoryError,
    PreconditionStatus, Projection, StabilityClass,
};
use serde_json::{Value, json};

fn entry(identifier: &str, stability: StabilityClass) -> InterfaceEntry {
    let pending = matches!(stability, StabilityClass::Blocked);
    InterfaceEntry {
        interface_id: identifier.to_owned(),
        kind: hunteval_release::InterfaceKind::Schema,
        owner: "release".to_owned(),
        stability,
        version_range: "1.0".to_owned(),
        fixture_path: Some("examples/contracts/result.json".to_owned()),
        verification_gate: Some("schema-contracts".to_owned()),
        projection: Projection::Public,
        authority: "normative".to_owned(),
        trust_boundary: "public-contract".to_owned(),
        bounds_documented: true,
        parser_behavior_documented: true,
        precondition_status: if pending {
            PreconditionStatus::Pending
        } else {
            PreconditionStatus::Satisfied
        },
        limitations: if pending {
            vec!["External conformance is pending.".to_owned()]
        } else {
            Vec::new()
        },
    }
}

fn inventory(interfaces: Vec<InterfaceEntry>) -> InterfaceInventory {
    InterfaceInventory {
        schema_version: "1.0".to_owned(),
        inventory_id: "r8-interface-inventory".to_owned(),
        baseline_revision: "4ac1ab77218e40ebb9f9bb4297d6e02fa6462f16".to_owned(),
        pre_r8_status: PreconditionStatus::Pending,
        interfaces,
    }
}

#[test]
fn freeze_manifest_is_order_independent_and_excludes_pending_interfaces()
-> Result<(), InventoryError> {
    let mut unavailable = entry("connector.unavailable", StabilityClass::Blocked);
    unavailable.precondition_status = PreconditionStatus::Unavailable;
    let first = inventory(vec![
        entry("schema.result", StabilityClass::StableCandidate),
        entry("connector.commercial", StabilityClass::Blocked),
        unavailable.clone(),
    ]);
    let second = inventory(vec![
        unavailable,
        entry("connector.commercial", StabilityClass::Blocked),
        entry("schema.result", StabilityClass::StableCandidate),
    ]);
    let left = first.freeze_manifest()?;
    let right = second.freeze_manifest()?;
    assert_eq!(left, right);
    assert_eq!(left.eligible_interfaces, ["schema.result"]);
    assert_eq!(left.exclusions[0].reason_code, "precondition_pending");
    assert_eq!(left.exclusions[1].reason_code, "precondition_unavailable");
    Ok(())
}

#[test]
fn pending_private_or_unverified_interfaces_cannot_claim_stability() {
    let mut candidate = entry("connector.pending", StabilityClass::StableCandidate);
    candidate.precondition_status = PreconditionStatus::Pending;
    assert_eq!(
        inventory(vec![candidate]).validate(),
        Err(InventoryError::IneligibleStableCandidate)
    );

    let mut private = entry("artifact.private", StabilityClass::StableCandidate);
    private.projection = Projection::EvaluatorPrivate;
    assert_eq!(
        inventory(vec![private]).validate(),
        Err(InventoryError::IneligibleStableCandidate)
    );

    let mut unverified = entry("schema.unverified", StabilityClass::StableCandidate);
    unverified.verification_gate = None;
    assert_eq!(
        inventory(vec![unverified]).validate(),
        Err(InventoryError::IneligibleStableCandidate)
    );
}

#[test]
fn malformed_duplicates_paths_and_unexplained_preview_fail_closed() {
    let duplicate = entry("schema.result", StabilityClass::StableCandidate);
    assert_eq!(
        inventory(vec![duplicate.clone(), duplicate]).validate(),
        Err(InventoryError::DuplicateInterface)
    );

    let mut traversal = entry("schema.traversal", StabilityClass::StableCandidate);
    traversal.fixture_path = Some("../private/ground-truth.json".to_owned());
    assert_eq!(
        inventory(vec![traversal]).validate(),
        Err(InventoryError::InvalidValue)
    );

    let preview = entry("connector.preview", StabilityClass::Preview);
    assert_eq!(
        inventory(vec![preview]).validate(),
        Err(InventoryError::InvalidValue)
    );

    let mut unknown_version = inventory(vec![entry(
        "schema.unsupported",
        StabilityClass::StableCandidate,
    )]);
    unknown_version.schema_version = "2.0".to_owned();
    assert_eq!(
        unknown_version.validate(),
        Err(InventoryError::UnsupportedVersion)
    );

    let mut contradictory = inventory(vec![entry("connector.pending", StabilityClass::Blocked)]);
    contradictory.pre_r8_status = PreconditionStatus::Satisfied;
    assert_eq!(contradictory.validate(), Err(InventoryError::InvalidValue));
}

#[test]
fn canonical_inventory_matches_schema_and_derived_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| std::io::Error::other("release crate is outside the workspace"))?
        .to_path_buf();
    let inventory_bytes =
        std::fs::read(root.join("examples/contracts/v1.0/release-interface-inventory.json"))?;
    let inventory: InterfaceInventory = serde_json::from_slice(&inventory_bytes)?;
    inventory.validate()?;
    let represented_kinds = inventory
        .interfaces
        .iter()
        .map(|interface| interface.kind)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        represented_kinds,
        [
            InterfaceKind::Artifact,
            InterfaceKind::Cli,
            InterfaceKind::CommercialConnector,
            InterfaceKind::Extension,
            InterfaceKind::FrameworkConnector,
            InterfaceKind::Knowledge,
            InterfaceKind::Metric,
            InterfaceKind::Platform,
            InterfaceKind::Protocol,
            InterfaceKind::Report,
            InterfaceKind::Schema,
            InterfaceKind::ScoringProfile,
            InterfaceKind::Sdk,
            InterfaceKind::Topology,
        ]
        .into_iter()
        .collect()
    );

    let schema: Value = serde_json::from_slice(&std::fs::read(
        root.join("schemas/v1.0/release-interface-inventory.schema.json"),
    )?)?;
    jsonschema::meta::validate(&schema)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let validator = jsonschema::validator_for(&schema)?;
    let value: Value = serde_json::from_slice(&inventory_bytes)?;
    assert!(validator.is_valid(&value));

    for interface in &inventory.interfaces {
        if matches!(
            interface.stability,
            StabilityClass::StableCandidate | StabilityClass::Retained
        ) {
            let fixture = interface
                .fixture_path
                .as_deref()
                .ok_or_else(|| std::io::Error::other("eligible interface has no fixture"))?;
            assert!(root.join(fixture).is_file(), "missing fixture: {fixture}");
        }
    }

    let manifest_path = root.join("examples/contracts/v1.0/interface-freeze-manifest.json");
    let manifest_bytes = std::fs::read(manifest_path)?;
    let expected: InterfaceFreezeManifest = serde_json::from_slice(&manifest_bytes)?;
    let manifest_schema: Value = serde_json::from_slice(&std::fs::read(
        root.join("schemas/v1.0/interface-freeze-manifest.schema.json"),
    )?)?;
    jsonschema::meta::validate(&manifest_schema)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    let manifest_validator = jsonschema::validator_for(&manifest_schema)?;
    let manifest_value: Value = serde_json::from_slice(&manifest_bytes)?;
    assert!(manifest_validator.is_valid(&manifest_value));
    assert_eq!(inventory.freeze_manifest()?, expected);

    let mut private_candidate = value;
    private_candidate["interfaces"][0]["projection"] = json!("evaluator_private");
    let malformed: InterfaceInventory = serde_json::from_value(private_candidate)?;
    assert_eq!(
        malformed.validate(),
        Err(InventoryError::IneligibleStableCandidate)
    );

    let mut unknown_field: Value = serde_json::from_slice(&inventory_bytes)?;
    unknown_field["unexpected"] = json!(true);
    assert!(serde_json::from_value::<InterfaceInventory>(unknown_field).is_err());
    Ok(())
}
