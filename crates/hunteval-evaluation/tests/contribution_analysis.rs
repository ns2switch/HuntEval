use hunteval_domain::{
    ContributionClaimStrength, ContributionMetricEffect, ContributionTarget,
    ContributionTargetKind, DiagnosticApplicability, DiagnosticSourceReference, SchemaVersion,
    Sha256Digest,
};
use hunteval_evaluation::{ControlledContributionInput, reduce_controlled_contribution};

fn input(eligible: bool) -> ControlledContributionInput {
    let experiment = Sha256Digest::from_bytes(b"experiment");
    let source = DiagnosticSourceReference::TopologyExperiment {
        artifact_id: "experiment-001".into(),
        artifact_sha256: experiment,
    };
    ControlledContributionInput {
        experiment_id: "experiment-001".into(),
        experiment_sha256: experiment,
        equivalence_sha256: Sha256Digest::from_bytes(b"equivalence"),
        equivalence_eligible: eligible,
        baseline_topology_sha256: Sha256Digest::from_bytes(b"baseline"),
        candidate_topology_sha256: Sha256Digest::from_bytes(b"candidate"),
        target: ContributionTarget {
            kind: ContributionTargetKind::Agent,
            id: "specialist".into(),
        },
        changed_variables: ["/agents/specialist".into()].into_iter().collect(),
        paired_cell_ids: [
            "cell:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            "cell:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        ]
        .into_iter()
        .collect(),
        metric_effects: vec![ContributionMetricEffect {
            metric_name: "event_recall".into(),
            metric_version: SchemaVersion::new(0, 3),
            baseline_value: 0.8,
            candidate_value: 0.6,
            difference: -0.2,
            interval: None,
            claim_strength: ContributionClaimStrength::Exploratory,
            sources: [source].into_iter().collect(),
        }],
        minimum_pairs: 2,
    }
}

#[test]
fn contribution_requires_control_equivalence_and_remains_topology_dependent()
-> Result<(), Box<dyn std::error::Error>> {
    let available = reduce_controlled_contribution(&input(true))?;
    assert_eq!(available.applicability, DiagnosticApplicability::Available);
    assert!(available.experimental && available.topology_dependent);
    assert!(
        available
            .limitations
            .contains("not_universally_transferable")
    );

    let unavailable = reduce_controlled_contribution(&input(false))?;
    assert_eq!(
        unavailable.applicability,
        DiagnosticApplicability::Unavailable
    );
    assert!(unavailable.metric_effects.is_empty());
    assert_eq!(
        unavailable.reason_code.as_deref(),
        Some("control_equivalence_ineligible")
    );
    Ok(())
}

#[test]
fn contribution_rejects_inconsistent_effects_and_invalid_sample_policy() {
    let mut inconsistent = input(true);
    inconsistent.metric_effects[0].difference = 0.7;
    assert!(reduce_controlled_contribution(&inconsistent).is_err());

    let mut no_minimum = input(true);
    no_minimum.minimum_pairs = 0;
    assert!(reduce_controlled_contribution(&no_minimum).is_err());
}
