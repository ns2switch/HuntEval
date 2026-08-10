use hunteval_reporting::{AnalyticalReport, AnalyticalReportMatch};

#[test]
fn analytical_html_is_static_and_escapes_untrusted_fields() -> Result<(), Box<dyn std::error::Error>>
{
    let report = AnalyticalReport {
        schema_version: "0.9".to_owned(),
        query_sha256: "a".repeat(64),
        index_sha256: "b".repeat(64),
        matches: vec![AnalyticalReportMatch {
            source_id: "source-1".to_owned(),
            source_kind: "run".to_owned(),
            artifact_sha256: "c".repeat(64),
            field: "finding".to_owned(),
            excerpt: "<script>alert('unsafe')</script>".to_owned(),
        }],
        limitations: vec!["No causal inference.".to_owned()],
    };
    let html = String::from_utf8(report.render_html()?)?;
    assert!(html.starts_with("<!doctype html>"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("javascript:"));
    Ok(())
}
