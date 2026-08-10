use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{DiagnosticSourceReference, EvidenceConfidence, FailureCategory, SourceFamily};
use crate::{RunId, SchemaVersion, Sha256Digest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureClassification {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub run_id: RunId,
    pub taxonomy_sha256: Sha256Digest,
    pub classifier_registry_sha256: Sha256Digest,
    pub code: String,
    pub category: FailureCategory,
    pub attribution_targets: BTreeSet<DiagnosticSourceReference>,
    pub evidence_sources: BTreeSet<DiagnosticSourceReference>,
    pub source_families: BTreeSet<SourceFamily>,
    pub confidence: EvidenceConfidence,
    pub claim_strength: DiagnosticClaimStrength,
    pub topology_dependent: bool,
    pub limitations: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticClaimStrength {
    Observational,
    Experimental,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassificationOmission {
    pub code: String,
    pub reason_code: String,
    pub available_sources: BTreeSet<DiagnosticSourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticHypothesis {
    pub id: String,
    pub classification_ids: BTreeSet<String>,
    pub affected_sources: BTreeSet<DiagnosticSourceReference>,
    pub hypothesis_code: String,
    pub rationale: String,
    pub validation_required: bool,
    pub status: HypothesisStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HypothesisStatus {
    Unvalidated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunDiagnosis {
    pub schema_version: SchemaVersion,
    pub run_id: RunId,
    pub run_manifest_sha256: Sha256Digest,
    pub taxonomy_sha256: Sha256Digest,
    pub classifier_registry_sha256: Sha256Digest,
    pub classifications: Vec<FailureClassification>,
    pub omissions: Vec<ClassificationOmission>,
    pub recommendation_hypotheses: Vec<DiagnosticHypothesis>,
    pub limitations: BTreeSet<String>,
}
