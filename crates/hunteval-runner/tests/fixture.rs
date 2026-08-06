use std::{fs, path::Path};

use hunteval_runner::{EpisodeLoadError, EpisodePackage};
use tempfile::TempDir;

fn canonical_fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("datasets/aws/aws-iam-001")
}

fn copy_fixture(destination: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(destination.join("public/telemetry"))?;
    fs::create_dir_all(destination.join("private"))?;
    for relative in [
        "package.yaml",
        "public/manifest.yaml",
        "public/telemetry/cloudtrail.parquet",
        "private/ground-truth.json",
    ] {
        fs::copy(
            canonical_fixture().join(relative),
            destination.join(relative),
        )?;
    }
    Ok(())
}

#[test]
fn fixture_loads_with_disjoint_public_and_private_state() -> Result<(), Box<dyn std::error::Error>>
{
    let package = EpisodePackage::load(canonical_fixture())?;
    let public_yaml = serde_yaml_ng::to_string(&package.public().manifest)?;

    assert_eq!(package.public().manifest.id.as_str(), "aws-iam-001");
    assert_eq!(package.ground_truth().malicious_event_ids.len(), 3);
    assert_eq!(package.digests().public_telemetry.len(), 1);
    assert!(!public_yaml.contains("ground_truth"));
    assert!(!public_yaml.contains("private"));
    Ok(())
}

#[test]
fn modified_public_artifact_changes_its_recorded_hash() -> Result<(), Box<dyn std::error::Error>> {
    let original = EpisodePackage::load(canonical_fixture())?;
    let temporary = TempDir::new()?;
    copy_fixture(temporary.path())?;
    let telemetry = temporary.path().join("public/telemetry/cloudtrail.parquet");
    let mut bytes = fs::read(&telemetry)?;
    bytes.push(0);
    fs::write(telemetry, bytes)?;

    let modified = EpisodePackage::load(temporary.path())?;
    assert_ne!(
        original.digests().public_telemetry,
        modified.digests().public_telemetry
    );
    Ok(())
}

#[test]
fn package_index_rejects_path_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let temporary = TempDir::new()?;
    fs::write(
        temporary.path().join("package.yaml"),
        "schema_version: '0.3'\nepisode_id: aws-iam-001\npublic_root: ../public\nprivate_ground_truth: private/ground-truth.json\n",
    )?;

    let error = EpisodePackage::load(temporary.path()).err();
    assert!(matches!(error, Some(EpisodeLoadError::UnsafePath)));
    Ok(())
}

#[cfg(unix)]
#[test]
fn package_loader_rejects_symlinked_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let temporary = TempDir::new()?;
    copy_fixture(temporary.path())?;
    let telemetry = temporary.path().join("public/telemetry/cloudtrail.parquet");
    fs::remove_file(&telemetry)?;
    symlink(
        canonical_fixture().join("public/telemetry/cloudtrail.parquet"),
        telemetry,
    )?;

    let error = EpisodePackage::load(temporary.path()).err();
    assert!(matches!(error, Some(EpisodeLoadError::UnsafePath)));
    Ok(())
}
