use std::collections::BTreeSet;

use hunteval_domain::{DiagnosticSourceKind, EvidenceConfidence, FailureDefinition, SourceFamily};

use super::ResolvedDiagnosticSource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSufficiency {
    pub confidence: EvidenceConfidence,
    pub observed_kinds: BTreeSet<DiagnosticSourceKind>,
    pub observed_families: BTreeSet<SourceFamily>,
}

#[must_use]
pub fn evaluate_sufficiency(
    definition: &FailureDefinition,
    sources: &[ResolvedDiagnosticSource],
    controlled_experiment_eligible: bool,
) -> Option<EvidenceSufficiency> {
    let unique: BTreeSet<_> = sources.iter().map(|source| &source.0).collect();
    let observed_kinds: BTreeSet<_> = unique.iter().map(|source| source.kind()).collect();
    let observed_families: BTreeSet<_> = unique.iter().map(|source| source.family()).collect();
    if unique.len() < definition.minimum_sources
        || !definition.required_source_kinds.is_subset(&observed_kinds)
        || !definition
            .required_source_families
            .is_subset(&observed_families)
    {
        return None;
    }
    let confidence = if controlled_experiment_eligible
        && observed_families.contains(&SourceFamily::TopologyExperiment)
    {
        EvidenceConfidence::Controlled
    } else if observed_families.len() >= 2 {
        EvidenceConfidence::Corroborated
    } else {
        EvidenceConfidence::Direct
    };
    if confidence < definition.minimum_confidence {
        return None;
    }
    Some(EvidenceSufficiency {
        confidence,
        observed_kinds,
        observed_families,
    })
}
