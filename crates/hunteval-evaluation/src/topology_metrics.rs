use std::collections::BTreeMap;

use hunteval_domain::{TopologyMetricApplicability, TopologyMetricValue};
use thiserror::Error;

/// Trusted observable counts for a topology-aware run projection.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyMetricInput {
    pub declared_agents: u64,
    pub active_agents: u64,
    pub tasks_created: u64,
    pub delegated_tasks: u64,
    pub evidence_items: u64,
    pub cross_agent_evidence_items: u64,
    pub tool_calls: u64,
    pub duplicate_tool_calls: u64,
    pub operational_messages: u64,
    pub parallel_active_ms: u64,
    pub measured_duration_ms: u64,
    pub investigation_quality: Option<f64>,
    pub verified_cost_utilization: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TopologyMetricError {
    #[error("topology observable count exceeds its denominator")]
    InvalidCount,
    #[error("topology normalized input is invalid")]
    InvalidValue,
}

/// Produces separate observable dimensions without inferring causal contribution.
pub fn evaluate_topology_metrics(
    input: &TopologyMetricInput,
) -> Result<BTreeMap<String, TopologyMetricValue>, TopologyMetricError> {
    validate(input)?;
    let mut metrics = BTreeMap::new();
    metrics.insert(
        "agent_utilization".to_owned(),
        ratio(
            input.active_agents,
            input.declared_agents,
            "no_declared_agents",
        ),
    );
    metrics.insert(
        "task_allocation".to_owned(),
        ratio(
            input.delegated_tasks,
            input.tasks_created,
            "no_tasks_created",
        ),
    );
    metrics.insert(
        "evidence_propagation".to_owned(),
        ratio(
            input.cross_agent_evidence_items,
            input.evidence_items,
            "no_evidence_items",
        ),
    );
    metrics.insert(
        "duplicate_work".to_owned(),
        ratio(
            input.duplicate_tool_calls,
            input.tool_calls,
            "no_tool_calls",
        ),
    );
    metrics.insert(
        "coordination_overhead".to_owned(),
        ratio(
            input.operational_messages,
            input.operational_messages.saturating_add(input.tool_calls),
            "no_observable_operations",
        ),
    );
    metrics.insert(
        "parallelism".to_owned(),
        ratio(
            input.parallel_active_ms,
            input.measured_duration_ms,
            "duration_unavailable",
        ),
    );
    metrics.insert(
        "investigation_quality".to_owned(),
        optional(input.investigation_quality, "scoring_profile_unavailable"),
    );
    metrics.insert(
        "verified_cost_utilization".to_owned(),
        optional(input.verified_cost_utilization, "verified_cost_unavailable"),
    );
    metrics.insert(
        "topology_resilience".to_owned(),
        unavailable("requires_paired_fault_run"),
    );
    metrics.insert(
        "role_contribution".to_owned(),
        unavailable("requires_controlled_ablation"),
    );
    Ok(metrics)
}

fn validate(input: &TopologyMetricInput) -> Result<(), TopologyMetricError> {
    if input.active_agents > input.declared_agents
        || input.delegated_tasks > input.tasks_created
        || input.cross_agent_evidence_items > input.evidence_items
        || input.duplicate_tool_calls > input.tool_calls
        || input.parallel_active_ms > input.measured_duration_ms
    {
        return Err(TopologyMetricError::InvalidCount);
    }
    if [input.investigation_quality, input.verified_cost_utilization]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
    {
        return Err(TopologyMetricError::InvalidValue);
    }
    Ok(())
}

fn ratio(numerator: u64, denominator: u64, reason: &str) -> TopologyMetricValue {
    if denominator == 0 {
        return unavailable(reason);
    }
    TopologyMetricValue {
        applicability: TopologyMetricApplicability::Applicable,
        value: Some(numerator as f64 / denominator as f64),
        reason_code: None,
    }
}

fn optional(value: Option<f64>, reason: &str) -> TopologyMetricValue {
    value.map_or_else(
        || unavailable(reason),
        |value| TopologyMetricValue {
            applicability: TopologyMetricApplicability::Applicable,
            value: Some(value),
            reason_code: None,
        },
    )
}

fn unavailable(reason: &str) -> TopologyMetricValue {
    TopologyMetricValue {
        applicability: TopologyMetricApplicability::Unavailable,
        value: None,
        reason_code: Some(reason.to_owned()),
    }
}
