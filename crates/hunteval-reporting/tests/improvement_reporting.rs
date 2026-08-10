use std::collections::BTreeSet;

use hunteval_domain::{RecommendationStatusV08, SchemaVersion, Sha256Digest};
use hunteval_reporting::{
    ImprovementJsonRenderer, ImprovementReport, ImprovementReportSection, ImprovementReportSource,
    ImprovementReportStage, ImprovementStaticHtmlRenderer,
};

fn report(text: &str) -> ImprovementReport {
    let candidate = Sha256Digest::from_bytes(b"candidate");
    ImprovementReport {
        schema_version: SchemaVersion::new(0, 8),
        id: "report-1".into(),
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
            text: text.into(),
            sources: vec![ImprovementReportSource {
                kind: "diagnostic_source".into(),
                artifact_sha256: Sha256Digest::from_bytes(b"diagnosis"),
                reference_id: "run-1".into(),
            }],
        }],
        limitations: BTreeSet::from(["controlled_validation_required".into()]),
    }
}

#[test]
fn json_is_authoritative_and_html_escapes_untrusted_text() -> Result<(), Box<dyn std::error::Error>>
{
    let report = report("Candidate <script>alert(1)</script> remains a hypothesis.");
    let json = ImprovementJsonRenderer.render(&report)?;
    let html = ImprovementStaticHtmlRenderer.render(&report)?;
    assert!(String::from_utf8(json)?.contains("<script>"));
    let html = String::from_utf8(html)?;
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    Ok(())
}

#[test]
fn rejects_universal_causal_language() {
    assert!(
        ImprovementJsonRenderer
            .render(&report("This candidate is universally superior."))
            .is_err()
    );
}

#[test]
fn canonical_schema_v08_report_is_a_typed_renderable_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/contracts/v0.8/improvement-report.json");
    let report: ImprovementReport = serde_json::from_slice(&std::fs::read(path)?)?;
    report.validate()?;
    ImprovementJsonRenderer.render(&report)?;
    ImprovementStaticHtmlRenderer.render(&report)?;
    Ok(())
}
