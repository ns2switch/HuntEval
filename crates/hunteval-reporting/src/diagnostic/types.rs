use std::collections::BTreeSet;
use std::path::{Component, Path};

use hunteval_domain::{
    ControlledContributionAnalysis, DiagnosticApplicability, DiagnosticSourceReference,
    SchemaVersion, Sha256Digest,
};
use serde::{Deserialize, Serialize};

use crate::ReportError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticReportScope {
    Run,
    Benchmark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticClaimStage {
    Observation,
    Classification,
    Hypothesis,
    ExperimentResult,
    ApprovedChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticValidationStatus {
    NotApplicable,
    Unvalidated,
    Experimental,
    Approved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticClaim {
    pub id: String,
    pub stage: DiagnosticClaimStage,
    pub code: String,
    pub summary: String,
    pub sources: BTreeSet<DiagnosticSourceReference>,
    pub validation_status: DiagnosticValidationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticArtifactKind {
    RunDiagnosis,
    Recurrence,
    BottleneckObservations,
    BottleneckAnalysis,
    ContributionAnalysis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticArtifactReference {
    pub kind: DiagnosticArtifactKind,
    pub path: String,
    pub sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub schema_version: SchemaVersion,
    pub report_id: String,
    pub scope: DiagnosticReportScope,
    pub subject_id: String,
    pub source_manifest_sha256: Sha256Digest,
    pub metric_vector_sha256: Option<Sha256Digest>,
    pub scoring_profile_sha256: Option<Sha256Digest>,
    pub claims: Vec<DiagnosticClaim>,
    pub artifacts: Vec<DiagnosticArtifactReference>,
    pub limitations: BTreeSet<String>,
}

impl DiagnosticReport {
    pub fn validate(&self) -> Result<(), ReportError> {
        if self.schema_version != SchemaVersion::new(0, 7)
            || self.claims.len() > 1024
            || self.artifacts.len() > 1024
            || self.limitations.is_empty()
            || self.limitations.len() > 128
        {
            return Err(ReportError::InvalidDiagnostic);
        }
        let mut ids = BTreeSet::new();
        for claim in &self.claims {
            if !ids.insert(&claim.id)
                || claim.id.is_empty()
                || claim.code.is_empty()
                || claim.summary.trim().is_empty()
                || claim.summary.len() > 4096
                || claim.sources.is_empty()
                || claim.sources.len() > 128
                || claim.sources.iter().any(|source| !source.has_safe_shape())
                || !valid_stage(claim.stage, claim.validation_status)
            {
                return Err(ReportError::InvalidDiagnostic);
            }
        }
        let mut artifact_paths = BTreeSet::new();
        for artifact in &self.artifacts {
            if !safe_relative(&artifact.path) || !artifact_paths.insert(&artifact.path) {
                return Err(ReportError::InvalidReference);
            }
        }
        Ok(())
    }

    pub fn include_controlled_contribution(
        &mut self,
        analysis: &ControlledContributionAnalysis,
        path: &str,
        bytes: &[u8],
    ) -> Result<(), ReportError> {
        if analysis.validate().is_err()
            || !analysis.experimental
            || !analysis.topology_dependent
            || !safe_relative(path)
            || (analysis.applicability == DiagnosticApplicability::Available
                && (analysis.metric_effects.is_empty() || analysis.reason_code.is_some()))
            || (analysis.applicability != DiagnosticApplicability::Available
                && (!analysis.metric_effects.is_empty() || analysis.reason_code.is_none()))
        {
            return Err(ReportError::InvalidDiagnostic);
        }
        let digest = Sha256Digest::from_bytes(bytes);
        self.artifacts.push(DiagnosticArtifactReference {
            kind: DiagnosticArtifactKind::ContributionAnalysis,
            path: path.into(),
            sha256: digest,
        });
        if analysis.applicability == DiagnosticApplicability::Available {
            for (index, effect) in analysis.metric_effects.iter().enumerate() {
                if effect.sources.is_empty() {
                    return Err(ReportError::InvalidDiagnostic);
                }
                self.claims.push(DiagnosticClaim {
                    id: format!("{}:effect-{index}", analysis.id),
                    stage: DiagnosticClaimStage::ExperimentResult,
                    code: format!("{}_controlled_effect", effect.metric_name),
                    summary: format!(
                        "The controlled topology-dependent experiment observed a metric difference of {} for {}.",
                        effect.difference, effect.metric_name
                    ),
                    sources: effect.sources.clone(),
                    validation_status: DiagnosticValidationStatus::Experimental,
                });
            }
        }
        self.limitations.insert("experimental".into());
        self.limitations.insert("topology_dependent".into());
        self.limitations
            .insert("not_universally_transferable".into());
        Ok(())
    }
}

fn valid_stage(stage: DiagnosticClaimStage, status: DiagnosticValidationStatus) -> bool {
    matches!(
        (stage, status),
        (
            DiagnosticClaimStage::Observation | DiagnosticClaimStage::Classification,
            DiagnosticValidationStatus::NotApplicable
        ) | (
            DiagnosticClaimStage::Hypothesis,
            DiagnosticValidationStatus::Unvalidated
        ) | (
            DiagnosticClaimStage::ExperimentResult,
            DiagnosticValidationStatus::Experimental
        )
    )
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
