use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    ActionId, AgentId, Confidence, EpisodeId, EventId, Evidence, EvidenceId, FinalSubmission,
    Finding, FindingId, FindingSeverity, GroundTruth, MessageId, RunId, SchemaVersion,
    Sha256Digest, SubmissionStatus, TaskId, TaskPriority, TaskSpec, TaskState,
};
use hunteval_evaluation::{
    EvaluationProvenance, ObservedAction, ObservedEvidence, ObservedFinding, ObservedRun,
    ObservedTask, ObservedToolOutcome, TrustedRunInput, TrustedRunView, TrustedViewError,
};

#[test]
fn trusted_view_reduces_deterministically_from_grounded_observations()
-> Result<(), Box<dyn std::error::Error>> {
    let first = TrustedRunView::reduce(valid_input()?)?;
    let second = TrustedRunView::reduce(valid_input()?)?;
    assert_eq!(first.evaluation_input(), second.evaluation_input());
    assert_eq!(first.observed(), second.observed());
    let metrics = first.evaluation_input();
    assert_eq!(metrics.grounded_evidence_items, 1);
    assert_eq!(metrics.valid_provenance_references, 1);
    assert_eq!(metrics.tasks_completed, 1);
    assert_eq!(metrics.tool_calls_used, 1);
    Ok(())
}

#[test]
fn trusted_view_rejects_wrong_owner_and_unissued_events() -> Result<(), Box<dyn std::error::Error>>
{
    let mut wrong_owner = valid_input()?;
    let evidence = wrong_owner
        .observed
        .evidence
        .values_mut()
        .next()
        .ok_or_else(|| std::io::Error::other("evidence missing"))?;
    evidence.agent_id = AgentId::new("other-agent")?;
    assert!(matches!(
        TrustedRunView::reduce(wrong_owner),
        Err(TrustedViewError::WrongAgentOwnership)
    ));

    let mut unissued = valid_input()?;
    let evidence = unissued
        .observed
        .evidence
        .values_mut()
        .next()
        .ok_or_else(|| std::io::Error::other("evidence missing"))?;
    evidence.evidence.event_ids = BTreeSet::from([EventId::new("evt-forged")?]);
    assert!(matches!(
        TrustedRunView::reduce(unissued),
        Err(TrustedViewError::UnissuedEventReference)
    ));
    Ok(())
}

#[test]
fn trusted_view_rejects_cross_run_episode_and_submission_mismatches()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cross_run = valid_input()?;
    cross_run.provenance.run_id = RunId::new("run-other")?;
    assert!(matches!(
        TrustedRunView::reduce(cross_run),
        Err(TrustedViewError::CrossRunReference)
    ));

    let mut wrong_episode = valid_input()?;
    wrong_episode.ground_truth.episode_id = EpisodeId::new("episode-other")?;
    assert!(matches!(
        TrustedRunView::reduce(wrong_episode),
        Err(TrustedViewError::EpisodeMismatch)
    ));

    let mut mismatch = valid_input()?;
    mismatch.terminal_submission.summary = "different stored bytes".to_owned();
    assert!(matches!(
        TrustedRunView::reduce(mismatch),
        Err(TrustedViewError::SubmissionMismatch)
    ));
    Ok(())
}

#[test]
fn trusted_view_deployment_safe_serialization_cannot_include_ground_truth()
-> Result<(), Box<dyn std::error::Error>> {
    let input = valid_input()?;
    assert!(!format!("{input:?}").contains("truth-only-entity"));
    let view = TrustedRunView::reduce(input)?;
    let json = serde_json::to_string(view.observed())?;
    assert!(!json.contains("truth-only-entity"));
    assert!(!json.contains("malicious_event_ids"));
    assert!(!json.contains("ground_truth"));
    assert!(!format!("{view:?}").contains("truth-only-entity"));
    Ok(())
}

fn valid_input() -> Result<TrustedRunInput, Box<dyn std::error::Error>> {
    let run_id = RunId::new("run-trusted")?;
    let episode_id = EpisodeId::new("episode-trusted")?;
    let agent_id = AgentId::new("investigator")?;
    let task_id = TaskId::new("task-1")?;
    let action_id = ActionId::new("action-1")?;
    let event_id = EventId::new("evt-1")?;
    let evidence_id = EvidenceId::new("evidence-1")?;
    let finding_id = FindingId::new("finding-1")?;
    let task = TaskSpec {
        id: task_id.clone(),
        objective: "Investigate the observable".to_owned(),
        priority: TaskPriority::High,
        dependencies: BTreeSet::new(),
        required_capabilities: BTreeSet::new(),
        parent_task_id: None,
    };
    let action = ObservedAction {
        action_id: action_id.clone(),
        agent_id: agent_id.clone(),
        task_id: task_id.clone(),
        request_message_id: MessageId::new("message-action")?,
        result_message_id: MessageId::new("message-result")?,
        tool: "duckdb_sql".to_owned(),
        purpose: "Inspect observable events".to_owned(),
        arguments: serde_json::json!({"query": "SELECT event_id"}),
        outcome: ObservedToolOutcome::Success,
        event_ids: BTreeSet::from([event_id.clone()]),
        result: serde_json::json!({"rows": []}),
    };
    let evidence = Evidence {
        id: evidence_id.clone(),
        summary: "Grounded event".to_owned(),
        source_action_ids: BTreeSet::from([action_id.clone()]),
        event_ids: BTreeSet::from([event_id.clone()]),
        entity_ids: BTreeSet::from(["principal:observed".to_owned()]),
        time_range: None,
        confidence: Confidence::new(0.9)?,
    };
    let finding = Finding {
        id: finding_id.clone(),
        title: "Suspicious identity activity".to_owned(),
        severity: FindingSeverity::High,
        evidence_ids: BTreeSet::from([evidence_id.clone()]),
        event_ids: BTreeSet::from([event_id.clone()]),
        entity_ids: BTreeSet::from(["principal:observed".to_owned()]),
        attack_techniques: BTreeSet::from(["T1078".to_owned()]),
        benign_alternatives: vec!["Authorized activity".to_owned()],
        confidence: Confidence::new(0.9)?,
    };
    let submission = FinalSubmission {
        status: SubmissionStatus::ConfirmedMaliciousActivity,
        summary: "Confirmed activity".to_owned(),
        finding_ids: BTreeSet::from([finding_id.clone()]),
        malicious_event_ids: BTreeSet::from([event_id.clone()]),
        malicious_entity_ids: BTreeSet::from(["principal:observed".to_owned()]),
        attack_path: vec![event_id.clone()],
        attack_techniques: BTreeSet::from(["T1078".to_owned()]),
        confidence: Confidence::new(0.9)?,
        limitations: Vec::new(),
        timeline: None,
    };
    let observed = ObservedRun {
        run_id: run_id.clone(),
        episode_id: episode_id.clone(),
        actions: BTreeMap::from([(action_id, action)]),
        tasks: BTreeMap::from([(
            task_id,
            ObservedTask {
                task,
                creator_agent_id: agent_id.clone(),
                assignee_agent_id: Some(agent_id.clone()),
                state: TaskState::Completed,
                created_message_id: MessageId::new("message-task")?,
                terminal_message_id: Some(MessageId::new("message-task-completed")?),
            },
        )]),
        evidence: BTreeMap::from([(
            evidence_id,
            ObservedEvidence {
                agent_id: agent_id.clone(),
                task_id: TaskId::new("task-1")?,
                message_id: MessageId::new("message-evidence")?,
                evidence,
            },
        )]),
        findings: BTreeMap::from([(
            finding_id,
            ObservedFinding {
                agent_id,
                task_id: TaskId::new("task-1")?,
                message_id: MessageId::new("message-finding")?,
                finding,
            },
        )]),
        messages: Vec::new(),
        timeline: None,
    };
    let ground_truth = GroundTruth {
        schema_version: SchemaVersion::new(0, 3),
        episode_id,
        malicious_event_ids: BTreeSet::from([event_id]),
        malicious_entity_ids: BTreeSet::from(["truth-only-entity".to_owned()]),
        expected_attack_path: Vec::new(),
        expected_attack_techniques: BTreeSet::new(),
        acceptable_conclusions: Vec::new(),
        acceptable_submission_statuses: None,
        expected_timeline_windows: None,
        minimum_evidence_items: 1,
    };
    Ok(TrustedRunInput {
        observed,
        submission: submission.clone(),
        terminal_submission: submission,
        ground_truth,
        provenance: EvaluationProvenance {
            run_id,
            trajectory_sha256: Sha256Digest::from_bytes(b"trajectory"),
            submission_sha256: Sha256Digest::from_bytes(b"submission"),
            ground_truth_sha256: Sha256Digest::from_bytes(b"truth"),
            trajectory_event_count: 10,
        },
        tool_call_limit: 4,
        benign_scored_episode: false,
    })
}
