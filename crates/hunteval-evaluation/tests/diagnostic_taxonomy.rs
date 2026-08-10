use hunteval_domain::{DiagnosticTaxonomy, FailureCategory};
use hunteval_evaluation::{canonical_taxonomy, classifier_registry_digest, validate_registry};

#[test]
fn canonical_taxonomy_is_bounded_complete_and_deterministic()
-> Result<(), Box<dyn std::error::Error>> {
    let taxonomy = canonical_taxonomy()?;
    taxonomy.validate()?;
    let categories = taxonomy
        .definitions
        .iter()
        .map(|definition| definition.category)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(categories.len(), 6);
    assert!(categories.contains(&FailureCategory::Investigation));
    assert_eq!(taxonomy.digest()?, canonical_taxonomy()?.digest()?);
    assert_eq!(classifier_registry_digest(), classifier_registry_digest());
    Ok(())
}

#[test]
fn taxonomy_rejects_duplicates_registry_drift_and_executable_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    let taxonomy = canonical_taxonomy()?;
    let mut duplicate = taxonomy.clone();
    duplicate.definitions[1].code = duplicate.definitions[0].code.clone();
    assert!(duplicate.validate().is_err());

    let mut drifted = taxonomy.clone();
    drifted.definitions[0].code = "unknown_rule".into();
    assert!(validate_registry(&drifted).is_err());

    let mut hostile = taxonomy;
    hostile.definitions[0].safe_description = "line\n<script>alert(1)</script>".into();
    assert!(hostile.validate().is_err());

    let malformed = serde_json::to_vec(&serde_json::json!({
        "schema_version": "0.7",
        "taxonomy_version": "1.0",
        "id": "taxonomy",
        "definitions": [],
        "executable_rule": "system('unsafe')"
    }))?;
    assert!(serde_json::from_slice::<DiagnosticTaxonomy>(&malformed).is_err());
    Ok(())
}
