#![cfg(unix)]

use std::{collections::BTreeSet, os::unix::fs::PermissionsExt, path::Path};

use hunteval_domain::{
    ExtensionCapabilityPolicy, ExtensionConformanceStatus, ExtensionLimits, ExtensionNetworkPolicy,
    SchemaVersion, Sha256Digest,
};

#[test]
fn malformed_crashed_and_timed_out_managed_tools_fail_closed()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    for (name, program, expected_reason) in [
        (
            "malformed",
            "#!/bin/sh\nprintf 'not-json'\n",
            "managed_tool_response",
        ),
        (
            "crash",
            "#!/bin/sh\nexit 7\n",
            "managed_tool_process_failure",
        ),
        ("timeout", "#!/bin/sh\nsleep 1\n", "managed_tool_timeout"),
    ] {
        let executable = directory.path().join(name);
        std::fs::write(&executable, program)?;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
        let (manifest, policy) = contracts(&executable)?;
        let result = hunteval_runner::conform_extension(&manifest, &policy, &executable, &[]);
        assert_eq!(result.status, ExtensionConformanceStatus::Rejected);
        assert!(
            result
                .reasons
                .iter()
                .any(|reason| reason == expected_reason),
            "{name}: {:?}",
            result.reasons
        );
        assert!(result.protocol_transcript_sha256.is_none());
    }
    Ok(())
}

fn contracts(
    executable: &Path,
) -> Result<(Vec<u8>, ExtensionCapabilityPolicy), Box<dyn std::error::Error>> {
    let limits = ExtensionLimits {
        wall_time_ms: 50,
        max_input_bytes: 1_048_576,
        max_output_bytes: 1_048_576,
        max_processes: 4,
        max_concurrency: 1,
    };
    let manifest = serde_json::to_vec(&serde_json::json!({
        "schema_version":"0.9",
        "id":"negative-managed-tool",
        "kind":"managed_tool",
        "executable_sha256":Sha256Digest::from_bytes(std::fs::read(executable)?),
        "supported_versions":["0.9"],
        "requested_capabilities":[],
        "network":"denied",
        "tools":["reference_query"],
        "limits":limits
    }))?;
    let policy = ExtensionCapabilityPolicy {
        schema_version: SchemaVersion::new(0, 9),
        policy_sha256: Sha256Digest::from_bytes(b"negative-tool-policy"),
        allowed_capabilities: BTreeSet::new(),
        network: ExtensionNetworkPolicy::Denied,
        maximum_limits: limits,
    };
    Ok((manifest, policy))
}
