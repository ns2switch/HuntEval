use std::{collections::BTreeSet, fmt::Write};

use hunteval_domain::{RecommendationStatusV08, SchemaVersion, Sha256Digest};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImprovementReportStage {
    Observation,
    Classification,
    Attribution,
    Hypothesis,
    SuggestedChange,
    ExperimentalSupport,
    HumanDecision,
    Adoption,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementReportSource {
    pub kind: String,
    pub artifact_sha256: Sha256Digest,
    pub reference_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementReportSection {
    pub id: String,
    pub stage: ImprovementReportStage,
    pub text: String,
    pub sources: Vec<ImprovementReportSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImprovementReport {
    pub schema_version: SchemaVersion,
    pub id: String,
    pub recommendation_id: String,
    pub status: RecommendationStatusV08,
    pub baseline_artifact_sha256: Sha256Digest,
    pub candidate_artifact_sha256: Sha256Digest,
    pub experiment_sha256: Option<Sha256Digest>,
    pub equivalence_sha256: Option<Sha256Digest>,
    pub validation_decision_sha256: Option<Sha256Digest>,
    pub sections: Vec<ImprovementReportSection>,
    pub limitations: BTreeSet<String>,
}

impl ImprovementReport {
    pub fn validate(&self) -> Result<(), ImprovementReportError> {
        if self.schema_version != SchemaVersion::new(0, 8)
            || self.id.is_empty()
            || self.sections.is_empty()
            || self.sections.len() > 128
            || self.limitations.is_empty()
            || self.sections.iter().any(invalid_section)
        {
            return Err(ImprovementReportError::InvalidReport);
        }
        let required_stage = match self.status {
            RecommendationStatusV08::Validated => Some(ImprovementReportStage::ExperimentalSupport),
            RecommendationStatusV08::Approved => Some(ImprovementReportStage::HumanDecision),
            RecommendationStatusV08::Adopted => Some(ImprovementReportStage::Adoption),
            _ => None,
        };
        if required_stage
            .is_some_and(|stage| !self.sections.iter().any(|section| section.stage == stage))
        {
            return Err(ImprovementReportError::StageMismatch);
        }
        if matches!(
            self.status,
            RecommendationStatusV08::Validated
                | RecommendationStatusV08::Approved
                | RecommendationStatusV08::Adopted
        ) && (self.validation_decision_sha256.is_none()
            || self.experiment_sha256.is_none()
            || self.equivalence_sha256.is_none())
        {
            return Err(ImprovementReportError::StageMismatch);
        }
        Ok(())
    }
}

fn invalid_section(section: &ImprovementReportSection) -> bool {
    let lower = section.text.to_ascii_lowercase();
    section.id.is_empty()
        || section.id.len() > 128
        || section.text.trim().is_empty()
        || section.text.len() > 4096
        || section.sources.is_empty()
        || section.sources.len() > 64
        || lower.contains("universally superior")
        || lower.contains("causally proven for all")
        || section.sources.iter().any(|source| {
            source.kind.is_empty()
                || source.kind.len() > 128
                || source.reference_id.is_empty()
                || source.reference_id.len() > 128
        })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ImprovementJsonRenderer;

impl ImprovementJsonRenderer {
    pub fn render(&self, report: &ImprovementReport) -> Result<Vec<u8>, ImprovementReportError> {
        report.validate()?;
        let mut bytes =
            serde_json::to_vec_pretty(report).map_err(|_| ImprovementReportError::Serialization)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ImprovementStaticHtmlRenderer;

impl ImprovementStaticHtmlRenderer {
    pub fn render(&self, report: &ImprovementReport) -> Result<Vec<u8>, ImprovementReportError> {
        report.validate()?;
        let mut sections = String::new();
        for section in &report.sections {
            write!(
                sections,
                "<article data-stage=\"{}\"><h2>{}</h2><p>{}</p><ul>",
                stage_name(section.stage),
                escape(&section.id),
                escape(&section.text)
            )
            .map_err(|_| ImprovementReportError::Serialization)?;
            for source in &section.sources {
                write!(
                    sections,
                    "<li>{}: {} ({})</li>",
                    escape(&source.kind),
                    source.artifact_sha256,
                    escape(&source.reference_id)
                )
                .map_err(|_| ImprovementReportError::Serialization)?;
            }
            sections.push_str("</ul></article>");
        }
        let limitations = report
            .limitations
            .iter()
            .map(|item| format!("<li>{}</li>", escape(item)))
            .collect::<String>();
        let html = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>HuntEval improvement report</title></head><body><h1>{}</h1><p>Status: {}</p><main>{sections}</main><aside><h2>Limitations</h2><ul>{limitations}</ul></aside></body></html>\n",
            escape(&report.id),
            escape(&format!("{:?}", report.status).to_ascii_lowercase())
        );
        Ok(html.into_bytes())
    }
}

const fn stage_name(stage: ImprovementReportStage) -> &'static str {
    match stage {
        ImprovementReportStage::Observation => "observation",
        ImprovementReportStage::Classification => "classification",
        ImprovementReportStage::Attribution => "attribution",
        ImprovementReportStage::Hypothesis => "hypothesis",
        ImprovementReportStage::SuggestedChange => "suggested_change",
        ImprovementReportStage::ExperimentalSupport => "experimental_support",
        ImprovementReportStage::HumanDecision => "human_decision",
        ImprovementReportStage::Adoption => "adoption",
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ImprovementReportError {
    #[error("improvement report is invalid or contains an unsupported claim")]
    InvalidReport,
    #[error("improvement report stage does not match its verified lifecycle state")]
    StageMismatch,
    #[error("improvement report serialization failed")]
    Serialization,
}
