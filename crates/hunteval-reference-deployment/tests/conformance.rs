use hunteval_runner::{ConformanceStatus, run_conformance};

#[test]
fn all_reference_topologies_pass_public_conformance() {
    let executable = std::path::Path::new(env!("CARGO_BIN_EXE_hunteval-reference-deployment"));
    for topology in [
        "single-agent",
        "supervisor-worker",
        "supervisor-specialist",
        "supervisor-specialists",
    ] {
        let result = run_conformance(executable, &["--topology".to_owned(), topology.to_owned()]);
        assert_eq!(
            result.status,
            ConformanceStatus::Conformant,
            "{topology}: {result:?}"
        );
        assert_ne!(result.transcript_sha256, "0".repeat(64));
    }
}

#[test]
fn invalid_deployment_is_a_bounded_nonconformance() {
    let result = run_conformance(std::path::Path::new("missing-deployment"), &[]);
    assert_eq!(result.status, ConformanceStatus::NonConformant);
    assert_eq!(result.checks, ["unsafe_deployment"]);
}
