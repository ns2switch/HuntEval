use std::{collections::BTreeSet, path::PathBuf};

use hunteval_domain::*;
use hunteval_evaluation::*;
use hunteval_reporting::*;
use hunteval_runner::*;

fn digest(value: impl AsRef<[u8]>) -> Sha256Digest {
    Sha256Digest::from_bytes(value)
}

fn baseline() -> StructuredArtifact {
    let immutable = [
        ImmutableSectionClass::AuthorizationPolicy,
        ImmutableSectionClass::ToolAccessPolicy,
        ImmutableSectionClass::FilesystemPolicy,
        ImmutableSectionClass::NetworkPolicy,
        ImmutableSectionClass::DataHandlingPolicy,
        ImmutableSectionClass::GroundTruthIsolation,
        ImmutableSectionClass::BenchmarkConstraints,
        ImmutableSectionClass::OutputIntegrity,
        ImmutableSectionClass::SecurityControls,
    ];
    let mut sections = immutable
        .into_iter()
        .enumerate()
        .map(|(index, class)| {
            let content = format!("Immutable policy {index}.");
            ArtifactSection {
                id: format!("immutable-{index}"),
                policy: SectionPolicy::Immutable(class),
                sha256: digest(&content),
                content,
            }
        })
        .collect::<Vec<_>>();
    let content = "Delegate investigations by capability.";
    sections.push(ArtifactSection {
        id: "delegation".into(),
        policy: SectionPolicy::Mutable(MutableSectionClass::DelegationStrategy),
        content: content.into(),
        sha256: digest(content),
    });
    StructuredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: "supervisor-structure".into(),
        registered_artifact_sha256: digest("baseline"),
        sections,
    }
}

fn policy() -> ImprovementPolicy {
    ImprovementPolicy {
        schema_version: SchemaVersion::new(0, 8),
        id: "policy".into(),
        immutable_section_classes: BTreeSet::from([
            ImmutableSectionClass::AuthorizationPolicy,
            ImmutableSectionClass::ToolAccessPolicy,
            ImmutableSectionClass::FilesystemPolicy,
            ImmutableSectionClass::NetworkPolicy,
            ImmutableSectionClass::DataHandlingPolicy,
            ImmutableSectionClass::GroundTruthIsolation,
            ImmutableSectionClass::BenchmarkConstraints,
            ImmutableSectionClass::OutputIntegrity,
            ImmutableSectionClass::SecurityControls,
        ]),
        allowed_targets: BTreeSet::from([MutableSectionClass::DelegationStrategy]),
        allowed_operations: BTreeSet::from([
            DiffOperationKind::ReplaceSection,
            DiffOperationKind::AddConstraint,
        ]),
        max_artifact_bytes: 65_536,
        max_growth_percent: 100,
        answer_leakage_check_required: true,
        hidden_test_feedback_during_selection: false,
        human_review_required: true,
        autonomous_adoption: false,
        constraints: vec![
            ImprovementConstraint {
                kind: ConstraintKind::MinimumMetric,
                metric: "event_recall".into(),
                threshold: 0.75,
                required_provenance: RequiredProvenance::None,
            },
            ImprovementConstraint {
                kind: ConstraintKind::MaximumRegression,
                metric: "event_recall".into(),
                threshold: 0.02,
                required_provenance: RequiredProvenance::None,
            },
        ],
    }
}

fn lifecycle_event(
    state: Option<&RecommendationState>,
    status: RecommendationStatusV08,
    candidate: Sha256Digest,
    caused_by: Sha256Digest,
    validation: Option<Sha256Digest>,
    human: Option<Sha256Digest>,
    adoption: Option<Sha256Digest>,
) -> Result<RecommendationEvent, Box<dyn std::error::Error>> {
    Ok(RecommendationEvent {
        schema_version: SchemaVersion::new(0, 8),
        recommendation_id: "recommendation".into(),
        sequence: state.map_or(1, |value| value.last_sequence + 1),
        timestamp: "2026-08-10T12:00:00Z".parse()?,
        previous_event_sha256: state.map(|value| value.last_event_sha256),
        event: status,
        candidate_artifact_sha256: candidate,
        caused_by_artifact_sha256: caused_by,
        reason_code: format!("{:?}", status).to_ascii_lowercase(),
        validation_decision_sha256: validation,
        human_decision_sha256: human,
        adoption_record_sha256: adoption,
    })
}

#[test]
fn controlled_improvement_is_deterministic_auditable_and_human_gated()
-> Result<(), Box<dyn std::error::Error>> {
    let baseline = baseline();
    let run_id: RunId = "run-r6-e2e".parse()?;
    let evidence = PromptDiagnosticEvidence {
        diagnostic_code: "duplicate_task_creation".into(),
        source_families: BTreeSet::from([
            ObservableSourceFamily::Task,
            ObservableSourceFamily::Coordination,
        ]),
        references: vec![DiagnosticSourceReference::Task {
            run_id,
            entity_id: "task-duplicate".into(),
            artifact_sha256: digest("diagnosis"),
        }],
    };
    let recommendation = analyze_prompt_weakness(
        "recommendation",
        "supervisor",
        "supervisor-instruction",
        baseline.registered_artifact_sha256,
        &baseline,
        &evidence,
    )?;
    let materialized = materialize_suggestion(
        &baseline,
        &recommendation,
        MutableSectionClass::DelegationStrategy,
        "Assign exactly one declared owner to every task.",
    )?;
    let diff = structural_diff(
        "diff",
        "supervisor_instruction",
        &baseline,
        &materialized.artifact,
    )?;
    let policy = policy();
    let safety = evaluate_candidate_safety(
        &policy,
        &baseline,
        &materialized.artifact,
        &diff,
        &["private-answer-canary".into()],
    )?;
    assert_eq!(safety.safety_status, SafetyStatus::Passed);
    let diff_sha = digest(serde_json::to_vec(&diff)?);
    let policy_sha = digest(serde_json::to_vec(&policy)?);
    let mut experiment: ImprovementExperiment = serde_json::from_slice(&std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/contracts/v0.8/improvement-experiment.json"),
    )?)?;
    experiment.id = "experiment".into();
    experiment.baseline_artifact_sha256 = baseline.registered_artifact_sha256;
    experiment.candidate_artifact_sha256 = materialized.artifact.registered_artifact_sha256;
    experiment.artifact_diff_sha256 = diff_sha;
    experiment.improvement_policy_sha256 = policy_sha;
    experiment.changed_variable = "supervisor_instruction".into();
    let experiment_bytes = serde_json::to_vec(&experiment)?;
    let experiment_sha = digest(&experiment_bytes);
    let equivalence =
        evaluate_improvement_equivalence(&experiment, experiment_sha, &diff, diff_sha, &safety);
    assert_eq!(equivalence.status, ImprovementEquivalenceStatus::Eligible);
    let equivalence_sha = digest(serde_json::to_vec(&equivalence)?);
    let observations = [PairedMetricObservation {
        pair_id: "pair-1".into(),
        metric: "event_recall".into(),
        version: SchemaVersion::new(0, 3),
        baseline: Some(0.80),
        candidate: Some(0.84),
        provenance: ResourceProvenance::Measured,
    }];
    let decision = decide_candidate(ControlledValidationInput {
        id: "validation",
        experiment: &experiment,
        experiment_sha256: experiment_sha,
        equivalence: &equivalence,
        equivalence_sha256: equivalence_sha,
        policy: &policy,
        policy_sha256: policy_sha,
        observations: &observations,
    })?;
    assert_eq!(decision.status, ValidationStatus::Passed);
    let decision_sha = digest(serde_json::to_vec(&decision)?);

    let recommendation_sha = digest(serde_json::to_vec(&recommendation)?);
    let mut events = Vec::new();
    for status in [
        RecommendationStatusV08::Proposed,
        RecommendationStatusV08::Testing,
        RecommendationStatusV08::Validated,
    ] {
        let state = (!events.is_empty())
            .then(|| project_recommendation(&events))
            .transpose()?;
        events.push(lifecycle_event(
            state.as_ref(),
            status,
            materialized.artifact.registered_artifact_sha256,
            if status == RecommendationStatusV08::Validated {
                decision_sha
            } else {
                recommendation_sha
            },
            (status == RecommendationStatusV08::Validated).then_some(decision_sha),
            None,
            None,
        )?);
    }
    let human = HumanDecision {
        schema_version: SchemaVersion::new(0, 8),
        id: "review".into(),
        recommendation_id: "recommendation".into(),
        candidate_artifact_sha256: materialized.artifact.registered_artifact_sha256,
        experiment_sha256: experiment_sha,
        validation_decision_sha256: decision_sha,
        improvement_policy_sha256: policy_sha,
        reviewer_id: "maintainer".into(),
        reviewed_at: "2026-08-10T13:00:00Z".parse()?,
        decision: ReviewDecision::Approve,
        reason_codes: BTreeSet::from(["controlled_validation_reviewed".into()]),
        explicit_confirmation: true,
    };
    let human_sha = digest(serde_json::to_vec(&human)?);
    verify_human_decision(&human, human_sha, &decision, decision_sha)?;
    let approved = project_recommendation(&events)?;
    events.push(lifecycle_event(
        Some(&approved),
        RecommendationStatusV08::Approved,
        materialized.artifact.registered_artifact_sha256,
        human_sha,
        Some(decision_sha),
        Some(human_sha),
        None,
    )?);
    let adoption = AdoptionRecord {
        schema_version: SchemaVersion::new(0, 8),
        id: "adoption".into(),
        recommendation_id: "recommendation".into(),
        candidate_artifact_sha256: materialized.artifact.registered_artifact_sha256,
        human_decision_sha256: human_sha,
        adopted_deployment_sha256: digest("external-deployment"),
        actor_id: "maintainer".into(),
        adopted_at: "2026-08-10T14:00:00Z".parse()?,
        external_adoption_confirmed: true,
    };
    verify_external_adoption(&adoption, &human, human_sha)?;
    let adoption_sha = digest(serde_json::to_vec(&adoption)?);
    let approved = project_recommendation(&events)?;
    events.push(lifecycle_event(
        Some(&approved),
        RecommendationStatusV08::Adopted,
        materialized.artifact.registered_artifact_sha256,
        adoption_sha,
        Some(decision_sha),
        Some(human_sha),
        Some(adoption_sha),
    )?);
    let state = project_recommendation(&events)?;
    assert_eq!(state.status, RecommendationStatusV08::Adopted);

    let source = |kind: &str, sha256, reference: &str| ImprovementReportSource {
        kind: kind.into(),
        artifact_sha256: sha256,
        reference_id: reference.into(),
    };
    let report = ImprovementReport {
        schema_version: SchemaVersion::new(0, 8),
        id: "improvement-e2e".into(),
        recommendation_id: "recommendation".into(),
        status: state.status,
        baseline_artifact_sha256: baseline.registered_artifact_sha256,
        candidate_artifact_sha256: materialized.artifact.registered_artifact_sha256,
        experiment_sha256: Some(experiment_sha),
        equivalence_sha256: Some(equivalence_sha),
        validation_decision_sha256: Some(decision_sha),
        sections: vec![
            ImprovementReportSection {
                id: "observation".into(),
                stage: ImprovementReportStage::Observation,
                text: "Duplicate task creation was observed.".into(),
                sources: vec![source(
                    "diagnostic_source",
                    digest("diagnosis"),
                    "run-r6-e2e",
                )],
            },
            ImprovementReportSection {
                id: "support".into(),
                stage: ImprovementReportStage::ExperimentalSupport,
                text: "The exact candidate passed the declared controlled validation.".into(),
                sources: vec![source("validation_decision", decision_sha, "validation")],
            },
            ImprovementReportSection {
                id: "review".into(),
                stage: ImprovementReportStage::HumanDecision,
                text: "A human explicitly approved the exact validated candidate.".into(),
                sources: vec![source("human_decision", human_sha, "review")],
            },
            ImprovementReportSection {
                id: "adoption".into(),
                stage: ImprovementReportStage::Adoption,
                text: "An external adoption was explicitly confirmed.".into(),
                sources: vec![source("adoption_record", adoption_sha, "adoption")],
            },
        ],
        limitations: BTreeSet::from(["experimental_topology_dependent".into()]),
    };
    let inputs = vec![ImprovementBundleInput {
        kind: "validation_decision".into(),
        relative_path: PathBuf::from("artifacts/validation.json"),
        bytes: serde_json::to_vec_pretty(&decision)?,
    }];
    let temp = tempfile::tempdir()?;
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    generate_improvement_bundle(&first, &report, &inputs)?;
    generate_improvement_bundle(&second, &report, &inputs)?;
    assert_eq!(
        verify_improvement_bundle(&first).status,
        ImprovementVerificationStatus::Verified
    );
    for path in [
        "improvement-report.json",
        "improvement-report.html",
        "improvement-bundle-manifest.json",
    ] {
        assert_eq!(
            std::fs::read(first.join(path))?,
            std::fs::read(second.join(path))?
        );
    }
    Ok(())
}
