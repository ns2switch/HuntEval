use std::collections::BTreeSet;

use hunteval_domain::{
    BottleneckInterval, BottleneckIntervalKind, BottleneckObservations, DiagnosticApplicability,
    RunId, SchemaVersion, Sha256Digest,
};
use hunteval_evaluation::evaluate_bottlenecks;

fn observations() -> Result<BottleneckObservations, Box<dyn std::error::Error>> {
    Ok(BottleneckObservations {
        schema_version: SchemaVersion::new(0, 7),
        run_id: RunId::new("run-bottleneck-001")?,
        trajectory_sha256: Sha256Digest::from_bytes(b"trajectory"),
        intervals: vec![BottleneckInterval {
            kind: BottleneckIntervalKind::TaskQueue,
            subject_id: "task-001".into(),
            start_event_sequence: Some(1),
            end_event_sequence: Some(2),
            start_time: Some("2026-08-09T00:00:00Z".parse()?),
            end_time: Some("2026-08-09T00:00:01Z".parse()?),
            duration_ms: Some(1_000),
            applicability: DiagnosticApplicability::Available,
            reason_code: None,
        }],
        reassignment_count: 1,
        duplicate_work_count: 0,
        tool_error_count: 0,
        tool_timeout_count: 0,
        limitations: BTreeSet::new(),
    })
}

#[test]
fn bottlenecks_keep_counts_durations_and_unavailable_ratios_separate()
-> Result<(), Box<dyn std::error::Error>> {
    let analysis = evaluate_bottlenecks(&observations()?, 2, 1, 0, Some(2_000))?;
    assert_eq!(analysis.metrics.len(), 14);
    let queue = analysis
        .metrics
        .iter()
        .find(|metric| metric.name == "task_queue_duration")
        .ok_or("missing queue metric")?;
    assert_eq!(queue.value, Some(1_000.0));
    let tool_errors = analysis
        .metrics
        .iter()
        .find(|metric| metric.name == "managed_tool_error_rate")
        .ok_or("missing tool metric")?;
    assert_eq!(
        tool_errors.applicability,
        DiagnosticApplicability::Unavailable
    );
    assert_eq!(tool_errors.value, None);
    Ok(())
}

#[test]
fn overlapping_intervals_for_the_same_subject_are_unioned() -> Result<(), Box<dyn std::error::Error>>
{
    let mut input = observations()?;
    input.intervals.push(BottleneckInterval {
        kind: BottleneckIntervalKind::TaskQueue,
        subject_id: "task-001".into(),
        start_event_sequence: Some(2),
        end_event_sequence: Some(3),
        start_time: Some("2026-08-09T00:00:00.500Z".parse()?),
        end_time: Some("2026-08-09T00:00:02Z".parse()?),
        duration_ms: Some(1_500),
        applicability: DiagnosticApplicability::Available,
        reason_code: None,
    });
    let analysis = evaluate_bottlenecks(&input, 2, 1, 0, Some(2_000))?;
    let queue = analysis
        .metrics
        .iter()
        .find(|metric| metric.name == "task_queue_duration")
        .ok_or("missing queue metric")?;
    assert_eq!(queue.value, Some(2_000.0));
    Ok(())
}

#[test]
fn bottlenecks_reject_reversed_or_self_reported_intervals() -> Result<(), Box<dyn std::error::Error>>
{
    let mut invalid = observations()?;
    invalid.intervals[0].duration_ms = Some(999);
    assert!(evaluate_bottlenecks(&invalid, 1, 1, 1, Some(1_000)).is_err());
    Ok(())
}
