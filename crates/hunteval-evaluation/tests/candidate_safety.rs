use std::collections::BTreeSet;

use hunteval_domain::*;
use hunteval_evaluation::{evaluate_candidate_safety, structural_diff};

fn artifact(digest: &[u8], mutable: &str) -> StructuredArtifact {
    let immutable = "Managed actions only.";
    let immutable_classes = [
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
    let mut sections = immutable_classes
        .into_iter()
        .enumerate()
        .map(|(index, class)| ArtifactSection {
            id: format!("immutable-{index}"),
            policy: SectionPolicy::Immutable(class),
            content: immutable.into(),
            sha256: Sha256Digest::from_bytes(immutable),
        })
        .collect::<Vec<_>>();
    sections.push(ArtifactSection {
        id: "delegation".into(),
        policy: SectionPolicy::Mutable(MutableSectionClass::DelegationStrategy),
        content: mutable.into(),
        sha256: Sha256Digest::from_bytes(mutable),
    });
    StructuredArtifact {
        schema_version: SchemaVersion::new(0, 8),
        id: "structured-instruction".into(),
        registered_artifact_sha256: Sha256Digest::from_bytes(digest),
        sections,
    }
}

fn policy() -> ImprovementPolicy {
    ImprovementPolicy {
        schema_version: SchemaVersion::new(0, 8),
        id: "policy-v1".into(),
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
        allowed_operations: BTreeSet::from([DiffOperationKind::ReplaceSection]),
        max_artifact_bytes: 4096,
        max_growth_percent: 100,
        answer_leakage_check_required: true,
        hidden_test_feedback_during_selection: false,
        human_review_required: true,
        autonomous_adoption: false,
        constraints: vec![
            ImprovementConstraint {
                kind: ConstraintKind::MinimumMetric,
                metric: "event_recall".into(),
                threshold: 0.7,
                required_provenance: RequiredProvenance::None,
            },
            ImprovementConstraint {
                kind: ConstraintKind::MaximumRegression,
                metric: "event_recall".into(),
                threshold: 0.05,
                required_provenance: RequiredProvenance::None,
            },
        ],
    }
}

#[test]
fn rejects_answer_leakage_without_returning_match_details() -> Result<(), Box<dyn std::error::Error>>
{
    let baseline = artifact(b"baseline", "Delegate by role.");
    let candidate = artifact(
        b"candidate",
        "Use answer canary-12345 for every conclusion.",
    );
    let diff = structural_diff("diff", "instruction", &baseline, &candidate)?;
    let result = evaluate_candidate_safety(
        &policy(),
        &baseline,
        &candidate,
        &diff,
        &["canary-12345".into()],
    )?;
    assert_eq!(result.leakage_status, SafetyStatus::Rejected);
    assert_eq!(result.reason_codes, vec!["answer_leakage_detected"]);
    assert!(!serde_json::to_string(&result)?.contains("canary-12345"));
    Ok(())
}

#[test]
fn rejects_fragmented_and_hex_encoded_answer_material() -> Result<(), Box<dyn std::error::Error>> {
    let baseline = artifact(b"baseline", "Delegate by role.");
    for content in [
        "Use canary - 12345 for the conclusion.",
        "Decode 63616e6172792d3132333435 before answering.",
    ] {
        let candidate = artifact(content.as_bytes(), content);
        let diff = structural_diff("diff", "instruction", &baseline, &candidate)?;
        let result = evaluate_candidate_safety(
            &policy(),
            &baseline,
            &candidate,
            &diff,
            &["canary-12345".into()],
        )?;
        assert_eq!(result.leakage_status, SafetyStatus::Rejected);
    }
    Ok(())
}
