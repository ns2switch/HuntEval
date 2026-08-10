use std::collections::BTreeSet;

use hunteval_domain::{
    ExtensionCapability, ExtensionCapabilityPolicy, ExtensionKind, ExtensionLimits,
    ExtensionManifest, ExtensionNetworkPolicy, ExtensionResolution, ExtensionResolutionStatus,
    ManagedToolAdapterRequest, ManagedToolAdapterResponse, SchemaVersion, Sha256Digest,
};

fn limits(value: u64) -> ExtensionLimits {
    ExtensionLimits {
        wall_time_ms: value,
        max_input_bytes: value,
        max_output_bytes: value,
        max_processes: value as u32,
        max_concurrency: value as u32,
    }
}

#[test]
fn managed_tool_messages_are_bounded_and_correlated() {
    let request = ManagedToolAdapterRequest {
        schema_version: SchemaVersion::new(0, 9),
        request_id: "request-001".to_owned(),
        tool: "reference_query".to_owned(),
        arguments: serde_json::json!({"value":1}),
    };
    assert!(request.validate().is_ok());
    let response = ManagedToolAdapterResponse::Success {
        schema_version: SchemaVersion::new(0, 9),
        request_id: request.request_id.clone(),
        result: serde_json::json!({"accepted":true}),
    };
    assert!(response.validate().is_ok());
    assert_eq!(response.request_id(), request.request_id);

    let oversized = ManagedToolAdapterRequest {
        arguments: serde_json::json!({"value":"x".repeat(1_048_576)}),
        ..request
    };
    assert!(oversized.validate().is_err());
}

#[test]
fn capability_resolution_is_deny_by_default() {
    let digest = Sha256Digest::from_bytes(b"manifest");
    let manifest = ExtensionManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "local-tool".to_owned(),
        kind: ExtensionKind::ManagedTool,
        executable_sha256: digest,
        supported_versions: BTreeSet::from([SchemaVersion::new(0, 3)]),
        requested_capabilities: BTreeSet::from([ExtensionCapability::LocalReadOnlyData]),
        network: ExtensionNetworkPolicy::Denied,
        tools: BTreeSet::from(["query".to_owned()]),
        limits: limits(10),
    };
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"policy"),
        allowed_capabilities: BTreeSet::new(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: limits(20),
    };
    let resolution = ExtensionResolution::resolve(digest, &manifest, &policy);
    assert_eq!(resolution.status, ExtensionResolutionStatus::Rejected);
    assert!(resolution.granted_capabilities.is_empty());
    assert_eq!(resolution.reasons, ["capability_not_allowed"]);
}

#[test]
fn valid_minimal_extension_is_eligible() -> Result<(), Box<dyn std::error::Error>> {
    let digest = Sha256Digest::from_bytes(b"manifest");
    let manifest = ExtensionManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "deployment-peer".to_owned(),
        kind: ExtensionKind::DeploymentAdapter,
        executable_sha256: digest,
        supported_versions: BTreeSet::from([SchemaVersion::new(0, 3)]),
        requested_capabilities: BTreeSet::from([ExtensionCapability::ManagedToolRequest]),
        network: ExtensionNetworkPolicy::Denied,
        tools: BTreeSet::new(),
        limits: limits(10),
    };
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"policy"),
        allowed_capabilities: manifest.requested_capabilities.clone(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: limits(10),
    };
    manifest.validate()?;
    let resolution = ExtensionResolution::resolve(digest, &manifest, &policy);
    assert_eq!(resolution.status, ExtensionResolutionStatus::Eligible);
    Ok(())
}
