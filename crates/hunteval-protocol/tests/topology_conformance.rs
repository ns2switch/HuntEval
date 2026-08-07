use std::{io, path::PathBuf};

#[test]
fn reference_topologies_are_framework_neutral_and_managed() -> Result<(), Box<dyn std::error::Error>>
{
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace unavailable"))?
        .join("deployments");
    for name in [
        "single-agent-scripted",
        "two-agent-scripted",
        "supervisor-specialists-scripted",
    ] {
        let text = std::fs::read_to_string(root.join(name).join("deployment.yaml"))?;
        let manifest: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text)?;
        assert_eq!(manifest["schema_version"], "0.4");
        assert_eq!(manifest["kind"], "external_reference_process");
        assert_eq!(manifest["scored_tools"], "hunteval_managed_only");
        assert_eq!(manifest["network_access"], false);
        assert_eq!(
            manifest["process"]["executable"],
            "bin/hunteval-reference-deployment"
        );
        assert_eq!(
            manifest["process"]["environment_allowlist"]
                .as_sequence()
                .map(Vec::len),
            Some(0)
        );
        assert!(
            manifest["agents"]
                .as_sequence()
                .is_some_and(|agents| !agents.is_empty())
        );
        assert!(!text.contains("ground_truth"));
    }
    Ok(())
}
