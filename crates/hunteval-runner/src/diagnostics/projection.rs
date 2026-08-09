use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{DiagnosticSourceReference, Sha256Digest, TaskState};
use hunteval_evaluation::{
    ClassificationCandidate, DiagnosticArtifactSet, DiagnosticInputV07, ObservedRun,
    ObservedToolOutcome,
};

pub(super) fn diagnostic_input(
    observed: &ObservedRun,
    trajectory_sha256: Sha256Digest,
    manifest_sha256: Sha256Digest,
    metric_names: BTreeSet<String>,
) -> DiagnosticInputV07 {
    let artifacts = artifact_set(observed, trajectory_sha256, manifest_sha256, metric_names);
    DiagnosticInputV07 {
        run_id: observed.run_id.clone(),
        artifacts,
        candidates: candidates(observed, trajectory_sha256, manifest_sha256),
    }
}

fn artifact_set(
    observed: &ObservedRun,
    digest: Sha256Digest,
    manifest_sha256: Sha256Digest,
    metric_names: BTreeSet<String>,
) -> DiagnosticArtifactSet {
    let agent_ids = observed
        .actions
        .values()
        .map(|action| action.agent_id.to_string())
        .chain(
            observed
                .tasks
                .values()
                .map(|task| task.creator_agent_id.to_string()),
        )
        .chain(observed.messages.iter().flat_map(|message| {
            [
                message.agent_id.to_string(),
                message.target_agent_id.to_string(),
            ]
        }))
        .collect();
    DiagnosticArtifactSet {
        run_id: observed.run_id.clone(),
        run_artifact_sha256: digest,
        event_sequences: observed.event_timestamps.keys().copied().collect(),
        agent_ids,
        action_ids: observed.actions.keys().map(ToString::to_string).collect(),
        task_ids: observed.tasks.keys().map(ToString::to_string).collect(),
        evidence_ids: observed.evidence.keys().map(ToString::to_string).collect(),
        finding_ids: observed.findings.keys().map(ToString::to_string).collect(),
        message_ids: observed
            .message_sequences
            .keys()
            .map(ToString::to_string)
            .collect(),
        metric_names,
        public_artifacts: BTreeMap::from([
            ("manifest.json".into(), manifest_sha256),
            ("trajectory.jsonl".into(), digest),
        ]),
        external_digests: BTreeSet::new(),
    }
}

fn candidates(
    observed: &ObservedRun,
    digest: Sha256Digest,
    manifest_sha256: Sha256Digest,
) -> Vec<ClassificationCandidate> {
    let mut candidates = Vec::new();
    for task in observed
        .tasks
        .values()
        .filter(|task| task.state != TaskState::Completed)
    {
        let sequence = task
            .terminal_message_id
            .as_ref()
            .or(Some(&task.created_message_id))
            .and_then(|message| observed.message_sequences.get(message));
        if let Some(sequence) = sequence {
            candidates.push(candidate(
                "task_incomplete",
                task_source(observed, task.task.id.as_str(), digest),
                *sequence,
                observed,
                digest,
            ));
        }
    }
    for action in observed
        .actions
        .values()
        .filter(|action| action.outcome == ObservedToolOutcome::Error)
    {
        if let Some(sequence) = observed.message_sequences.get(&action.result_message_id) {
            let action_source = DiagnosticSourceReference::Action {
                run_id: observed.run_id.clone(),
                entity_id: action.action_id.to_string(),
                artifact_sha256: digest,
            };
            candidates.push(candidate(
                "tool_action_failed",
                action_source,
                *sequence,
                observed,
                digest,
            ));
        }
    }
    candidates.extend(duplicate_task_candidates(observed, digest));
    candidates.extend(unavailable_agent_candidates(observed, digest));
    candidates.extend(ungrounded_finding_candidates(observed, digest));
    candidates.extend(policy_violation_candidates(
        observed,
        digest,
        manifest_sha256,
    ));
    candidates.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then(left.attribution_targets.cmp(&right.attribution_targets))
    });
    candidates
}

fn ungrounded_finding_candidates(
    observed: &ObservedRun,
    digest: Sha256Digest,
) -> Vec<ClassificationCandidate> {
    observed
        .findings
        .values()
        .filter_map(|finding| {
            let evidence = finding
                .finding
                .evidence_ids
                .iter()
                .filter_map(|id| observed.evidence.get(id))
                .collect::<Vec<_>>();
            let grounded_events: BTreeSet<_> = evidence
                .iter()
                .flat_map(|item| item.evidence.event_ids.iter().cloned())
                .collect();
            let grounded_entities: BTreeSet<_> = evidence
                .iter()
                .flat_map(|item| item.evidence.entity_ids.iter().cloned())
                .collect();
            if finding.finding.event_ids.is_subset(&grounded_events)
                && finding.finding.entity_ids.is_subset(&grounded_entities)
            {
                return None;
            }
            let evidence_id = finding.finding.evidence_ids.first()?;
            let finding_source = DiagnosticSourceReference::Finding {
                run_id: observed.run_id.clone(),
                entity_id: finding.finding.id.to_string(),
                artifact_sha256: digest,
            };
            Some(ClassificationCandidate {
                code: "evidence_ungrounded".into(),
                attribution_targets: [finding_source.clone()].into_iter().collect(),
                evidence_sources: [
                    finding_source,
                    DiagnosticSourceReference::Evidence {
                        run_id: observed.run_id.clone(),
                        entity_id: evidence_id.to_string(),
                        artifact_sha256: digest,
                    },
                ]
                .into_iter()
                .collect(),
                controlled_experiment_eligible: false,
                topology_dependent: true,
            })
        })
        .collect()
}

fn policy_violation_candidates(
    observed: &ObservedRun,
    digest: Sha256Digest,
    manifest_sha256: Sha256Digest,
) -> Vec<ClassificationCandidate> {
    const POLICY_REASON_CODES: &[&str] = &[
        "authorization_denied",
        "policy_denied",
        "sql_policy_denied",
        "tool_policy_denied",
    ];
    observed
        .actions
        .values()
        .filter(|action| action.outcome == ObservedToolOutcome::Error)
        .filter(|action| {
            action
                .result
                .get("reason_code")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|code| POLICY_REASON_CODES.binary_search(&code).is_ok())
        })
        .map(|action| {
            let action_source = DiagnosticSourceReference::Action {
                run_id: observed.run_id.clone(),
                entity_id: action.action_id.to_string(),
                artifact_sha256: digest,
            };
            ClassificationCandidate {
                code: "policy_violation_observed".into(),
                attribution_targets: [action_source.clone()].into_iter().collect(),
                evidence_sources: [
                    action_source,
                    DiagnosticSourceReference::Artifact {
                        path: "manifest.json".into(),
                        artifact_sha256: manifest_sha256,
                        pointer: Some("/hashes".into()),
                    },
                ]
                .into_iter()
                .collect(),
                controlled_experiment_eligible: false,
                topology_dependent: true,
            }
        })
        .collect()
}

fn duplicate_task_candidates(
    observed: &ObservedRun,
    digest: Sha256Digest,
) -> Vec<ClassificationCandidate> {
    let mut fingerprints: BTreeMap<Vec<u8>, Vec<_>> = BTreeMap::new();
    for action in observed.actions.values() {
        if let Ok(key) = serde_json::to_vec(&(action.tool.as_str(), &action.arguments)) {
            fingerprints.entry(key).or_default().push(action);
        }
    }
    fingerprints
        .into_values()
        .filter(|actions| actions.len() > 1)
        .flat_map(|actions| actions.into_iter().skip(1))
        .map(|action| {
            candidate(
                "duplicate_task_creation",
                task_source(observed, action.task_id.as_str(), digest),
                action.request_sequence,
                observed,
                digest,
            )
        })
        .collect()
}

fn unavailable_agent_candidates(
    observed: &ObservedRun,
    digest: Sha256Digest,
) -> Vec<ClassificationCandidate> {
    observed
        .task_transitions
        .iter()
        .filter(|transition| transition.state == TaskState::Failed)
        .filter(|failed| {
            observed.task_transitions.iter().any(|transition| {
                transition.task_id == failed.task_id && transition.state == TaskState::Reassigned
            })
        })
        .map(|transition| {
            candidate(
                "agent_unavailable",
                DiagnosticSourceReference::Agent {
                    run_id: observed.run_id.clone(),
                    entity_id: transition.agent_id.to_string(),
                    artifact_sha256: digest,
                },
                transition.sequence,
                observed,
                digest,
            )
        })
        .collect()
}

fn candidate(
    code: &str,
    target: DiagnosticSourceReference,
    sequence: u64,
    observed: &ObservedRun,
    digest: Sha256Digest,
) -> ClassificationCandidate {
    let event = DiagnosticSourceReference::TrajectoryEvent {
        run_id: observed.run_id.clone(),
        event_sequence: sequence,
        artifact_sha256: digest,
    };
    ClassificationCandidate {
        code: code.into(),
        attribution_targets: [target.clone()].into_iter().collect(),
        evidence_sources: [event, target].into_iter().collect(),
        controlled_experiment_eligible: false,
        topology_dependent: true,
    }
}

fn task_source(
    observed: &ObservedRun,
    task_id: &str,
    digest: Sha256Digest,
) -> DiagnosticSourceReference {
    DiagnosticSourceReference::Task {
        run_id: observed.run_id.clone(),
        entity_id: task_id.into(),
        artifact_sha256: digest,
    }
}
