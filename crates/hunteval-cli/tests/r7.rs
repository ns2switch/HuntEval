use std::process::Command;

#[test]
fn validates_r7_corpus_and_extension_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let binary = env!("CARGO_BIN_EXE_hunteval");
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| std::io::Error::other("workspace root is unavailable"))?
        .to_path_buf();
    let corpus = Command::new(binary)
        .args(["knowledge", "validate"])
        .arg(root.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))
        .status()?;
    assert!(corpus.success());
    let build = Command::new(binary)
        .args(["knowledge", "build"])
        .arg(root.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))
        .arg("--root")
        .arg(&root)
        .status()?;
    assert!(build.success());
    let audit_root = tempfile::tempdir()?;
    let audit = audit_root.path().join("audit.jsonl");
    let query = Command::new(binary)
        .args(["knowledge", "query"])
        .arg(root.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))
        .arg(root.join("examples/contracts/v0.9/analytical-query.json"))
        .arg("--root")
        .arg(&root)
        .arg("--audit")
        .arg(&audit)
        .status()?;
    assert!(query.success());
    let html = Command::new(binary)
        .args(["knowledge", "query"])
        .arg(root.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))
        .arg(root.join("examples/contracts/v0.9/analytical-query.json"))
        .arg("--root")
        .arg(&root)
        .arg("--audit")
        .arg(&audit)
        .args(["--format", "html"])
        .output()?;
    assert!(html.status.success());
    assert!(String::from_utf8(html.stdout)?.starts_with("<!doctype html>"));
    let verify = Command::new(binary)
        .args(["knowledge", "verify"])
        .arg(root.join("examples/contracts/v0.9/analytical-corpus-manifest.json"))
        .arg("--root")
        .arg(&root)
        .arg("--audit")
        .arg(&audit)
        .status()?;
    assert!(verify.success());
    let extension = Command::new(binary)
        .args(["extension", "validate"])
        .arg(root.join("examples/contracts/v0.9/extension-manifest.json"))
        .arg("--policy")
        .arg(root.join("examples/contracts/v0.9/extension-capability-policy.json"))
        .status()?;
    assert!(extension.success());
    Ok(())
}
