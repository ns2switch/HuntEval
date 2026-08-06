use std::{fs, io, path::PathBuf};

use hunteval_fixture_tool::generate_fixture;

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
