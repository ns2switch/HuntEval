use std::{io, path::PathBuf, process::Command};

#[test]
fn diagnose_run_generates_and_verifies_offline_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))?
        .to_path_buf();
    let temporary = tempfile::tempdir()?;
    let runs = temporary.path().join("runs");
    let diagnosis = temporary.path().join("diagnosis");
    let run = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["run", "--episode"])
        .arg(workspace.join("datasets/aws/aws-iam-001"))
        .arg("--deployment")
        .arg(workspace.join("deployments/two-agent-scripted"))
        .arg("--output")
        .arg(&runs)
        .status()?;
    assert!(run.success());

    let generated = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["diagnose", "run"])
        .arg(runs.join("latest"))
        .arg("--output")
        .arg(&diagnosis)
        .status()?;
    assert!(generated.success());

    let verified = Command::new(env!("CARGO_BIN_EXE_hunteval"))
        .args(["diagnose", "verify"])
        .arg(&diagnosis)
        .args(["--format", "json"])
        .output()?;
    assert!(verified.status.success());
    let result: serde_json::Value = serde_json::from_slice(&verified.stdout)?;
    assert_eq!(result["status"], "verified");
    let html = std::fs::read_to_string(diagnosis.join("diagnostic-report.html"))?;
    assert!(!html.contains("<script"));
    assert!(!html.contains("private/ground-truth.json"));
    Ok(())
}
