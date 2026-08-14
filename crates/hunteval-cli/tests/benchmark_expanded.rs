use std::{io, path::Path, path::PathBuf, process::Command};

#[test]
fn expanded_benchmark_resolves_all_324_cells() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_root()?;
    let output = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .arg("benchmark")
        .arg("validate")
        .arg(workspace.join("examples/cloud-expanded-benchmark.yaml"))
        .arg("--artifact-root")
        .arg(&workspace)
        .output()?;
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(value["benchmark_id"], "cloud-expanded-r8");
    assert_eq!(value["run_cells"], 324);
    Ok(())
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))
}
