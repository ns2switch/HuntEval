use std::{fs, io, path::PathBuf};

use hunteval_domain::SchemaVersion;
use hunteval_runner::{RunInputError, load_scoring_profile};

#[test]
fn profile_compatibility_loads_v04_and_adapts_v03_deterministically()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let current = load_scoring_profile(&root.join("examples/scoring-profile-balanced.yaml"))?;
    assert_eq!(current.schema_version, SchemaVersion::new(0, 4));
    assert_eq!(current.id, "balanced-0.4");

    let legacy_path = root.join("examples/scoring-profile-balanced-v0.3.yaml");
    let original = fs::read(&legacy_path)?;
    let first = load_scoring_profile(&legacy_path)?;
    let second = load_scoring_profile(&legacy_path)?;
    assert_eq!(first, second);
    assert_eq!(first.schema_version, SchemaVersion::new(0, 4));
    assert_eq!(
        first.metrics["event_recall"].version,
        SchemaVersion::new(0, 3)
    );
    assert_eq!(fs::read(legacy_path)?, original);
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn profile_compatibility_rejects_unknown_versions_and_metrics()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let path = temporary.path().join("profile.yaml");
    fs::write(
        &path,
        "schema_version: '0.5'\nid: invalid\nmissing_metric_policy: reject\nmetrics:\n  event_recall: {version: '0.3', weight: 1.0}\nconstraints: []\n",
    )?;
    assert!(matches!(
        load_scoring_profile(&path),
        Err(RunInputError::InvalidScoringProfile)
    ));

    fs::write(
        &path,
        "schema_version: '0.4'\nid: invalid\nmissing_metric_policy: reject\nmetrics:\n  invented_metric: {version: '0.4', weight: 1.0}\nconstraints: []\n",
    )?;
    assert!(matches!(
        load_scoring_profile(&path),
        Err(RunInputError::InvalidScoringProfile)
    ));

    fs::write(
        &path,
        "schema_version: '0.4'\nid: invalid\nmissing_metric_policy: reject\nmetrics:\n  event_recall: {version: '0.3', weight: 1.0}\nconstraints:\n  - kind: metric_threshold\n    code: minimum_event_recall\n    metric: {name: event_recall, version: '0.3'}\n    comparison: minimum\n    threshold: 0.5\n    disqualifying: true\n",
    )?;
    assert!(matches!(
        load_scoring_profile(&path),
        Err(RunInputError::InvalidScoringProfile)
    ));
    Ok(())
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| io::Error::other("runner crate is not inside the workspace"))
}
