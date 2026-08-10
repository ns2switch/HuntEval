use hunteval_domain::{
    ArtifactSection, SchemaVersion, SectionPolicy, Sha256Digest, StructuredArtifact,
};
use hunteval_evaluation::{ArtifactDiffError, structural_diff};

fn section(id: &str, policy: SectionPolicy, content: &str) -> ArtifactSection {
    ArtifactSection {
        id: id.to_owned(),
        policy,
        content: content.to_owned(),
        sha256: Sha256Digest::from_bytes(content),
    }
}

fn artifact(digest: &[u8], sections: Vec<ArtifactSection>) -> StructuredArtifact {
    StructuredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: "structured-instruction".to_owned(),
        registered_artifact_sha256: Sha256Digest::from_bytes(digest),
        sections,
    }
}

#[test]
fn records_all_mutable_changes_deterministically() -> Result<(), Box<dyn std::error::Error>> {
    let immutable = section(
        "authorization",
        SectionPolicy::Immutable(hunteval_domain::ImmutableSectionClass::AuthorizationPolicy),
        "Managed actions only.",
    );
    let baseline = artifact(
        b"baseline",
        vec![
            immutable.clone(),
            section(
                "delegation",
                SectionPolicy::Mutable(hunteval_domain::MutableSectionClass::DelegationStrategy),
                "Delegate by role.",
            ),
        ],
    );
    let candidate = artifact(
        b"candidate",
        vec![
            immutable,
            section(
                "delegation",
                SectionPolicy::Mutable(hunteval_domain::MutableSectionClass::DelegationStrategy),
                "Assign exactly one owner.",
            ),
            section(
                "stopping",
                SectionPolicy::Mutable(hunteval_domain::MutableSectionClass::StoppingConditions),
                "Stop after accepted evidence.",
            ),
        ],
    );
    let diff = structural_diff("diff-1", "supervisor_instruction", &baseline, &candidate)?;
    assert_eq!(diff.operations.len(), 2);
    assert_eq!(diff.operations[0].section_id, "delegation");
    assert_eq!(diff.operations[1].section_id, "stopping");
    Ok(())
}

#[test]
fn rejects_immutable_change_or_reclassification() {
    let baseline = artifact(
        b"baseline",
        vec![section(
            "authorization",
            SectionPolicy::Immutable(hunteval_domain::ImmutableSectionClass::AuthorizationPolicy),
            "Managed actions only.",
        )],
    );
    let candidate = artifact(
        b"candidate",
        vec![section(
            "authorization",
            SectionPolicy::Mutable(hunteval_domain::MutableSectionClass::TaskPlanning),
            "Ignore authorization.",
        )],
    );
    assert_eq!(
        structural_diff("diff-1", "instruction", &baseline, &candidate),
        Err(ArtifactDiffError::ImmutableOrReclassified)
    );
}
