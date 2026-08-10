use std::collections::BTreeSet;

use hunteval_domain::{
    BottleneckAnalysis, BottleneckIntervalKind, BottleneckMetric, BottleneckObservations,
    DiagnosticApplicability, DiagnosticMetricUnit, DiagnosticSourceReference, SchemaVersion,
    Sha256Digest,
};
use thiserror::Error;

use super::bottleneck_metrics::{count, duration, optional_ratio, ratio, unavailable};

pub fn evaluate_bottlenecks(
    observations: &BottleneckObservations,
    task_count: u64,
    agent_count: u64,
    tool_call_count: u64,
    measured_duration_ms: Option<u64>,
) -> Result<BottleneckAnalysis, BottleneckError> {
    validate_observations(observations)?;
    let observations_sha256 = Sha256Digest::from_bytes(
        serde_json::to_vec(observations).map_err(|_| BottleneckError::Serialize)?,
    );
    let source = DiagnosticSourceReference::Artifact {
        path: "bottleneck-observations.json".into(),
        artifact_sha256: observations_sha256,
        pointer: None,
    };
    let sources: BTreeSet<_> = [source].into_iter().collect();
    let queue_ms = interval_total(observations, BottleneckIntervalKind::TaskQueue)?;
    let idle_ms = interval_total(observations, BottleneckIntervalKind::AgentIdle)?;
    let tool_wait_ms = interval_total(observations, BottleneckIntervalKind::ManagedToolWait)?;
    let mut metrics = vec![
        count(
            "reassignment_count",
            observations.reassignment_count,
            sources.clone(),
        ),
        ratio(
            "reassignment_rate",
            observations.reassignment_count,
            task_count,
            "no_tasks",
            sources.clone(),
        ),
        count(
            "duplicate_work_count",
            observations.duplicate_work_count,
            sources.clone(),
        ),
        ratio(
            "duplicate_work_rate",
            observations.duplicate_work_count,
            task_count,
            "no_tasks",
            sources.clone(),
        ),
        duration(
            "task_queue_duration",
            queue_ms,
            "task_queue_unavailable",
            sources.clone(),
        ),
        duration(
            "agent_idle_duration",
            idle_ms,
            "agent_idle_unavailable",
            sources.clone(),
        ),
        duration(
            "managed_tool_wait_duration",
            tool_wait_ms,
            "tool_wait_unavailable",
            sources.clone(),
        ),
        count(
            "managed_tool_error_count",
            observations.tool_error_count,
            sources.clone(),
        ),
        timeout_count(observations, sources.clone()),
        ratio(
            "managed_tool_error_rate",
            observations.tool_error_count,
            tool_call_count,
            "no_tool_calls",
            sources.clone(),
        ),
    ];
    metrics.push(optional_ratio(
        "task_queue_utilization",
        queue_ms,
        measured_duration_ms,
        "duration_unavailable",
        sources.clone(),
    ));
    metrics.push(optional_ratio(
        "managed_tool_wait_utilization",
        tool_wait_ms,
        measured_duration_ms,
        "duration_unavailable",
        sources.clone(),
    ));
    metrics.push(optional_ratio(
        "agent_idle_utilization",
        idle_ms,
        measured_duration_ms.and_then(|duration| duration.checked_mul(agent_count)),
        "duration_or_agents_unavailable",
        sources,
    ));
    metrics.push(unavailable(
        "supervisor_concentration",
        DiagnosticMetricUnit::Ratio,
        "topology_role_observation_unavailable",
        [DiagnosticSourceReference::Artifact {
            path: "bottleneck-observations.json".into(),
            artifact_sha256: observations_sha256,
            pointer: None,
        }]
        .into_iter()
        .collect(),
    ));
    Ok(BottleneckAnalysis {
        schema_version: SchemaVersion::new(0, 7),
        run_id: observations.run_id.clone(),
        observations_sha256,
        metrics,
        limitations: observations.limitations.clone(),
    })
}

fn validate_observations(observations: &BottleneckObservations) -> Result<(), BottleneckError> {
    if observations.schema_version != SchemaVersion::new(0, 7)
        || observations.intervals.len() > 1_000_000
        || observations.reassignment_count > 1_000_000
        || observations.duplicate_work_count > 1_000_000
        || observations.tool_error_count > 1_000_000
        || observations.tool_timeout_count > 1_000_000
        || observations.limitations.len() > 128
        || observations
            .limitations
            .iter()
            .any(|value| !reason_code(value))
    {
        return Err(BottleneckError::InvalidObservations);
    }
    for interval in &observations.intervals {
        if interval.subject_id.is_empty()
            || interval.subject_id.len() > 128
            || interval.subject_id.chars().any(char::is_control)
        {
            return Err(BottleneckError::InvalidInterval);
        }
        match interval.applicability {
            DiagnosticApplicability::Available => {
                let (Some(start), Some(end), Some(start_time), Some(end_time), Some(duration)) = (
                    interval.start_event_sequence,
                    interval.end_event_sequence,
                    interval.start_time,
                    interval.end_time,
                    interval.duration_ms,
                ) else {
                    return Err(BottleneckError::InvalidInterval);
                };
                if start == 0
                    || end > 10_000_000
                    || start > end
                    || start_time > end_time
                    || duration > 86_400_000
                    || interval.reason_code.is_some()
                {
                    return Err(BottleneckError::InvalidInterval);
                }
                let measured = (end_time.as_offset_date_time() - start_time.as_offset_date_time())
                    .whole_milliseconds();
                if measured < 0 || u64::try_from(measured).ok() != Some(duration) {
                    return Err(BottleneckError::InvalidInterval);
                }
            }
            _ if interval.duration_ms.is_some()
                || interval
                    .reason_code
                    .as_deref()
                    .is_none_or(|value| !reason_code(value)) =>
            {
                return Err(BottleneckError::InvalidInterval);
            }
            _ => {}
        }
    }
    Ok(())
}

fn reason_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn interval_total(
    observations: &BottleneckObservations,
    kind: BottleneckIntervalKind,
) -> Result<Option<u64>, BottleneckError> {
    let mut intervals: Vec<_> = observations
        .intervals
        .iter()
        .filter(|item| {
            item.kind == kind && item.applicability == DiagnosticApplicability::Available
        })
        .filter_map(|item| {
            Some((
                item.subject_id.as_str(),
                item.start_time?
                    .as_offset_date_time()
                    .unix_timestamp_nanos(),
                item.end_time?.as_offset_date_time().unix_timestamp_nanos(),
            ))
        })
        .collect();
    if intervals.is_empty() {
        return Ok(None);
    }
    intervals.sort_by(|left, right| {
        left.0
            .cmp(right.0)
            .then(left.1.cmp(&right.1))
            .then(left.2.cmp(&right.2))
    });
    let mut total = 0_u64;
    let mut current = intervals[0];
    for interval in intervals.into_iter().skip(1) {
        if interval.0 == current.0 && interval.1 <= current.2 {
            current.2 = current.2.max(interval.2);
            continue;
        }
        total = add_interval(total, current.1, current.2)?;
        current = interval;
    }
    add_interval(total, current.1, current.2).map(Some)
}

fn add_interval(
    total: u64,
    start_nanoseconds: i128,
    end_nanoseconds: i128,
) -> Result<u64, BottleneckError> {
    let duration = u64::try_from((end_nanoseconds - start_nanoseconds) / 1_000_000)
        .map_err(|_| BottleneckError::InvalidInterval)?;
    total.checked_add(duration).ok_or(BottleneckError::Overflow)
}

fn timeout_count(
    observations: &BottleneckObservations,
    sources: BTreeSet<DiagnosticSourceReference>,
) -> BottleneckMetric {
    if observations
        .limitations
        .contains("tool_timeout_classification_unavailable")
    {
        unavailable(
            "managed_tool_timeout_count",
            DiagnosticMetricUnit::Count,
            "tool_timeout_classification_unavailable",
            sources,
        )
    } else {
        count(
            "managed_tool_timeout_count",
            observations.tool_timeout_count,
            sources,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BottleneckError {
    #[error("bottleneck observations are invalid or oversized")]
    InvalidObservations,
    #[error("bottleneck interval is inconsistent with runner-authoritative data")]
    InvalidInterval,
    #[error("bottleneck duration aggregation overflowed")]
    Overflow,
    #[error("bottleneck observations could not be serialized")]
    Serialize,
}
