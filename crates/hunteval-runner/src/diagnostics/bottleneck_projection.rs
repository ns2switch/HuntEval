use std::collections::BTreeSet;

use hunteval_domain::{
    BottleneckInterval, BottleneckIntervalKind, BottleneckObservations, DiagnosticApplicability,
    SchemaVersion, Sha256Digest, TaskState,
};
use hunteval_evaluation::{ObservedRun, ObservedToolOutcome};

pub(super) fn bottleneck_observations(
    observed: &ObservedRun,
    trajectory_sha256: Sha256Digest,
) -> BottleneckObservations {
    let mut intervals = Vec::new();
    for task in observed.tasks.values() {
        let created = observed
            .message_sequences
            .get(&task.created_message_id)
            .copied();
        let started = observed
            .task_transitions
            .iter()
            .find(|item| item.task_id == task.task.id && item.state == TaskState::Started)
            .map(|item| item.sequence);
        let terminal = task
            .terminal_message_id
            .as_ref()
            .and_then(|message| observed.message_sequences.get(message))
            .copied();
        intervals.push(interval(
            BottleneckIntervalKind::TaskQueue,
            task.task.id.as_str(),
            created,
            started,
            observed,
        ));
        intervals.push(interval(
            BottleneckIntervalKind::TaskExecution,
            task.task.id.as_str(),
            started,
            terminal,
            observed,
        ));
    }
    for action in observed.actions.values() {
        let end = observed
            .message_sequences
            .get(&action.result_message_id)
            .copied();
        intervals.push(interval(
            BottleneckIntervalKind::ManagedToolWait,
            action.action_id.as_str(),
            Some(action.request_sequence),
            end,
            observed,
        ));
    }
    let agents: BTreeSet<_> = observed
        .task_transitions
        .iter()
        .map(|item| item.agent_id.to_string())
        .collect();
    for agent in agents {
        intervals.push(unavailable_interval(
            BottleneckIntervalKind::AgentIdle,
            &agent,
            "idle_window_unavailable",
        ));
    }
    BottleneckObservations {
        schema_version: SchemaVersion::new(0, 7),
        run_id: observed.run_id.clone(),
        trajectory_sha256,
        intervals,
        reassignment_count: observed
            .task_transitions
            .iter()
            .filter(|item| item.state == TaskState::Reassigned)
            .count() as u64,
        duplicate_work_count: duplicate_count(observed),
        tool_error_count: observed
            .actions
            .values()
            .filter(|item| item.outcome == ObservedToolOutcome::Error)
            .count() as u64,
        tool_timeout_count: 0,
        limitations: [
            "agent_idle_requires_explicit_schedule_window".into(),
            "tool_timeout_classification_unavailable".into(),
            "supervisor_role_observation_unavailable".into(),
        ]
        .into_iter()
        .collect(),
    }
}

fn interval(
    kind: BottleneckIntervalKind,
    subject: &str,
    start: Option<u64>,
    end: Option<u64>,
    observed: &ObservedRun,
) -> BottleneckInterval {
    let timestamps = start.zip(end).and_then(|(start, end)| {
        Some((
            start,
            end,
            *observed.event_timestamps.get(&start)?,
            *observed.event_timestamps.get(&end)?,
        ))
    });
    let Some((start_sequence, end_sequence, start_time, end_time)) = timestamps else {
        return unavailable_interval(kind, subject, "lifecycle_interval_unavailable");
    };
    let milliseconds =
        (end_time.as_offset_date_time() - start_time.as_offset_date_time()).whole_milliseconds();
    let Ok(duration_ms) = u64::try_from(milliseconds) else {
        return unavailable_interval(kind, subject, "reversed_lifecycle_interval");
    };
    BottleneckInterval {
        kind,
        subject_id: subject.into(),
        start_event_sequence: Some(start_sequence),
        end_event_sequence: Some(end_sequence),
        start_time: Some(start_time),
        end_time: Some(end_time),
        duration_ms: Some(duration_ms),
        applicability: DiagnosticApplicability::Available,
        reason_code: None,
    }
}

fn unavailable_interval(
    kind: BottleneckIntervalKind,
    subject: &str,
    reason: &str,
) -> BottleneckInterval {
    BottleneckInterval {
        kind,
        subject_id: subject.into(),
        start_event_sequence: None,
        end_event_sequence: None,
        start_time: None,
        end_time: None,
        duration_ms: None,
        applicability: DiagnosticApplicability::Unavailable,
        reason_code: Some(reason.into()),
    }
}

fn duplicate_count(observed: &ObservedRun) -> u64 {
    let mut fingerprints = BTreeSet::new();
    observed
        .actions
        .values()
        .filter(|action| {
            let key = serde_json::to_vec(&(action.tool.as_str(), &action.arguments));
            key.is_ok_and(|key| !fingerprints.insert(key))
        })
        .count() as u64
}
