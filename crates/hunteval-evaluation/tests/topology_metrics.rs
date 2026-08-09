use hunteval_domain::TopologyMetricApplicability;
use hunteval_evaluation::{TopologyMetricInput, evaluate_topology_metrics};

#[test]
fn topology_metrics_keep_overhead_resources_and_quality_separate()
-> Result<(), Box<dyn std::error::Error>> {
    let metrics = evaluate_topology_metrics(&TopologyMetricInput {
        declared_agents: 4,
        active_agents: 3,
        tasks_created: 8,
        delegated_tasks: 6,
        evidence_items: 5,
        cross_agent_evidence_items: 4,
        tool_calls: 10,
        duplicate_tool_calls: 2,
        operational_messages: 10,
        parallel_active_ms: 25,
        measured_duration_ms: 100,
        investigation_quality: Some(0.9),
        verified_cost_utilization: Some(0.4),
    })?;
    assert_eq!(metrics["agent_utilization"].value, Some(0.75));
    assert_eq!(metrics["coordination_overhead"].value, Some(0.5));
    assert_eq!(metrics["investigation_quality"].value, Some(0.9));
    assert_eq!(metrics["verified_cost_utilization"].value, Some(0.4));
    assert_eq!(
        metrics["role_contribution"].applicability,
        TopologyMetricApplicability::Unavailable
    );
    Ok(())
}

#[test]
fn unsupported_and_zero_denominator_metrics_remain_unavailable()
-> Result<(), Box<dyn std::error::Error>> {
    let metrics = evaluate_topology_metrics(&TopologyMetricInput {
        declared_agents: 1,
        active_agents: 1,
        tasks_created: 0,
        delegated_tasks: 0,
        evidence_items: 0,
        cross_agent_evidence_items: 0,
        tool_calls: 0,
        duplicate_tool_calls: 0,
        operational_messages: 0,
        parallel_active_ms: 0,
        measured_duration_ms: 0,
        investigation_quality: None,
        verified_cost_utilization: None,
    })?;
    for name in [
        "task_allocation",
        "evidence_propagation",
        "duplicate_work",
        "parallelism",
        "investigation_quality",
        "verified_cost_utilization",
        "topology_resilience",
        "role_contribution",
    ] {
        assert_eq!(
            metrics[name].applicability,
            TopologyMetricApplicability::Unavailable
        );
        assert_eq!(metrics[name].value, None);
    }
    Ok(())
}

#[test]
fn malformed_observable_counts_fail_closed() {
    let result = evaluate_topology_metrics(&TopologyMetricInput {
        declared_agents: 1,
        active_agents: 2,
        tasks_created: 0,
        delegated_tasks: 0,
        evidence_items: 0,
        cross_agent_evidence_items: 0,
        tool_calls: 0,
        duplicate_tool_calls: 0,
        operational_messages: 0,
        parallel_active_ms: 0,
        measured_duration_ms: 0,
        investigation_quality: None,
        verified_cost_utilization: None,
    });
    assert!(result.is_err());
}
