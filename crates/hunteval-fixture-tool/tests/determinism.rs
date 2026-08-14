use std::{
    fs, io,
    path::{Path, PathBuf},
};

use hunteval_fixture_tool::{generate_expanded_catalog, generate_fixture};

type FileTree = Vec<(PathBuf, Vec<u8>)>;

#[test]
fn generator_is_byte_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("fixture crate is not inside the workspace"))?;
    let source = workspace_root.join("datasets/aws/aws-iam-001/source/events.json");
    let first_dir = tempfile::tempdir()?;
    let second_dir = tempfile::tempdir()?;
    let first = first_dir.path().join("cloudtrail.parquet");
    let second = second_dir.path().join("cloudtrail.parquet");

    generate_fixture(&source, &first)?;
    generate_fixture(&source, &second)?;
    assert_eq!(fs::read(first)?, fs::read(second)?);
    Ok(())
}

#[test]
fn generator_rejects_duplicate_event_identifiers() -> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let source = temporary.path().join("events.json");
    let output = temporary.path().join("telemetry/events.parquet");
    let event = serde_json::json!({
        "event_id": "evt-0001",
        "event_time": "2026-02-10T00:00:00Z",
        "provider": "aws",
        "account_id": "aws-scope-001",
        "principal": "operator",
        "event_name": "ListUsers",
        "resource": "aws:iam:resource-001",
        "source_ip": "198.51.100.10",
        "user_agent": "fixture/1.0"
    });
    fs::write(&source, serde_json::to_vec(&vec![&event, &event])?)?;
    assert!(matches!(
        generate_fixture(&source, &output),
        Err(hunteval_fixture_tool::FixtureGenerationError::DuplicateEventId)
    ));
    Ok(())
}

#[test]
fn expanded_catalog_is_byte_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let first = tempfile::tempdir()?;
    let second = tempfile::tempdir()?;
    generate_expanded_catalog(first.path())?;
    generate_expanded_catalog(second.path())?;

    let generated = tree(first.path())?;
    assert_eq!(generated, tree(second.path())?);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_datasets = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("fixture crate is not inside the workspace"))?
        .join("datasets");
    for (relative, expected) in generated {
        assert_eq!(
            fs::read(workspace_datasets.join(&relative))?,
            expected,
            "checked-in fixture differs from deterministic generation: {}",
            relative.display()
        );
    }
    Ok(())
}

fn tree(root: &Path) -> Result<FileTree, Box<dyn std::error::Error>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push((path.strip_prefix(root)?.to_path_buf(), fs::read(path)?));
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}
