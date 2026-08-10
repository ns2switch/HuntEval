use std::collections::BTreeSet;

use hunteval_domain::{
    ArtifactReferenceV08, DiagnosticSourceReference, DiffOperationKind, EvidenceSufficiency,
    MutableSectionClass, ObservableSourceFamily, PromptFailureTaxonomy, PromptRecommendation,
    PromptWeaknessCode, ProposedStatus, RecommendationTarget, RecommendationTargetKind,
    RecommendationValidation, SchemaVersion, Sha256Digest, StructuredArtifact, SuggestedChange,
    SuspectedWeakness,
};
use thiserror::Error;

use super::prompt_rules::{CompiledPromptRule, RULES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptDiagnosticEvidence {
    pub diagnostic_code: String,
    pub source_families: BTreeSet<ObservableSourceFamily>,
    pub references: Vec<DiagnosticSourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedCandidate {
    pub bytes: Vec<u8>,
    pub artifact: StructuredArtifact,
}

pub fn materialize_suggestion(
    baseline: &StructuredArtifact,
    recommendation: &PromptRecommendation,
    target_class: MutableSectionClass,
    proposed_content: &str,
) -> Result<MaterializedCandidate, PromptAnalysisError> {
    baseline
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidArtifact)?;
    recommendation
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidRecommendation)?;
    if recommendation.target_artifact.sha256 != baseline.registered_artifact_sha256
        || proposed_content.trim().is_empty()
        || proposed_content.len() > 65_536
    {
        return Err(PromptAnalysisError::StaleOrUnsafeSuggestion);
    }
    let operation = recommendation.suggested_change.operation;
    let mut sections = baseline.sections.clone();
    let position = sections
        .iter()
        .position(|section| section.id == recommendation.target_section);
    match (operation, position) {
        (DiffOperationKind::AddSection, None) => sections.push(hunteval_domain::ArtifactSection {
            id: recommendation.target_section.clone(),
            policy: hunteval_domain::SectionPolicy::Mutable(target_class),
            content: proposed_content.to_owned(),
            sha256: Sha256Digest::from_bytes(proposed_content),
        }),
        (DiffOperationKind::ReplaceSection | DiffOperationKind::AddConstraint, Some(index)) => {
            let hunteval_domain::SectionPolicy::Mutable(class) = sections[index].policy else {
                return Err(PromptAnalysisError::StaleOrUnsafeSuggestion);
            };
            if class != target_class {
                return Err(PromptAnalysisError::StaleOrUnsafeSuggestion);
            }
            sections[index].content = proposed_content.to_owned();
            sections[index].sha256 = Sha256Digest::from_bytes(proposed_content);
        }
        (DiffOperationKind::RemoveSection, Some(index)) => {
            let hunteval_domain::SectionPolicy::Mutable(class) = sections[index].policy else {
                return Err(PromptAnalysisError::StaleOrUnsafeSuggestion);
            };
            if class != target_class || class == MutableSectionClass::OutputContract {
                return Err(PromptAnalysisError::StaleOrUnsafeSuggestion);
            }
            sections.remove(index);
        }
        _ => return Err(PromptAnalysisError::StaleOrUnsafeSuggestion),
    }
    let bytes =
        serde_json::to_vec(&sections).map_err(|_| PromptAnalysisError::InvalidRecommendation)?;
    let artifact = StructuredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: format!("{}-candidate", baseline.id),
        registered_artifact_sha256: Sha256Digest::from_bytes(&bytes),
        sections,
    };
    artifact
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidArtifact)?;
    Ok(MaterializedCandidate { bytes, artifact })
}

pub fn canonical_prompt_taxonomy() -> Result<PromptFailureTaxonomy, PromptAnalysisError> {
    let taxonomy: PromptFailureTaxonomy = serde_json::from_str(include_str!(
        "../../../../taxonomies/prompt-configuration-weaknesses-v1.json"
    ))
    .map_err(|_| PromptAnalysisError::InvalidTaxonomy)?;
    taxonomy
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidTaxonomy)?;
    validate_compiled_registry(&taxonomy)?;
    Ok(taxonomy)
}

pub fn analyze_prompt_weakness(
    recommendation_id: &str,
    target_id: &str,
    target_artifact_id: &str,
    target_artifact_sha256: Sha256Digest,
    artifact: &StructuredArtifact,
    evidence: &PromptDiagnosticEvidence,
) -> Result<PromptRecommendation, PromptAnalysisError> {
    artifact
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidArtifact)?;
    let taxonomy = canonical_prompt_taxonomy()?;
    let rule = RULES
        .iter()
        .find(|rule| {
            rule.diagnostic == evidence.diagnostic_code
                && rule
                    .sources
                    .iter()
                    .all(|source| evidence.source_families.contains(source))
        })
        .ok_or(PromptAnalysisError::InsufficientEvidence)?;
    let definition = taxonomy
        .definitions
        .iter()
        .find(|definition| definition.code == rule.weakness)
        .ok_or(PromptAnalysisError::InvalidTaxonomy)?;
    let (section_id, operation) = select_target(artifact, rule)?;
    let recommendation = PromptRecommendation {
        schema_version: SchemaVersion::new(0, 8),
        id: recommendation_id.to_owned(),
        target: RecommendationTarget {
            kind: RecommendationTargetKind::Agent,
            id: target_id.to_owned(),
        },
        issue_code: evidence.diagnostic_code.clone(),
        observed_evidence: evidence.references.clone(),
        suspected_weakness: SuspectedWeakness {
            code: definition.code,
            evidence_sufficiency: EvidenceSufficiency::Corroborated,
        },
        target_artifact: ArtifactReferenceV08 {
            id: target_artifact_id.to_owned(),
            sha256: target_artifact_sha256,
        },
        target_section: section_id,
        suggested_change: SuggestedChange {
            operation,
            rationale: rationale(definition.code).to_owned(),
        },
        expected_effects: BTreeSet::from([expected_effect(definition.code).to_owned()]),
        possible_trade_offs: BTreeSet::from(["requires_controlled_validation".to_owned()]),
        validation: RecommendationValidation {
            required: true,
            experiment_id: None,
        },
        status: ProposedStatus::Proposed,
    };
    recommendation
        .validate()
        .map_err(|_| PromptAnalysisError::InvalidRecommendation)?;
    Ok(recommendation)
}

fn select_target(
    artifact: &StructuredArtifact,
    rule: &CompiledPromptRule,
) -> Result<(String, DiffOperationKind), PromptAnalysisError> {
    for section in &artifact.sections {
        if let hunteval_domain::SectionPolicy::Mutable(class) = section.policy
            && rule.targets.contains(&class)
        {
            let operation = rule
                .operations
                .iter()
                .next()
                .copied()
                .ok_or(PromptAnalysisError::InvalidTaxonomy)?;
            return Ok((section.id.clone(), operation));
        }
    }
    let class = rule
        .targets
        .iter()
        .next()
        .copied()
        .ok_or(PromptAnalysisError::InvalidTaxonomy)?;
    if rule.operations.contains(&DiffOperationKind::AddSection) {
        Ok((
            default_section_id(class).to_owned(),
            DiffOperationKind::AddSection,
        ))
    } else {
        Err(PromptAnalysisError::MissingMutableTarget)
    }
}

fn validate_compiled_registry(taxonomy: &PromptFailureTaxonomy) -> Result<(), PromptAnalysisError> {
    if taxonomy.definitions.len() != RULES.len() {
        return Err(PromptAnalysisError::InvalidTaxonomy);
    }
    for rule in RULES {
        let definition = taxonomy
            .definitions
            .iter()
            .find(|definition| definition.code == rule.weakness)
            .ok_or(PromptAnalysisError::InvalidTaxonomy)?;
        let sources = rule.sources.iter().copied().collect::<BTreeSet<_>>();
        let targets = rule.targets.iter().copied().collect::<BTreeSet<_>>();
        let operations = rule.operations.iter().copied().collect::<BTreeSet<_>>();
        if definition.required_diagnostic_codes != BTreeSet::from([rule.diagnostic.to_owned()])
            || definition.required_source_families != sources
            || definition.target_section_classes != targets
            || definition.allowed_operations != operations
        {
            return Err(PromptAnalysisError::InvalidTaxonomy);
        }
    }
    Ok(())
}

fn default_section_id(class: MutableSectionClass) -> &'static str {
    match class {
        MutableSectionClass::TaskPlanning => "task_planning",
        MutableSectionClass::EvidenceRequirements => "evidence_requirements",
        MutableSectionClass::DelegationStrategy => "delegation",
        MutableSectionClass::StoppingConditions => "stopping_conditions",
        MutableSectionClass::CommunicationFormat => "communication_format",
        MutableSectionClass::ErrorRecovery => "error_recovery",
        MutableSectionClass::OutputContract => "output_contract",
    }
}

fn rationale(code: PromptWeaknessCode) -> &'static str {
    match code {
        PromptWeaknessCode::MissingTaskOwnership => {
            "Require one declared owner for every created task."
        }
        PromptWeaknessCode::MissingEvidenceRequirements => {
            "Require every finding to cite grounded evidence."
        }
        PromptWeaknessCode::InsufficientErrorHandling => {
            "Add a bounded recovery action for managed-tool errors."
        }
        _ => "Add the bounded observable contract identified by the registered weakness rule.",
    }
}

fn expected_effect(code: PromptWeaknessCode) -> &'static str {
    match code {
        PromptWeaknessCode::MissingTaskOwnership => "reduce_duplicate_work",
        PromptWeaknessCode::MissingEvidenceRequirements => "increase_grounded_findings",
        PromptWeaknessCode::InsufficientErrorHandling => "improve_tool_error_recovery",
        _ => "reduce_observed_failure_recurrence",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PromptAnalysisError {
    #[error("prompt weakness taxonomy is invalid")]
    InvalidTaxonomy,
    #[error("structured artifact is invalid")]
    InvalidArtifact,
    #[error("observable evidence does not satisfy a compiled weakness rule")]
    InsufficientEvidence,
    #[error("no eligible mutable target section exists")]
    MissingMutableTarget,
    #[error("prompt recommendation is invalid")]
    InvalidRecommendation,
    #[error("suggestion is stale, targets immutable policy, or cannot be represented safely")]
    StaleOrUnsafeSuggestion,
}
