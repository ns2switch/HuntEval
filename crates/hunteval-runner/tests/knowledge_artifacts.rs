use hunteval_domain::{SchemaVersion, Sha256Digest};
use hunteval_knowledge::{
    AnalyticalCorpusManifest, AnalyticalSourceKind, CorpusScope, CorpusSource,
};

fn manifest(path: &str, bytes: &[u8]) -> AnalyticalCorpusManifest {
    AnalyticalCorpusManifest {
        schema_version: SchemaVersion::new(0, 9),
        id: "history".to_owned(),
        scope: CorpusScope::EvaluatorAnalytics,
        sources: vec![CorpusSource {
            id: "run-01".to_owned(),
            kind: AnalyticalSourceKind::Run,
            path: path.to_owned(),
            artifact_sha256: Sha256Digest::from_bytes(bytes),
            verified: true,
        }],
    }
}

#[test]
fn loader_verifies_exact_public_bytes_and_rejects_private_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let public = br#"{"run_id":"run-01","finding":"access key"}"#;
    std::fs::write(directory.path().join("run.json"), public)?;
    let loaded =
        hunteval_runner::load_analytical_corpus(directory.path(), manifest("run.json", public))?;
    assert_eq!(loaded.open()?.manifest().source_hashes.len(), 1);

    let private = br#"{"run_id":"run-01","ground_truth":{"answer":"hidden"}}"#;
    std::fs::write(directory.path().join("private.json"), private)?;
    assert!(
        hunteval_runner::load_analytical_corpus(
            directory.path(),
            manifest("private.json", private),
        )
        .is_err()
    );
    std::fs::write(directory.path().join("run.json"), b"changed")?;
    assert!(
        hunteval_runner::load_analytical_corpus(directory.path(), manifest("run.json", public),)
            .is_err()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn loader_rejects_symlinked_path_components() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir()?;
    let actual = directory.path().join("actual");
    std::fs::create_dir(&actual)?;
    let public = br#"{"run_id":"run-01","finding":"access key"}"#;
    std::fs::write(actual.join("run.json"), public)?;
    symlink(&actual, directory.path().join("linked"))?;
    assert!(
        hunteval_runner::load_analytical_corpus(
            directory.path(),
            manifest("linked/run.json", public),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn canonical_index_and_query_fixtures_match_verified_sources()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| std::io::Error::other("workspace root is unavailable"))?;
    let contracts = root.join("examples/contracts/v0.9");
    let manifest = std::fs::read(contracts.join("analytical-corpus-manifest.json"))?;
    let expected_index: hunteval_knowledge::AnalyticalIndexManifest = serde_json::from_slice(
        &std::fs::read(contracts.join("analytical-index-manifest.json"))?,
    )?;
    let expected_result: hunteval_knowledge::AnalyticalResult =
        serde_json::from_slice(&std::fs::read(contracts.join("analytical-result.json"))?)?;
    assert_eq!(
        hunteval_runner::build_analytical_index(root, &manifest)?,
        expected_index
    );
    assert_eq!(
        hunteval_runner::query_analytical_index(
            root,
            &manifest,
            &std::fs::read(contracts.join("analytical-query.json"))?,
        )?,
        expected_result
    );
    Ok(())
}
