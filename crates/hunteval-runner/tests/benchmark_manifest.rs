use std::{fs, io, path::Path, path::PathBuf};

use hunteval_domain::SchemaVersion;
use hunteval_runner::{BenchmarkError, load_benchmark, resolve_benchmark};
use tempfile::TempDir;

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("runner crate is not inside the workspace"))
}

fn write_fixture(root: &Path) -> Result<(), io::Error> {
    fs::create_dir_all(root.join("deployments/a"))?;
    fs::create_dir_all(root.join("deployments/b"))?;
    fs::create_dir_all(root.join("episodes/a"))?;
    fs::create_dir_all(root.join("episodes/b"))?;
    fs::create_dir_all(root.join("profiles"))?;
    fs::write(
        root.join("deployments/a/deployment.yaml"),
        "schema_version: '0.4'\nid: deployment-a\n",
    )?;
    fs::write(
        root.join("deployments/b/deployment.yaml"),
        "schema_version: '0.4'\nid: deployment-b\n",
    )?;
    fs::write(
        root.join("episodes/a/package.yaml"),
        "schema_version: '0.4'\nepisode_id: episode-a\n",
    )?;
    fs::write(
        root.join("episodes/b/package.yaml"),
        "schema_version: '0.4'\nepisode_id: episode-b\n",
    )?;
    fs::write(
        root.join("profiles/balanced.yaml"),
        "schema_version: '0.4'\nid: balanced:1.0.0\n",
    )?;
    Ok(())
}

fn manifest(deployments: &[&str], episodes: &[&str], seeds: &[u64]) -> String {
    let deployments = deployments
        .iter()
        .map(|value| format!("  - {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    let episodes = episodes
        .iter()
        .map(|value| format!("  - {value}"))
        .collect::<Vec<_>>()
        .join("\n");
    let seeds = seeds
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "schema_version: '0.4'\nid: benchmark-test\ndeployments:\n{deployments}\nepisodes:\n{episodes}\nseeds: [{seeds}]\nscoring_profile: profiles/balanced.yaml\nfault_profile: null\n"
    )
}

#[test]
fn benchmark_manifest_adapts_v03_into_a_resolved_v04_definition()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let definition = resolve_benchmark(&root.join("examples/cloud-mvp-benchmark.yaml"), &root)?;
    assert_eq!(definition.schema_version, SchemaVersion::new(0, 4));
    assert_eq!(definition.cell_count()?, 108);
    assert_eq!(definition.cells()?.len(), 108);

    let expanded = resolve_benchmark(&root.join("examples/cloud-expanded-benchmark.yaml"), &root)?;
    assert_eq!(expanded.cell_count()?, 324);
    assert_eq!(expanded.cells()?.len(), 324);
    Ok(())
}

#[test]
fn benchmark_manifest_resolution_is_order_independent_and_digest_sensitive()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TempDir::new()?;
    write_fixture(directory.path())?;
    let first_path = directory.path().join("first.yaml");
    let second_path = directory.path().join("second.yaml");
    fs::write(
        &first_path,
        manifest(
            &["deployments/b", "deployments/a"],
            &["episodes/b", "episodes/a"],
            &[29, 11],
        ),
    )?;
    fs::write(
        &second_path,
        manifest(
            &["deployments/a", "deployments/b"],
            &["episodes/a", "episodes/b"],
            &[11, 29],
        ),
    )?;
    let first = resolve_benchmark(&first_path, directory.path())?;
    let second = resolve_benchmark(&second_path, directory.path())?;
    assert_eq!(first, second);

    fs::write(
        directory.path().join("deployments/a/extra.conf"),
        "changed configuration bytes",
    )?;
    let changed = resolve_benchmark(&second_path, directory.path())?;
    assert_ne!(first.cells()?, changed.cells()?);
    Ok(())
}

#[test]
fn benchmark_manifest_rejects_duplicates_traversal_and_unknown_versions()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = TempDir::new()?;
    let path = directory.path().join("benchmark.yaml");
    fs::write(
        &path,
        "schema_version: '0.4'\nid: test\ndeployments: [same, same]\nepisodes: [episode]\nseeds: [1]\nscoring_profile: profile.yaml\n",
    )?;
    assert!(matches!(
        load_benchmark(&path),
        Err(BenchmarkError::DuplicateDimension("deployment"))
    ));

    fs::write(
        &path,
        "schema_version: '0.4'\nid: test\ndeployments: [../private]\nepisodes: [episode]\nseeds: [1]\nscoring_profile: profile.yaml\n",
    )?;
    assert!(matches!(
        load_benchmark(&path),
        Err(BenchmarkError::UnsafePath)
    ));

    fs::write(
        &path,
        "schema_version: '0.5'\nid: test\ndeployments: [deployment]\nepisodes: [episode]\nseeds: [1]\nscoring_profile: profile.yaml\n",
    )?;
    assert!(matches!(
        load_benchmark(&path),
        Err(BenchmarkError::UnsupportedSchema(_))
    ));
    Ok(())
}

#[cfg(unix)]
#[test]
fn benchmark_manifest_rejects_symlinked_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let directory = TempDir::new()?;
    write_fixture(directory.path())?;
    let manifest_path = directory.path().join("benchmark.yaml");
    fs::write(
        &manifest_path,
        manifest(&["deployments/link"], &["episodes/a"], &[11]),
    )?;
    std::os::unix::fs::symlink("a", directory.path().join("deployments/link"))?;
    assert!(matches!(
        resolve_benchmark(&manifest_path, directory.path()),
        Err(BenchmarkError::SymlinkArtifact)
    ));
    Ok(())
}
