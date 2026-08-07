use hunteval_domain::RunResult;
use hunteval_reporting::{ReportRenderer, RunReport, StaticHtmlRenderer};

#[test]
fn html_escapes_untrusted_metric_names_and_has_no_script() -> Result<(), Box<dyn std::error::Error>>
{
    let mut result: RunResult =
        serde_json::from_str(include_str!("../../../examples/contracts/result.json"))?;
    let metric = result
        .raw_metrics
        .values()
        .next()
        .cloned()
        .ok_or("example result has no metrics")?;
    result
        .raw_metrics
        .insert("<img src=x onerror=alert(1)>".into(), metric);
    let html = String::from_utf8(StaticHtmlRenderer.render_run(&RunReport::from_result(result)?)?)?;
    assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
    assert!(!html.contains("<img"));
    assert!(!html.contains("<script"));
    Ok(())
}
