use std::{collections::BTreeSet, path::PathBuf};

use hunteval_domain::{RecommendationStatusV08, SchemaVersion, Sha256Digest};
use hunteval_reporting::{
    ImprovementReport, ImprovementReportSection, ImprovementReportSource, ImprovementReportStage,
};
use hunteval_runner::{
    ImprovementBundleInput, ImprovementVerificationStatus, generate_improvement_bundle,
    verify_improvement_bundle,
};

fn report() -> ImprovementReport {
    let candidate = Sha256Digest::from_bytes(b"candidate");
    ImprovementReport {
        schema_version: SchemaVersion::new(0, 8),
        id: "bundle-1".into(),
        recommendation_id: "recommendation-1".into(),
        status: RecommendationStatusV08::Proposed,
        baseline_artifact_sha256: Sha256Digest::from_bytes(b"baseline"),
        candidate_artifact_sha256: candidate,
        experiment_sha256: None,
        equivalence_sha256: None,
        validation_decision_sha256: None,
        sections: vec![ImprovementReportSection {
            id: "hypothesis".into(),
            stage: ImprovementReportStage::Hypothesis,
            text: "Observable evidence supports a bounded hypothesis.".into(),
            sources: vec![ImprovementReportSource {
                kind: "diagnostic_source".into(),
                artifact_sha256: Sha256Digest::from_bytes(b"diagnosis"),
                reference_id: "run-1".into(),
            }],
        }],
        limitations: BTreeSet::from(["validation_required".into()]),
    }
}

#[test]
fn bundle_verification_detects_any_changed_byte() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let bundle = temp.path().join("bundle");
    generate_improvement_bundle(
        &bundle,
        &report(),
        &[ImprovementBundleInput {
            kind: "recommendation".into(),
            relative_path: PathBuf::from("artifacts/recommendation.json"),
            bytes: b"{\"status\":\"proposed\"}\n".to_vec(),
        }],
    )?;
    assert_eq!(
        verify_improvement_bundle(&bundle).status,
        ImprovementVerificationStatus::Verified
    );
    std::fs::write(bundle.join("artifacts/recommendation.json"), b"tampered")?;
    assert_eq!(
        verify_improvement_bundle(&bundle).status,
        ImprovementVerificationStatus::Rejected
    );
    Ok(())
}
