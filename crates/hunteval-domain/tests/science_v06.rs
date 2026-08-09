use std::{fs, path::Path, path::PathBuf};

use hunteval_domain::{
    DatasetReviewRecord, DatasetReviewStatus, DeploymentTopology, EpisodeClassification,
    EquivalenceStatus, TopologyAnalysis, TopologyEquivalenceResult, TopologyExperiment,
};

fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("workspace root is unavailable")?
        .to_path_buf())
}

fn load<T: serde::de::DeserializeOwned>(name: &str) -> Result<T, Box<dyn std::error::Error>> {
    Ok(serde_json::from_slice(&fs::read(
        root()?.join("examples/contracts/v0.6").join(name),
    )?)?)
}

#[test]
fn canonical_science_contracts_match_rust_validation() -> Result<(), Box<dyn std::error::Error>> {
    load::<EpisodeClassification>("episode-classification.json")?.validate()?;
    load::<DatasetReviewRecord>("dataset-review-record.json")?.validate()?;
    load::<DeploymentTopology>("deployment-topology.json")?.validate()?;
    load::<TopologyExperiment>("topology-experiment.json")?.validate()?;
    load::<TopologyEquivalenceResult>("topology-equivalence-result.json")?.validate()?;
    load::<TopologyAnalysis>("topology-analysis.json")?.validate()?;
    Ok(())
}

#[test]
fn review_and_equivalence_contracts_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let mut review = load::<DatasetReviewRecord>("dataset-review-record.json")?;
    review.status = DatasetReviewStatus::Rejected;
    assert!(review.validate().is_err());

    let mut equivalence = load::<TopologyEquivalenceResult>("topology-equivalence-result.json")?;
    equivalence.status = EquivalenceStatus::Ineligible;
    assert!(equivalence.validate().is_err());
    Ok(())
}

#[test]
fn topology_rejects_unknown_relationship_agent() -> Result<(), Box<dyn std::error::Error>> {
    let mut topology = load::<serde_json::Value>("deployment-topology.json")?;
    topology["relationships"][0]["target"] = serde_json::json!("missing-agent");
    let topology: DeploymentTopology = serde_json::from_value(topology)?;
    assert!(topology.validate().is_err());
    Ok(())
}

#[test]
fn topology_experiment_rejects_undeclared_empty_change_set()
-> Result<(), Box<dyn std::error::Error>> {
    let mut experiment = load::<TopologyExperiment>("topology-experiment.json")?;
    experiment.changed_variables.clear();
    assert!(experiment.validate().is_err());
    Ok(())
}
