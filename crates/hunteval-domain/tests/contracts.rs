use std::{fs, io, path::PathBuf};

use hunteval_domain::{
    Confidence, DeploymentRegistration, EpisodeManifest, GroundTruth, MetricValue, RunResult,
};

fn workspace_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|crates| crates.parent())
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))?;
    Ok(fs::read_to_string(workspace_root.join(path))?)
}

#[test]
fn canonical_contracts_round_trip_and_validate() -> Result<(), Box<dyn std::error::Error>> {
    let manifest: EpisodeManifest =
        serde_yaml_ng::from_str(&workspace_file("examples/contracts/episode-manifest.yaml")?)?;
    manifest.validate()?;
    let normalized_manifest = serde_yaml_ng::to_string(&manifest)?;
    let reparsed: EpisodeManifest = serde_yaml_ng::from_str(&normalized_manifest)?;
    assert_eq!(reparsed, manifest);

    let ground_truth: GroundTruth =
        serde_json::from_str(&workspace_file("examples/contracts/ground-truth.json")?)?;
    assert_eq!(ground_truth.episode_id, manifest.id);

    let deployment: DeploymentRegistration = serde_json::from_str(&workspace_file(
        "examples/contracts/deployment-registration.json",
    )?)?;
    deployment.validate(manifest.limits.max_agents)?;

    let result: RunResult =
        serde_json::from_str(&workspace_file("examples/contracts/result.json")?)?;
    result.validate()?;
    Ok(())
}

#[test]
fn public_manifest_rejects_ground_truth_fields() -> Result<(), Box<dyn std::error::Error>> {
    let public = workspace_file("examples/contracts/episode-manifest.yaml")?;
    let leaked = format!("{public}ground_truth_ref: private/ground-truth.json\n");
    assert!(serde_yaml_ng::from_str::<EpisodeManifest>(&leaked).is_err());
    Ok(())
}

#[test]
fn manifest_validation_rejects_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let public = workspace_file("examples/contracts/episode-manifest.yaml")?;
    let mut manifest: EpisodeManifest = serde_yaml_ng::from_str(&public)?;
    manifest.telemetry.tables[0].path = "../private/ground-truth.json".to_owned();
    assert!(manifest.validate().is_err());
    Ok(())
}

#[test]
fn confidence_rejects_invalid_numbers_during_deserialization() {
    for value in ["-0.1", "1.1", "null"] {
        assert!(serde_json::from_str::<Confidence>(value).is_err());
    }
}

#[test]
fn metric_validation_rejects_value_for_non_applicable_metric() -> Result<(), serde_json::Error> {
    let metric: MetricValue = serde_json::from_str(
        r#"{
            "value": 1.0,
            "applicability": "requires_repeated_runs",
            "direction": "higher_is_better",
            "range": {"minimum": 0.0, "maximum": 1.0},
            "numerator": null,
            "denominator": null
        }"#,
    )?;
    assert!(metric.validate().is_err());
    Ok(())
}
