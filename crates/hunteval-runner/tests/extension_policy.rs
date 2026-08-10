use std::collections::BTreeSet;

use hunteval_domain::{
    ExtensionCapabilityPolicy, ExtensionConformanceStatus, ExtensionLimits, ExtensionNetworkPolicy,
    SchemaVersion, Sha256Digest,
};

#[test]
fn conformance_binds_exact_executable_and_policy() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let executable = directory.path().join("adapter");
    std::fs::write(&executable, b"adapter")?;
    let digest = Sha256Digest::from_bytes(b"adapter");
    let manifest = serde_json::json!({
        "schema_version":"0.9","id":"adapter","kind":"deployment_adapter",
        "executable_sha256":digest,"supported_versions":["0.3"],
        "requested_capabilities":[],"network":"denied","tools":[],
        "limits":{"wall_time_ms":1,"max_input_bytes":1,"max_output_bytes":1,"max_processes":1,"max_concurrency":1}
    });
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"policy"),
        allowed_capabilities: BTreeSet::new(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: ExtensionLimits {
            wall_time_ms: 1,
            max_input_bytes: 1,
            max_output_bytes: 1,
            max_processes: 1,
            max_concurrency: 1,
        },
    };
    let result =
        hunteval_runner::check_extension(&serde_json::to_vec(&manifest)?, &policy, &executable);
    assert_eq!(result.status, ExtensionConformanceStatus::Conformant);
    std::fs::write(&executable, b"changed")?;
    let result =
        hunteval_runner::check_extension(&serde_json::to_vec(&manifest)?, &policy, &executable);
    assert_eq!(result.status, ExtensionConformanceStatus::Rejected);

    let mut invalid_policy = policy;
    invalid_policy.maximum_limits.wall_time_ms = 0;
    let result = hunteval_runner::check_extension(
        &serde_json::to_vec(&manifest)?,
        &invalid_policy,
        &executable,
    );
    assert_eq!(result.status, ExtensionConformanceStatus::Rejected);
    assert!(
        result
            .reasons
            .iter()
            .any(|reason| reason == "invalid_policy")
    );
    Ok(())
}
