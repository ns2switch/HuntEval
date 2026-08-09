use hunteval_domain::{SchemaVersion, Sha256Digest, TopologyExperiment};
use hunteval_reporting::{ConstraintFirstStatus, ReportFormat, TopologyComparisonReport};
use hunteval_statistics::StatisticalPolicy;
use thiserror::Error;

use crate::{
    TopologyAblationObservations, TopologyControlError, evaluate_topology_equivalence,
    execute_controlled_topology_ablation,
};

#[derive(Debug, Clone, Copy)]
pub struct ControlledTopologyReportInput<'a> {
    pub experiment: &'a [u8],
    pub baseline_topology: &'a [u8],
    pub candidate_topology: &'a [u8],
    pub statistical_policy: &'a [u8],
    pub scoring_profile: &'a [u8],
    pub observations: &'a [u8],
    pub seed: u64,
    pub format: ReportFormat,
}

/// Reduces verified controlled inputs into a deterministic topology report.
pub fn render_controlled_topology_report(
    input: ControlledTopologyReportInput<'_>,
) -> Result<Vec<u8>, ControlledTopologyReportError> {
    let experiment: TopologyExperiment = parse(input.experiment)?;
    let policy: StatisticalPolicy = parse(input.statistical_policy)?;
    policy
        .validate()
        .map_err(|_| ControlledTopologyReportError::InvalidInput)?;
    let observations: TopologyAblationObservations = parse(input.observations)?;
    let equivalence = evaluate_topology_equivalence(
        &experiment,
        Sha256Digest::from_bytes(input.experiment),
        input.baseline_topology,
        input.candidate_topology,
    )?;
    let ablation = execute_controlled_topology_ablation(
        &experiment,
        &equivalence,
        &policy,
        Sha256Digest::from_bytes(input.statistical_policy),
        &observations,
        input.seed,
    )?;
    let report = TopologyComparisonReport {
        schema_version: SchemaVersion::new(0, 6),
        analysis: ablation.analysis,
        statistical_policy_sha256: Sha256Digest::from_bytes(input.statistical_policy),
        scoring_profile_sha256: Sha256Digest::from_bytes(input.scoring_profile),
        comparisons: ablation.comparisons,
        aggregate_score: None,
        constraint_first_status: ConstraintFirstStatus::Incomparable,
        limitations: vec![
            "experimental_topology_dependent".to_owned(),
            "multiplicity_adjusted_inference_unavailable".to_owned(),
            "raw_metric_vector_authoritative".to_owned(),
            "constraint_first_ranking_requires_declared_constraints".to_owned(),
        ],
    };
    Ok(match input.format {
        ReportFormat::Json => report.render_json()?,
        ReportFormat::Html => report.render_html()?,
    })
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, ControlledTopologyReportError> {
    if bytes.is_empty() || bytes.len() > 64 * 1024 * 1024 {
        return Err(ControlledTopologyReportError::InvalidInput);
    }
    serde_json::from_slice(bytes).map_err(|_| ControlledTopologyReportError::InvalidInput)
}

#[derive(Debug, Error)]
pub enum ControlledTopologyReportError {
    #[error("controlled topology report input is invalid")]
    InvalidInput,
    #[error("controlled topology analysis failed: {0}")]
    Control(#[from] TopologyControlError),
    #[error("controlled topology rendering failed: {0}")]
    Report(#[from] hunteval_reporting::TopologyReportError),
}
