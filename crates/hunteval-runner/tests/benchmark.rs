use hunteval_domain::{BenchmarkId, SchemaVersion};
use hunteval_runner::BenchmarkManifest;

#[test]
fn benchmark_matrix_is_exact_cartesian_product_and_repetitions_match()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest = BenchmarkManifest {
        schema_version: SchemaVersion::new(0, 3),
        id: BenchmarkId::new("mvp")?,
        deployments: vec!["d1".into(), "d2".into()],
        episodes: vec!["e1".into(), "e2".into()],
        seeds: vec![1, 2],
        repetitions: Some(2),
        scoring_profile: "profile.yaml".into(),
        fault_profile: None,
    };
    manifest.validate()?;
    assert_eq!(manifest.cells().len(), 8);
    let mut invalid = manifest;
    invalid.repetitions = Some(3);
    assert!(invalid.validate().is_err());
    Ok(())
}
