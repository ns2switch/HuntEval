use hunteval_domain::{
    ArtifactRegistry, ImprovementExperiment, ImprovementPolicy, PromptFailureTaxonomy,
    RegisteredArtifact, StructuredArtifact,
};

fn example(name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/contracts/v0.8")
            .join(name),
    )?)
}

#[test]
fn typed_v08_contracts_validate_canonical_examples() -> Result<(), Box<dyn std::error::Error>> {
    let registered: RegisteredArtifact =
        serde_json::from_slice(&example("registered-artifact.json")?)?;
    registered.validate()?;
    let registry: ArtifactRegistry = serde_json::from_slice(&example("artifact-registry.json")?)?;
    registry.validate()?;
    let experiment: ImprovementExperiment =
        serde_json::from_slice(&example("improvement-experiment.json")?)?;
    experiment.validate()?;
    let _: hunteval_domain::ArtifactDiff = serde_json::from_slice(&example("artifact-diff.json")?)?;
    let _: hunteval_domain::ImprovementEquivalenceResult =
        serde_json::from_slice(&example("improvement-equivalence-result.json")?)?;
    let policy: ImprovementPolicy = serde_json::from_slice(&example("improvement-policy.json")?)?;
    policy.validate()?;
    let taxonomy: PromptFailureTaxonomy =
        serde_json::from_slice(&example("prompt-failure-taxonomy.json")?)?;
    taxonomy.validate()?;
    let recommendation: hunteval_domain::PromptRecommendation =
        serde_json::from_slice(&example("recommendation.json")?)?;
    recommendation.validate()?;
    let _: hunteval_domain::ControlledValidationDecision =
        serde_json::from_slice(&example("validation-decision.json")?)?;
    let event: hunteval_domain::RecommendationEvent =
        serde_json::from_slice(&example("recommendation-event.json")?)?;
    event.validate()?;
    let _: hunteval_domain::RecommendationState =
        serde_json::from_slice(&example("recommendation-state.json")?)?;
    let human: hunteval_domain::HumanDecision =
        serde_json::from_slice(&example("human-decision.json")?)?;
    human.validate()?;
    let adoption: hunteval_domain::AdoptionRecord =
        serde_json::from_slice(&example("adoption-record.json")?)?;
    adoption.validate()?;
    Ok(())
}

#[test]
fn structured_content_hashes_are_verified() -> Result<(), Box<dyn std::error::Error>> {
    let mut value: serde_json::Value =
        serde_json::from_slice(&example("structured-artifact.json")?)?;
    let content = value["sections"][0]["content"]
        .as_str()
        .ok_or("missing content")?;
    value["sections"][0]["sha256"] =
        serde_json::Value::String(hunteval_domain::Sha256Digest::from_bytes(content).to_string());
    let content = value["sections"][1]["content"]
        .as_str()
        .ok_or("missing content")?;
    value["sections"][1]["sha256"] =
        serde_json::Value::String(hunteval_domain::Sha256Digest::from_bytes(content).to_string());
    let artifact: StructuredArtifact = serde_json::from_value(value)?;
    artifact.validate()?;

    let mut tampered = artifact;
    tampered.sections[0].content.push_str(" changed");
    assert!(tampered.validate().is_err());
    Ok(())
}
