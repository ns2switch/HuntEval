use std::collections::BTreeSet;

use hunteval_domain::{
    BenchmarkCellId, ContributionMetricEffect, ContributionTarget, ControlledContributionAnalysis,
    DiagnosticApplicability, SchemaVersion, Sha256Digest,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct ControlledContributionInput {
    pub experiment_id: String,
    pub experiment_sha256: Sha256Digest,
    pub equivalence_sha256: Sha256Digest,
    pub equivalence_eligible: bool,
    pub baseline_topology_sha256: Sha256Digest,
    pub candidate_topology_sha256: Sha256Digest,
    pub target: ContributionTarget,
    pub changed_variables: BTreeSet<String>,
    pub paired_cell_ids: BTreeSet<String>,
    pub metric_effects: Vec<ContributionMetricEffect>,
    pub minimum_pairs: usize,
}

pub fn reduce_controlled_contribution(
    input: &ControlledContributionInput,
) -> Result<ControlledContributionAnalysis, ContributionError> {
    if input.minimum_pairs == 0
        || input.experiment_id.is_empty()
        || input.target.id.is_empty()
        || !bounded_identifier(&input.experiment_id)
        || !bounded_identifier(&input.target.id)
        || input.changed_variables.is_empty()
        || input.changed_variables.len() > 32
        || input.paired_cell_ids.len() > 1_000_000
        || input.metric_effects.len() > 128
        || input
            .paired_cell_ids
            .iter()
            .any(|id| id.parse::<BenchmarkCellId>().is_err())
    {
        return Err(ContributionError::InvalidInput);
    }
    if input.changed_variables.iter().any(|pointer| {
        !pointer.starts_with('/') || pointer.len() > 1024 || pointer.chars().any(char::is_control)
    }) {
        return Err(ContributionError::InvalidInput);
    }
    let available = input.equivalence_eligible
        && input.paired_cell_ids.len() >= input.minimum_pairs
        && !input.metric_effects.is_empty();
    if input.metric_effects.iter().any(|effect| {
        !effect.baseline_value.is_finite()
            || !effect.candidate_value.is_finite()
            || !effect.difference.is_finite()
            || (effect.difference - (effect.candidate_value - effect.baseline_value)).abs()
                > f64::EPSILON * 16.0
            || effect.interval.as_ref().is_some_and(|interval| {
                !interval.lower.is_finite()
                    || !interval.upper.is_finite()
                    || !interval.confidence.is_finite()
                    || interval.lower > interval.upper
                    || !(0.0..=1.0).contains(&interval.confidence)
            })
            || effect.sources.is_empty()
    }) {
        return Err(ContributionError::InvalidEffect);
    }
    let id = contribution_id(input);
    Ok(ControlledContributionAnalysis {
        schema_version: SchemaVersion::new(0, 7),
        id,
        experiment_id: input.experiment_id.clone(),
        experiment_sha256: input.experiment_sha256,
        equivalence_sha256: input.equivalence_sha256,
        baseline_topology_sha256: input.baseline_topology_sha256,
        candidate_topology_sha256: input.candidate_topology_sha256,
        target: input.target.clone(),
        changed_variables: input.changed_variables.clone(),
        paired_cell_ids: input.paired_cell_ids.clone(),
        metric_effects: if available {
            input.metric_effects.clone()
        } else {
            Vec::new()
        },
        applicability: if available {
            DiagnosticApplicability::Available
        } else {
            DiagnosticApplicability::Unavailable
        },
        reason_code: if available {
            None
        } else if !input.equivalence_eligible {
            Some("control_equivalence_ineligible".into())
        } else {
            Some("insufficient_paired_samples".into())
        },
        experimental: true,
        topology_dependent: true,
        limitations: [
            "experimental".to_owned(),
            "topology_dependent".to_owned(),
            "not_universally_transferable".to_owned(),
        ]
        .into_iter()
        .collect(),
    })
}

fn bounded_identifier(value: &str) -> bool {
    value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn contribution_id(input: &ControlledContributionInput) -> String {
    let mut bytes = format!(
        "{}\n{}\n{}\n",
        input.experiment_sha256, input.equivalence_sha256, input.target.id
    )
    .into_bytes();
    for variable in &input.changed_variables {
        bytes.extend_from_slice(variable.as_bytes());
        bytes.push(b'\n');
    }
    format!("contribution:{}", Sha256Digest::from_bytes(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ContributionError {
    #[error("controlled contribution input is malformed or exceeds its bounds")]
    InvalidInput,
    #[error("controlled contribution effect is invalid or uncited")]
    InvalidEffect,
}
