use hunteval_domain::{RunResult, RunStatus};
use hunteval_reporting::{JsonRenderer, ReportRenderer, RunReport};

fn result() -> Result<RunResult, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(include_str!(
        "../../../examples/contracts/result.json"
    ))?)
}

#[test]
fn normalized_json_is_deterministic_and_labels_incomplete_runs()
-> Result<(), Box<dyn std::error::Error>> {
    let mut result = result()?;
    result.status = RunStatus::Incomplete;
    let report = RunReport::from_result(result)?;
    assert_eq!(report.status_label, "incomplete");
    assert_eq!(
        JsonRenderer.render_run(&report)?,
        JsonRenderer.render_run(&report)?
    );
    assert!(report.claims.iter().all(|claim| !claim.source.is_empty()));
    Ok(())
}

#[test]
fn rejects_traversing_artifact_links() -> Result<(), Box<dyn std::error::Error>> {
    let mut report = RunReport::from_result(result()?)?;
    report.artifacts[0].path = "../private.json".into();
    assert!(report.validate().is_err());
    Ok(())
}
