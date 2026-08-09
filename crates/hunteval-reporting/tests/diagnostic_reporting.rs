use hunteval_domain::{
    ContributionClaimStrength, ContributionMetricEffect, ContributionTarget,
    ContributionTargetKind, ControlledContributionAnalysis, DiagnosticApplicability,
    DiagnosticSourceReference, SchemaVersion, Sha256Digest,
};
use hunteval_reporting::{
    DiagnosticArtifactKind, DiagnosticArtifactReference, DiagnosticClaim, DiagnosticClaimStage,
    DiagnosticJsonRenderer, DiagnosticReport, DiagnosticReportScope, DiagnosticStaticHtmlRenderer,
    DiagnosticValidationStatus,
};

fn report(summary: &str) -> DiagnosticReport {
    let source_digest = Sha256Digest::from_bytes(b"run-diagnosis");
    DiagnosticReport {
        schema_version: SchemaVersion::new(0, 7),
        report_id: "diagnostic-report-001".into(),
        scope: DiagnosticReportScope::Run,
        subject_id: "run-001".into(),
        source_manifest_sha256: Sha256Digest::from_bytes(b"manifest"),
        metric_vector_sha256: None,
        scoring_profile_sha256: None,
        claims: vec![DiagnosticClaim {
            id: "claim-001".into(),
            stage: DiagnosticClaimStage::Classification,
            code: "tool_action_failed".into(),
            summary: summary.into(),
            sources: [DiagnosticSourceReference::Artifact {
                path: "run-diagnosis.json".into(),
                artifact_sha256: source_digest,
                pointer: Some("/classifications/0".into()),
            }]
            .into_iter()
            .collect(),
            validation_status: DiagnosticValidationStatus::NotApplicable,
        }],
        artifacts: vec![DiagnosticArtifactReference {
            kind: DiagnosticArtifactKind::RunDiagnosis,
            path: "run-diagnosis.json".into(),
            sha256: source_digest,
        }],
        limitations: ["observational_only".into()].into_iter().collect(),
    }
}

#[test]
fn diagnostic_json_and_html_are_deterministic_and_stage_explicit()
-> Result<(), Box<dyn std::error::Error>> {
    let report = report("The managed action failed.");
    let json = DiagnosticJsonRenderer.render(&report)?;
    assert_eq!(json, DiagnosticJsonRenderer.render(&report)?);
    let html = DiagnosticStaticHtmlRenderer.render(&report)?;
    assert_eq!(html, DiagnosticStaticHtmlRenderer.render(&report)?);
    let html = String::from_utf8(html)?;
    assert!(html.contains("data-stage=\"classification\""));
    assert!(!html.contains("<script"));
    Ok(())
}

#[test]
fn diagnostic_html_escapes_untrusted_text_and_r5_rejects_approved_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let hostile = DiagnosticStaticHtmlRenderer.render(&report(
        "<img src=x onerror=alert(1)><script>alert(2)</script>",
    ))?;
    let hostile = String::from_utf8(hostile)?;
    assert!(!hostile.contains("<img"));
    assert!(!hostile.contains("<script"));
    assert!(hostile.contains("&lt;script&gt;"));

    let mut approved = report("unsupported approval");
    approved.claims[0].stage = DiagnosticClaimStage::ApprovedChange;
    approved.claims[0].validation_status = DiagnosticValidationStatus::Approved;
    assert!(DiagnosticJsonRenderer.render(&approved).is_err());
    Ok(())
}

#[test]
fn controlled_contribution_is_explicitly_experimental_and_topology_dependent()
-> Result<(), Box<dyn std::error::Error>> {
    let mut report = report("baseline classification");
    let source = DiagnosticSourceReference::TopologyExperiment {
        artifact_id: "experiment-001".into(),
        artifact_sha256: Sha256Digest::from_bytes(b"experiment"),
    };
    let analysis = ControlledContributionAnalysis {
        schema_version: SchemaVersion::new(0, 7),
        id: "contribution-001".into(),
        experiment_id: "experiment-001".into(),
        experiment_sha256: Sha256Digest::from_bytes(b"experiment"),
        equivalence_sha256: Sha256Digest::from_bytes(b"equivalence"),
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
        applicability: DiagnosticApplicability::Available,
        reason_code: None,
        experimental: true,
        topology_dependent: true,
        limitations: ["experimental".into(), "topology_dependent".into()]
            .into_iter()
            .collect(),
    };
    let bytes = serde_json::to_vec(&analysis)?;
    report.include_controlled_contribution(
        &analysis,
        "controlled-contribution-analysis.json",
        &bytes,
    )?;
    report.validate()?;
    assert!(report.claims.iter().any(|claim| {
        claim.stage == DiagnosticClaimStage::ExperimentResult
            && claim.validation_status == DiagnosticValidationStatus::Experimental
    }));
    assert!(report.limitations.contains("not_universally_transferable"));
    Ok(())
}
