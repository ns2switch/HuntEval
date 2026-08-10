use hunteval_domain::SchemaVersion;
use hunteval_reporting::{
    JsonRenderer, LegacyDiagnosticFinding, LegacyDiagnosticReport, LegacyDiagnosticValidationStatus,
};

fn report() -> LegacyDiagnosticReport {
    LegacyDiagnosticReport {
        schema_version: SchemaVersion::new(0, 1),
        findings: vec![LegacyDiagnosticFinding {
            classification: "low_event_recall".into(),
            affected_runs: vec!["run-001".into()],
            observable_sources: vec!["result.json#/raw_metrics/event_recall".into()],
            recommendation: "Require an observable coverage check.".into(),
            validation_status: LegacyDiagnosticValidationStatus::Unvalidated,
            validation_source: None,
            human_review_required: true,
        }],
    }
}

#[test]
fn diagnostic_report_retains_sources_and_review_status() -> Result<(), Box<dyn std::error::Error>> {
    let report = report();
    let rendered = JsonRenderer.render_legacy_diagnostic(&report)?;
    assert_eq!(rendered, JsonRenderer.render_legacy_diagnostic(&report)?);
    assert!(String::from_utf8(rendered)?.contains("event_recall"));
    Ok(())
}

#[test]
fn diagnostic_report_rejects_uncited_or_prematurely_validated_claims() {
    let mut uncited = report();
    uncited.findings[0].observable_sources.clear();
    assert!(JsonRenderer.render_legacy_diagnostic(&uncited).is_err());

    let mut validated = report();
    validated.findings[0].validation_status = LegacyDiagnosticValidationStatus::Validated;
    assert!(JsonRenderer.render_legacy_diagnostic(&validated).is_err());
}
