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
        assert_eq!(manifest["scored_tools"], "hunteval_managed_only");
        assert_eq!(manifest["network_access"], false);
        assert!(
            manifest["agents"]
                .as_sequence()
                .is_some_and(|agents| !agents.is_empty())
        );
    }
    Ok(())
}
