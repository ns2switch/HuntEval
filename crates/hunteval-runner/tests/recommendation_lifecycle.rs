use hunteval_domain::{
    RecommendationEvent, RecommendationStatusV08, SchemaVersion, Sha256Digest, UtcTimestamp,
};
use hunteval_runner::{RecommendationJournal, RecommendationJournalError};

fn event(
    sequence: u64,
    previous: Option<Sha256Digest>,
    status: RecommendationStatusV08,
    validation: Option<Sha256Digest>,
) -> Result<RecommendationEvent, Box<dyn std::error::Error>> {
    Ok(RecommendationEvent {
        schema_version: SchemaVersion::new(0, 8),
        recommendation_id: "recommendation".into(),
        sequence,
        timestamp: "2026-08-10T12:00:00Z".parse::<UtcTimestamp>()?,
        previous_event_sha256: previous,
        event: status,
        candidate_artifact_sha256: Sha256Digest::from_bytes(b"candidate"),
        caused_by_artifact_sha256: Sha256Digest::from_bytes(format!("cause-{sequence}")),
        reason_code: format!("event_{sequence}"),
        validation_decision_sha256: validation,
        human_decision_sha256: None,
        adoption_record_sha256: None,
    })
}

#[test]
fn append_only_journal_replays_and_detects_skipped_transitions()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let mut journal = RecommendationJournal::open(temp.path())?;
    let proposed = event(1, None, RecommendationStatusV08::Proposed, None)?;
    let state = journal.append(proposed)?;
    assert_eq!(state.status, RecommendationStatusV08::Proposed);
    let (sequence, previous) = journal.next_link();
    let invalid = event(
        sequence,
        previous,
        RecommendationStatusV08::Validated,
        Some(Sha256Digest::from_bytes(b"decision")),
    )?;
    assert_eq!(
        journal.append(invalid),
        Err(RecommendationJournalError::InvalidTransition)
    );
    drop(journal);
    let reopened = RecommendationJournal::open(temp.path())?;
    assert_eq!(
        reopened.state()?.ok_or("missing state")?.status,
        RecommendationStatusV08::Proposed
    );
    Ok(())
}
