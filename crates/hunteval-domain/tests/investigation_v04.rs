use std::collections::BTreeSet;

use hunteval_domain::{
    EventId, ExpectedTimelineWindow, FinalSubmissionArtifact, GroundTruth, SchemaVersion,
    SubmissionStatus, TimelineEntry,
};

#[test]
fn v04_investigation_artifacts_adapt_without_exposing_private_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let truth: GroundTruth = serde_json::from_str(include_str!(
        "../../../examples/contracts/v0.4/ground-truth.json"
    ))?;
    truth.validate()?;
    assert_eq!(truth.schema_version, SchemaVersion::new(0, 4));
    assert!(truth.acceptable_submission_statuses.is_some());
    assert!(truth.expected_timeline_windows.is_some());

    let artifact: FinalSubmissionArtifact = serde_json::from_str(include_str!(
        "../../../examples/contracts/v0.4/submission.json"
    ))?;
    let submission = artifact.into_submission()?;
    assert_eq!(submission.timeline.as_deref().map(<[_]>::len), Some(1));
    let public = serde_json::to_string(&submission)?;
    assert!(!public.contains("expected_timeline_windows"));
    assert!(!public.contains("acceptable_submission_statuses"));
    Ok(())
}

#[test]
fn v03_investigation_artifacts_preserve_explicit_absence() -> Result<(), Box<dyn std::error::Error>>
{
    let truth: GroundTruth = serde_json::from_str(include_str!(
        "../../../examples/contracts/ground-truth.json"
    ))?;
    truth.validate()?;
    assert_eq!(truth.acceptable_submission_statuses, None);
    assert_eq!(truth.expected_timeline_windows, None);
    let encoded = serde_json::to_value(truth)?;
    assert!(encoded.get("acceptable_submission_statuses").is_none());
    assert!(encoded.get("expected_timeline_windows").is_none());
    Ok(())
}

#[test]
fn v04_investigation_contracts_reject_invalid_time_duplicates_and_versions()
-> Result<(), Box<dyn std::error::Error>> {
    let event_id = EventId::new("evt-1")?;
    let invalid_window = ExpectedTimelineWindow {
        event_id: event_id.clone(),
        earliest: "2026-08-06T18:00:01Z".parse()?,
        latest: "2026-08-06T18:00:00Z".parse()?,
    };
    assert!(invalid_window.validate().is_err());

    let duplicate = TimelineEntry {
        event_id,
        observed_at: "2026-08-06T18:00:00Z".parse()?,
        summary: "Observed event".to_owned(),
        evidence_ids: BTreeSet::new(),
    };
    let mut artifact: FinalSubmissionArtifact = serde_json::from_str(include_str!(
        "../../../examples/contracts/v0.4/submission.json"
    ))?;
    artifact.submission.timeline = Some(vec![duplicate.clone(), duplicate]);
    assert!(artifact.into_submission().is_err());

    let mut unsupported: FinalSubmissionArtifact = serde_json::from_str(include_str!(
        "../../../examples/contracts/v0.4/submission.json"
    ))?;
    unsupported.schema_version = SchemaVersion::new(0, 5);
    assert!(unsupported.into_submission().is_err());
    assert!(
        serde_json::from_str::<TimelineEntry>(
            r#"{"event_id":"evt-1","observed_at":"not-a-time","summary":"x","evidence_ids":[]}"#
        )
        .is_err()
    );

    let mut truth: GroundTruth = serde_json::from_str(include_str!(
        "../../../examples/contracts/v0.4/ground-truth.json"
    ))?;
    truth.acceptable_submission_statuses = Some(BTreeSet::<SubmissionStatus>::new());
    assert!(truth.validate().is_err());
    Ok(())
}
