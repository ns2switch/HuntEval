use hunteval_domain::{
    DiagnosticClaimStrength, DiagnosticSourceReference, EvidenceConfidence, FailureCategory,
    FailureClassification, RunId, SchemaVersion, Sha256Digest, SourceFamily,
};
use hunteval_evaluation::{ComparableDiagnosticCell, reduce_recurrence};

fn classification(run_id: &RunId, cell_digest: Sha256Digest) -> FailureClassification {
    let source = DiagnosticSourceReference::BenchmarkCell {
        cell_id: format!("cell:{cell_digest}"),
        artifact_sha256: cell_digest,
    };
    FailureClassification {
        schema_version: SchemaVersion::new(0, 7),
        id: format!("classification:{cell_digest}"),
        run_id: run_id.clone(),
        taxonomy_sha256: Sha256Digest::from_bytes(b"taxonomy"),
        classifier_registry_sha256: Sha256Digest::from_bytes(b"registry"),
        code: "tool_action_failed".into(),
        category: FailureCategory::ToolUse,
        attribution_targets: [source.clone()].into_iter().collect(),
        evidence_sources: [source].into_iter().collect(),
        source_families: [SourceFamily::Benchmark].into_iter().collect(),
        confidence: EvidenceConfidence::Direct,
        claim_strength: DiagnosticClaimStrength::Observational,
        topology_dependent: false,
        limitations: ["observational_only".into()].into_iter().collect(),
    }
}

#[test]
fn recurrence_preserves_denominators_exclusions_and_descriptive_scope()
-> Result<(), Box<dyn std::error::Error>> {
    let deployment = Sha256Digest::from_bytes(b"deployment");
    let configuration = Sha256Digest::from_bytes(b"configuration");
    let topology = Sha256Digest::from_bytes(b"topology");
    let cells = (0..3)
        .map(|index| {
            let run_id = RunId::new(format!("run-{index}"))?;
            let cell_digest = Sha256Digest::from_bytes(format!("cell-{index}"));
            Ok(ComparableDiagnosticCell {
                cell_id: format!("cell:{cell_digest}"),
                run_id: Some(run_id.clone()),
                deployment_sha256: deployment,
                configuration_sha256: configuration,
                topology_sha256: topology,
                cell_artifact_sha256: cell_digest,
                classifications: (index < 2)
                    .then(|| classification(&run_id, cell_digest))
                    .into_iter()
                    .collect(),
                exclusion_reason: (index == 2).then(|| "cell_failed".into()),
            })
        })
        .collect::<Result<Vec<_>, hunteval_domain::IdValidationError>>()?;
    let groups = reduce_recurrence(&cells)?;
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].eligible_samples, 2);
    assert_eq!(groups[0].occurrences, 2);
    assert_eq!(groups[0].excluded_cells.len(), 1);
    assert!(
        groups[0]
            .limitations
            .contains("recurrence_is_not_causality")
    );
    assert_eq!(groups, reduce_recurrence(&cells)?);
    Ok(())
}

#[test]
fn recurrence_rejects_duplicate_cells() -> Result<(), Box<dyn std::error::Error>> {
    let digest = Sha256Digest::from_bytes(b"same");
    let cell = ComparableDiagnosticCell {
        cell_id: format!("cell:{digest}"),
        run_id: Some(RunId::new("run-1")?),
        deployment_sha256: digest,
        configuration_sha256: digest,
        topology_sha256: digest,
        cell_artifact_sha256: digest,
        classifications: Vec::new(),
        exclusion_reason: None,
    };
    assert!(reduce_recurrence(&[cell.clone(), cell]).is_err());
    Ok(())
}
