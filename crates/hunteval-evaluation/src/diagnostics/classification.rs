use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    ClassificationOmission, DiagnosticClaimStrength, DiagnosticSourceReference,
    FailureClassification, RunId, SchemaVersion, Sha256Digest,
};

use super::{
    DiagnosticArtifactSet, canonical_taxonomy, classifier_registry_digest, evaluate_sufficiency,
    resolve_sources,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationCandidate {
    pub code: String,
    pub attribution_targets: BTreeSet<DiagnosticSourceReference>,
    pub evidence_sources: BTreeSet<DiagnosticSourceReference>,
    pub controlled_experiment_eligible: bool,
    pub topology_dependent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticInputV07 {
    pub run_id: RunId,
    pub artifacts: DiagnosticArtifactSet,
    pub candidates: Vec<ClassificationCandidate>,
}

pub fn classify_verified(
    input: &DiagnosticInputV07,
) -> Result<(Vec<FailureClassification>, Vec<ClassificationOmission>), Box<dyn std::error::Error>> {
    if input.run_id != input.artifacts.run_id || input.candidates.len() > 256 {
        return Err(std::io::Error::other("invalid or oversized diagnostic input").into());
    }
    let taxonomy = canonical_taxonomy()?;
    let taxonomy_sha256 = taxonomy.digest()?;
    let registry_sha256 = classifier_registry_digest();
    let mut classifications = BTreeMap::new();
    let mut omissions = Vec::new();
    for candidate in &input.candidates {
        let Some(definition) = taxonomy.definition(&candidate.code) else {
            omissions.push(omission(candidate, "unsupported_classification_code"));
            continue;
        };
        let resolved = match resolve_sources(&input.artifacts, &candidate.evidence_sources) {
            Ok(resolved) => resolved,
            Err(_) => {
                omissions.push(omission(candidate, "unresolved_evidence_source"));
                continue;
            }
        };
        if resolve_sources(&input.artifacts, &candidate.attribution_targets).is_err() {
            omissions.push(omission(candidate, "unresolved_attribution_target"));
            continue;
        }
        let Some(sufficiency) = evaluate_sufficiency(
            definition,
            &resolved,
            candidate.controlled_experiment_eligible,
        ) else {
            omissions.push(omission(candidate, "insufficient_evidence"));
            continue;
        };
        let id = classification_id(&input.run_id, candidate, taxonomy_sha256, registry_sha256)?;
        classifications.insert(
            id.clone(),
            FailureClassification {
                schema_version: SchemaVersion::new(0, 7),
                id,
                run_id: input.run_id.clone(),
                taxonomy_sha256,
                classifier_registry_sha256: registry_sha256,
                code: candidate.code.clone(),
                category: definition.category,
                attribution_targets: candidate.attribution_targets.clone(),
                evidence_sources: candidate.evidence_sources.clone(),
                source_families: sufficiency.observed_families,
                confidence: sufficiency.confidence,
                claim_strength: if sufficiency.confidence
                    == hunteval_domain::EvidenceConfidence::Controlled
                {
                    DiagnosticClaimStrength::Experimental
                } else {
                    DiagnosticClaimStrength::Observational
                },
                topology_dependent: candidate.topology_dependent,
                limitations: if sufficiency.confidence
                    == hunteval_domain::EvidenceConfidence::Controlled
                {
                    ["experimental", "topology_dependent"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect()
                } else {
                    ["observational_only"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect()
                },
            },
        );
    }
    omissions.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then(left.reason_code.cmp(&right.reason_code))
    });
    Ok((classifications.into_values().collect(), omissions))
}

fn omission(candidate: &ClassificationCandidate, reason: &str) -> ClassificationOmission {
    ClassificationOmission {
        code: candidate.code.clone(),
        reason_code: reason.to_owned(),
        available_sources: candidate.evidence_sources.clone(),
    }
}

fn classification_id(
    run_id: &RunId,
    candidate: &ClassificationCandidate,
    taxonomy: Sha256Digest,
    registry: Sha256Digest,
) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(&(
        run_id,
        &candidate.code,
        &candidate.attribution_targets,
        &candidate.evidence_sources,
        taxonomy,
        registry,
    ))?;
    Ok(format!(
        "classification:{}",
        Sha256Digest::from_bytes(bytes)
    ))
}
