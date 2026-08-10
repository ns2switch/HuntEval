use std::{fs, io, path::PathBuf};

use hunteval_domain::{
    BottleneckAnalysis, BottleneckObservations, ControlledContributionAnalysis,
    DiagnosticRecurrenceGroup, DiagnosticSourceReference, DiagnosticTaxonomy,
    FailureClassification, RunDiagnosis,
};

fn example(name: &str) -> Result<Vec<u8>, io::Error> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))?
        .to_path_buf();
    fs::read(root.join("examples/contracts/v0.7").join(name))
}

#[test]
fn canonical_diagnosis_contracts_match_typed_domain_models()
-> Result<(), Box<dyn std::error::Error>> {
    let taxonomy: DiagnosticTaxonomy =
        serde_json::from_slice(&example("diagnostic-taxonomy.json")?)?;
    taxonomy.validate()?;
    let source: DiagnosticSourceReference =
        serde_json::from_slice(&example("diagnostic-source-reference.json")?)?;
    assert!(source.has_safe_shape());
    let _: FailureClassification =
        serde_json::from_slice(&example("failure-classification.json")?)?;
    let _: RunDiagnosis = serde_json::from_slice(&example("run-diagnosis.json")?)?;
    let _: DiagnosticRecurrenceGroup =
        serde_json::from_slice(&example("diagnostic-recurrence.json")?)?;
    let _: BottleneckObservations =
        serde_json::from_slice(&example("bottleneck-observations.json")?)?;
    let _: BottleneckAnalysis = serde_json::from_slice(&example("bottleneck-analysis.json")?)?;
    let contribution: ControlledContributionAnalysis =
        serde_json::from_slice(&example("contribution-analysis.json")?)?;
    contribution.validate()?;
    Ok(())
}

#[test]
fn typed_diagnosis_contracts_reject_unknown_and_private_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut classification: serde_json::Value =
        serde_json::from_slice(&example("failure-classification.json")?)?;
    classification["private_ground_truth"] = serde_json::json!({"answer": true});
    assert!(serde_json::from_value::<FailureClassification>(classification).is_err());

    let mut source: serde_json::Value =
        serde_json::from_slice(&example("diagnostic-source-reference.json")?)?;
    source["event_sequence"] = serde_json::json!(0);
    let source: DiagnosticSourceReference = serde_json::from_value(source)?;
    assert!(!source.has_safe_shape());
    Ok(())
}
