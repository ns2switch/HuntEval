use std::collections::BTreeSet;

use hunteval_domain::{
    ArtifactSection, DiagnosticSourceReference, MutableSectionClass, ObservableSourceFamily, RunId,
    SchemaVersion, SectionPolicy, Sha256Digest, StructuredArtifact,
};
use hunteval_evaluation::{
    PromptDiagnosticEvidence, analyze_prompt_weakness, materialize_suggestion,
};

#[test]
fn recommendation_cites_observable_sources_and_materializes_separately()
-> Result<(), Box<dyn std::error::Error>> {
    let content = "Delegate investigations by capability.";
    let baseline = StructuredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: "supervisor-instruction".into(),
        registered_artifact_sha256: Sha256Digest::from_bytes(b"baseline"),
        sections: vec![ArtifactSection {
            id: "delegation".into(),
            policy: SectionPolicy::Mutable(MutableSectionClass::DelegationStrategy),
            content: content.into(),
            sha256: Sha256Digest::from_bytes(content),
        }],
    };
    let run_id: RunId = "run-r6-001".parse()?;
    let evidence = PromptDiagnosticEvidence {
        diagnostic_code: "duplicate_task_creation".into(),
        source_families: BTreeSet::from([
            ObservableSourceFamily::Task,
            ObservableSourceFamily::Coordination,
        ]),
        references: vec![DiagnosticSourceReference::Task {
            run_id,
            entity_id: "task-duplicate".into(),
            artifact_sha256: Sha256Digest::from_bytes(b"trajectory"),
        }],
    };
    let recommendation = analyze_prompt_weakness(
        "recommendation-1",
        "supervisor",
        "supervisor-instruction",
        baseline.registered_artifact_sha256,
        &baseline,
        &evidence,
    )?;
    assert!(recommendation.validation.required);
    let materialized = materialize_suggestion(
        &baseline,
        &recommendation,
        MutableSectionClass::DelegationStrategy,
        "Assign exactly one declared owner to every task.",
    )?;
    assert_ne!(
        materialized.artifact.registered_artifact_sha256,
        baseline.registered_artifact_sha256
    );
    assert_eq!(baseline.sections[0].content, content);
    assert_eq!(
        recommendation.target.kind,
        hunteval_domain::RecommendationTargetKind::Agent
    );
    Ok(())
}
