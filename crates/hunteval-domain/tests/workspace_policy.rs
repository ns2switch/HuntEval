use std::{fs, io, path::PathBuf};

fn workspace_manifest() -> Result<String, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))?;
    Ok(fs::read_to_string(workspace_root.join("Cargo.toml"))?)
}

#[test]
fn workspace_forbids_unsafe_and_panic_shortcuts() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = workspace_manifest()?;

    for required_policy in [
        "unsafe_code = \"forbid\"",
        "expect_used = \"deny\"",
        "panic = \"deny\"",
        "todo = \"deny\"",
        "unimplemented = \"deny\"",
        "unwrap_used = \"deny\"",
    ] {
        assert!(
            manifest.contains(required_policy),
            "workspace is missing policy: {required_policy}"
        );
    }

    Ok(())
}
