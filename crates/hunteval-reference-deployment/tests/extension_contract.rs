use std::collections::BTreeSet;

use hunteval_domain::{
    ExtensionCapabilityPolicy, ExtensionConformanceStatus, ExtensionLimits, ExtensionNetworkPolicy,
    SchemaVersion, Sha256Digest,
};

#[test]
fn reference_deployment_passes_manifest_backed_conformance()
-> Result<(), Box<dyn std::error::Error>> {
    let executable = std::path::Path::new(env!("CARGO_BIN_EXE_hunteval-reference-deployment"));
    let executable_sha256 = Sha256Digest::from_bytes(std::fs::read(executable)?);
    let limits = ExtensionLimits {
        wall_time_ms: 10_000,
        max_input_bytes: 1_048_576,
        max_output_bytes: 1_048_576,
        max_processes: 16,
        max_concurrency: 4,
    };
    let manifest = serde_json::to_vec(&serde_json::json!({
        "schema_version":"0.9",
        "id":"reference-deployment",
        "kind":"deployment_adapter",
        "executable_sha256":executable_sha256,
        "supported_versions":["0.3"],
        "requested_capabilities":[],
        "network":"denied",
        "tools":[],
        "limits":limits
    }))?;
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"reference-extension-policy"),
        allowed_capabilities: BTreeSet::new(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: limits,
    };
    let arguments = ["--topology".to_owned(), "single-agent".to_owned()];
    let result = hunteval_runner::conform_extension(&manifest, &policy, executable, &arguments);
    assert_eq!(
        result.status,
        ExtensionConformanceStatus::Conformant,
        "conformance reasons: {:?}",
        result.reasons
    );
    assert!(result.checks.contains("deployment_protocol"));
    assert!(result.protocol_transcript_sha256.is_some());
    Ok(())
}

#[test]
fn reference_managed_tool_passes_manifest_backed_conformance()
-> Result<(), Box<dyn std::error::Error>> {
    let executable = std::path::Path::new(env!("CARGO_BIN_EXE_hunteval-reference-tool"));
    let executable_sha256 = Sha256Digest::from_bytes(std::fs::read(executable)?);
    let limits = ExtensionLimits {
        wall_time_ms: 10_000,
        max_input_bytes: 1_048_576,
        max_output_bytes: 1_048_576,
        max_processes: 4,
        max_concurrency: 1,
    };
    let manifest = serde_json::to_vec(&serde_json::json!({
        "schema_version":"0.9",
        "id":"reference-managed-tool",
        "kind":"managed_tool",
        "executable_sha256":executable_sha256,
        "supported_versions":["0.9"],
        "requested_capabilities":[],
        "network":"denied",
        "tools":["reference_query"],
        "limits":limits
    }))?;
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"reference-tool-policy"),
        allowed_capabilities: BTreeSet::new(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: limits,
    };
    let result = hunteval_runner::conform_extension(&manifest, &policy, executable, &[]);
    assert_eq!(
        result.status,
        ExtensionConformanceStatus::Conformant,
        "conformance reasons: {:?}",
        result.reasons
    );
    assert!(result.checks.contains("managed_tool_protocol"));
    assert!(result.protocol_transcript_sha256.is_some());
    Ok(())
}
