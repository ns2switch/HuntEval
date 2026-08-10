use std::collections::BTreeMap;

use hunteval_domain::{
    ArtifactDiff, ArtifactSection, DiffOperation, DiffOperationKind, ImprovementContractError,
    MutableSectionClass, SafetyStatus, SchemaVersion, StructuredArtifact,
};
use thiserror::Error;

pub fn structural_diff(
    id: &str,
    changed_variable: &str,
    baseline: &StructuredArtifact,
    candidate: &StructuredArtifact,
) -> Result<ArtifactDiff, ArtifactDiffError> {
    baseline.validate()?;
    candidate.validate()?;
    if baseline.registered_artifact_sha256 == candidate.registered_artifact_sha256 {
        return Err(ArtifactDiffError::NoChange);
    }
    let baseline_sections = indexed(&baseline.sections);
    let candidate_sections = indexed(&candidate.sections);
    let mut operations = Vec::new();

    for (section_id, section) in &baseline_sections {
        match candidate_sections.get(section_id) {
            Some(other) if section == other => {}
            Some(other) => operations.push(changed_section(section, other)?),
            None => operations.push(removed_section(section)?),
        }
    }
    for (section_id, section) in &candidate_sections {
        if !baseline_sections.contains_key(section_id) {
            operations.push(added_section(section)?);
        }
    }
    operations.sort_by(|left, right| left.section_id.cmp(&right.section_id));
    if operations.is_empty() {
        return Err(ArtifactDiffError::NoChange);
    }
    Ok(ArtifactDiff {
        schema_version: SchemaVersion::new(0, 8),
        id: id.to_owned(),
        baseline_artifact_sha256: baseline.registered_artifact_sha256,
        candidate_artifact_sha256: candidate.registered_artifact_sha256,
        changed_variable: changed_variable.to_owned(),
        operations,
        immutable_policy_status: SafetyStatus::Passed,
        reason_codes: Vec::new(),
    })
}

fn indexed(sections: &[ArtifactSection]) -> BTreeMap<&str, &ArtifactSection> {
    sections
        .iter()
        .map(|section| (section.id.as_str(), section))
        .collect()
}

fn changed_section(
    baseline: &ArtifactSection,
    candidate: &ArtifactSection,
) -> Result<DiffOperation, ArtifactDiffError> {
    match (baseline.policy, candidate.policy) {
        (
            hunteval_domain::SectionPolicy::Mutable(baseline_class),
            hunteval_domain::SectionPolicy::Mutable(candidate_class),
        ) if baseline_class == candidate_class => Ok(DiffOperation {
            operation: DiffOperationKind::ReplaceSection,
            section_id: baseline.id.clone(),
            section_class: baseline_class,
            baseline_sha256: Some(baseline.sha256),
            candidate_sha256: Some(candidate.sha256),
        }),
        _ => Err(ArtifactDiffError::ImmutableOrReclassified),
    }
}

fn removed_section(section: &ArtifactSection) -> Result<DiffOperation, ArtifactDiffError> {
    let hunteval_domain::SectionPolicy::Mutable(class) = section.policy else {
        return Err(ArtifactDiffError::ImmutableOrReclassified);
    };
    Ok(operation(
        DiffOperationKind::RemoveSection,
        section,
        class,
        Some(section.sha256),
        None,
    ))
}

fn added_section(section: &ArtifactSection) -> Result<DiffOperation, ArtifactDiffError> {
    let hunteval_domain::SectionPolicy::Mutable(class) = section.policy else {
        return Err(ArtifactDiffError::ImmutableOrReclassified);
    };
    Ok(operation(
        DiffOperationKind::AddSection,
        section,
        class,
        None,
        Some(section.sha256),
    ))
}

fn operation(
    operation: DiffOperationKind,
    section: &ArtifactSection,
    section_class: MutableSectionClass,
    baseline_sha256: Option<hunteval_domain::Sha256Digest>,
    candidate_sha256: Option<hunteval_domain::Sha256Digest>,
) -> DiffOperation {
    DiffOperation {
        operation,
        section_id: section.id.clone(),
        section_class,
        baseline_sha256,
        candidate_sha256,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ArtifactDiffError {
    #[error("structured artifact is invalid")]
    InvalidArtifact,
    #[error("baseline and candidate do not contain an observable structural change")]
    NoChange,
    #[error("candidate changes, removes, adds, or reclassifies immutable structure")]
    ImmutableOrReclassified,
}

impl From<ImprovementContractError> for ArtifactDiffError {
    fn from(_: ImprovementContractError) -> Self {
        Self::InvalidArtifact
    }
}
