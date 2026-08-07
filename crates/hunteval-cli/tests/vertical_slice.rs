use std::{io, path::PathBuf, process::Command};

#[test]
fn vertical_slice_writes_replayable_normalized_artifacts() -> Result<(), Box<dyn std::error::Error>>
{
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))?
        .to_path_buf();
    let output = tempfile::tempdir()?;
    let status = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["run", "--episode"])
        .arg(workspace.join("datasets/aws/aws-iam-001"))
        .arg("--deployment")
        .arg(workspace.join("deployments/two-agent-scripted"))
        .arg("--output")
        .arg(output.path())
        .status()?;
    assert!(status.success());
    let root = output.path().join("latest");
    for artifact in [
        "trajectory.jsonl",
        "submission.json",
        "metrics.json",
        "result.json",
        "manifest.json",
    ] {
        assert!(root.join(artifact).is_file(), "missing {artifact}");
    }
    let result = std::fs::read_to_string(root.join("result.json"))?;
    assert!(result.contains("\"status\": \"completed\""));
    let inspect = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["trajectory", "inspect"])
        .arg(root.join("trajectory.jsonl"))
        .status()?;
    assert!(inspect.success());
    for artifact in [
        "trajectory.jsonl",
        "submission.json",
        "metrics.json",
        "result.json",
        "manifest.json",
    ] {
        let bytes = std::fs::read(root.join(artifact))?;
        let text = String::from_utf8_lossy(&bytes);
        assert!(!text.contains("acceptable_conclusions"));
        assert!(!text.contains("private/ground-truth.json"));
    }
    Ok(())
}
