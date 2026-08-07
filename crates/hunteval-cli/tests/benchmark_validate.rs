use std::{io, path::Path, path::PathBuf, process::Command};

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))
}

#[test]
fn benchmark_validate_resolves_all_referenced_artifacts() -> Result<(), Box<dyn std::error::Error>>
{
    let workspace = workspace_root()?;
    let output = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["benchmark", "validate"])
        .arg(workspace.join("examples/cloud-mvp-benchmark.yaml"))
        .arg("--artifact-root")
        .arg(&workspace)
        .output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?, "run cells: 36\n");
    Ok(())
}
