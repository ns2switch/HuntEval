use std::collections::BTreeSet;

use hunteval_domain::{
    DiagnosticClaimStrength, DiagnosticSourceReference, EvidenceConfidence, RunId, SchemaVersion,
    Sha256Digest,
};
use hunteval_evaluation::{
    ClassificationCandidate, DiagnosticArtifactSet, DiagnosticInputV07, classify_verified,
};

fn source_set(
    run_id: &RunId,
    digest: Sha256Digest,
) -> Result<BTreeSet<DiagnosticSourceReference>, Box<dyn std::error::Error>> {
    Ok([
        DiagnosticSourceReference::TrajectoryEvent {
            run_id: run_id.clone(),
            event_sequence: 2,
            artifact_sha256: digest,
        },
        DiagnosticSourceReference::Task {
            run_id: run_id.clone(),
            entity_id: "task-002".into(),
            artifact_sha256: digest,
        },
    ]
    .into_iter()
    .collect())
}

fn artifacts(run_id: &RunId, digest: Sha256Digest) -> DiagnosticArtifactSet {
    DiagnosticArtifactSet {
        run_id: run_id.clone(),
        run_artifact_sha256: digest,
        event_sequences: [2].into_iter().collect(),
        agent_ids: BTreeSet::new(),
        action_ids: BTreeSet::new(),
        task_ids: ["task-002".into()].into_iter().collect(),
        evidence_ids: BTreeSet::new(),
        finding_ids: BTreeSet::new(),
        message_ids: BTreeSet::new(),
        metric_names: BTreeSet::new(),
        public_artifacts: Default::default(),
        external_digests: BTreeSet::new(),
    }
}

#[test]
fn classification_is_exact_deterministic_and_evidence_backed()
-> Result<(), Box<dyn std::error::Error>> {
    let run_id = RunId::new("run-r5-001")?;
    let digest = Sha256Digest::from_bytes(b"verified-run-artifacts");
    let sources = source_set(&run_id, digest)?;
    let input = DiagnosticInputV07 {
        run_id: run_id.clone(),
        artifacts: artifacts(&run_id, digest),
        candidates: vec![ClassificationCandidate {
            code: "duplicate_task_creation".into(),
            attribution_targets: sources.iter().skip(1).cloned().collect(),
            evidence_sources: sources,
            controlled_experiment_eligible: false,
            topology_dependent: true,
        }],
    };
    let (first, omissions) = classify_verified(&input)?;
    let (second, _) = classify_verified(&input)?;
    assert!(omissions.is_empty());
    assert_eq!(first, second);
    assert_eq!(first[0].schema_version, SchemaVersion::new(0, 7));
    assert_eq!(first[0].confidence, EvidenceConfidence::Corroborated);
    assert_eq!(
        first[0].claim_strength,
        DiagnosticClaimStrength::Observational
    );
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn missing_forged_and_repeated_evidence_produce_only_omissions()
-> Result<(), Box<dyn std::error::Error>> {
    let run_id = RunId::new("run-r5-001")?;
    let digest = Sha256Digest::from_bytes(b"verified-run-artifacts");
    let other_digest = Sha256Digest::from_bytes(b"tampered");
    let forged: BTreeSet<_> = [DiagnosticSourceReference::TrajectoryEvent {
        run_id: run_id.clone(),
        event_sequence: 2,
        artifact_sha256: other_digest,
    }]
    .into_iter()
    .collect();
    let input = DiagnosticInputV07 {
        run_id: run_id.clone(),
        artifacts: artifacts(&run_id, digest),
        candidates: vec![ClassificationCandidate {
            code: "duplicate_task_creation".into(),
            attribution_targets: forged.clone(),
            evidence_sources: forged,
            controlled_experiment_eligible: false,
            topology_dependent: true,
        }],
    };
    let (classifications, omissions) = classify_verified(&input)?;
    assert!(classifications.is_empty());
    assert_eq!(omissions.len(), 1);
    assert_eq!(omissions[0].reason_code, "unresolved_evidence_source");
    Ok(())
}

#[test]
fn an_eligibility_flag_without_topology_experiment_evidence_is_not_causal()
-> Result<(), Box<dyn std::error::Error>> {
    let run_id = RunId::new("run-r5-controlled-001")?;
    let digest = Sha256Digest::from_bytes(b"verified-run-artifacts");
    let input = DiagnosticInputV07 {
        run_id: run_id.clone(),
        artifacts: artifacts(&run_id, digest),
        candidates: vec![ClassificationCandidate {
            code: "duplicate_task_creation".into(),
            attribution_targets: source_set(&run_id, digest)?
                .iter()
                .skip(1)
                .cloned()
                .collect(),
            evidence_sources: source_set(&run_id, digest)?,
            controlled_experiment_eligible: true,
            topology_dependent: true,
        }],
    };
    let (classifications, _) = classify_verified(&input)?;
    assert_eq!(
        classifications[0].claim_strength,
        DiagnosticClaimStrength::Observational
    );
    assert_ne!(
        classifications[0].confidence,
        EvidenceConfidence::Controlled
    );
    Ok(())
}

#[test]
fn every_registered_failure_category_has_a_positive_rule_fixture()
-> Result<(), Box<dyn std::error::Error>> {
    let run_id = RunId::new("run-r5-taxonomy-coverage")?;
    let digest = Sha256Digest::from_bytes(b"complete-observable-artifact-set");
    let mut available = artifacts(&run_id, digest);
    available.agent_ids.insert("agent-001".into());
    available.action_ids.insert("action-001".into());
    available.evidence_ids.insert("evidence-001".into());
    available.finding_ids.insert("finding-001".into());
    available
        .public_artifacts
        .insert("manifest.json".into(), digest);
    let event = DiagnosticSourceReference::TrajectoryEvent {
        run_id: run_id.clone(),
        event_sequence: 2,
        artifact_sha256: digest,
    };
    let task = DiagnosticSourceReference::Task {
        run_id: run_id.clone(),
        entity_id: "task-002".into(),
        artifact_sha256: digest,
    };
    let action = DiagnosticSourceReference::Action {
        run_id: run_id.clone(),
        entity_id: "action-001".into(),
        artifact_sha256: digest,
    };
    let candidates = vec![
        covered("task_incomplete", &task, &[task.clone(), event.clone()]),
        covered(
            "evidence_ungrounded",
            &DiagnosticSourceReference::Finding {
                run_id: run_id.clone(),
                entity_id: "finding-001".into(),
                artifact_sha256: digest,
            },
            &[
                DiagnosticSourceReference::Finding {
                    run_id: run_id.clone(),
                    entity_id: "finding-001".into(),
                    artifact_sha256: digest,
                },
                DiagnosticSourceReference::Evidence {
                    run_id: run_id.clone(),
                    entity_id: "evidence-001".into(),
                    artifact_sha256: digest,
                },
            ],
        ),
        covered(
            "tool_action_failed",
            &action,
            &[action.clone(), event.clone()],
        ),
        covered(
            "duplicate_task_creation",
            &task,
            &[task.clone(), event.clone()],
        ),
        covered(
            "agent_unavailable",
            &DiagnosticSourceReference::Agent {
                run_id: run_id.clone(),
                entity_id: "agent-001".into(),
                artifact_sha256: digest,
            },
            &[
                DiagnosticSourceReference::Agent {
                    run_id: run_id.clone(),
                    entity_id: "agent-001".into(),
                    artifact_sha256: digest,
                },
                event,
            ],
        ),
        covered(
            "policy_violation_observed",
            &action,
            &[
                action.clone(),
                DiagnosticSourceReference::Artifact {
                    path: "manifest.json".into(),
                    artifact_sha256: digest,
                    pointer: Some("/hashes".into()),
                },
            ],
        ),
    ];
    let (classifications, omissions) = classify_verified(&DiagnosticInputV07 {
        run_id,
        artifacts: available,
        candidates,
    })?;
    assert!(omissions.is_empty());
    assert_eq!(classifications.len(), 6);
    Ok(())
}

fn covered(
    code: &str,
    target: &DiagnosticSourceReference,
    evidence: &[DiagnosticSourceReference],
) -> ClassificationCandidate {
    ClassificationCandidate {
        code: code.into(),
        attribution_targets: [target.clone()].into_iter().collect(),
        evidence_sources: evidence.iter().cloned().collect(),
        controlled_experiment_eligible: false,
        topology_dependent: true,
    }
}
