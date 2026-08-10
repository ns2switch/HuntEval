use hunteval_domain::Sha256Digest;

#[test]
fn audited_queries_append_and_tampering_fails_verification()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let public = br#"{"run_id":"run-01","finding":"access key"}"#;
    std::fs::write(directory.path().join("run.json"), public)?;
    let manifest = serde_json::to_vec(&serde_json::json!({
        "schema_version":"0.9",
        "id":"history",
        "scope":"evaluator_analytics",
        "sources":[{
            "id":"run-01",
            "kind":"run",
            "path":"run.json",
            "artifact_sha256":Sha256Digest::from_bytes(public),
            "verified":true
        }]
    }))?;
    let index = hunteval_runner::build_analytical_index(directory.path(), &manifest)?;
    let query = serde_json::to_vec(&serde_json::json!({
        "schema_version":"0.9",
        "index_sha256":index.index_sha256,
        "scope":"evaluator_analytics",
        "terms":["access"],
        "source_kinds":["run"],
        "max_results":5
    }))?;
    let audit = directory.path().join("retrieval.jsonl");
    for _ in 0..2 {
        hunteval_runner::query_analytical_index_audited(
            directory.path(),
            &manifest,
            &query,
            &audit,
        )?;
    }
    assert_eq!(hunteval_runner::verify_retrieval_audit(&audit)?, 2);

    let mut bytes = std::fs::read(&audit)?;
    if let Some(byte) = bytes.iter_mut().find(|byte| **byte == b'1') {
        *byte = b'2';
    }
    std::fs::write(&audit, bytes)?;
    assert!(hunteval_runner::verify_retrieval_audit(&audit).is_err());
    Ok(())
}
