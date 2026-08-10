use hunteval_domain::{
    DiagnosticHypothesis, HypothesisStatus, RunDiagnosis, SchemaVersion, Sha256Digest,
};
use hunteval_reporting::{
    DiagnosticArtifactKind, DiagnosticArtifactReference, DiagnosticClaim, DiagnosticClaimStage,
    DiagnosticReport, DiagnosticReportScope, DiagnosticValidationStatus,
};

use crate::RunManifest;

pub(super) struct ReportInputs<'a> {
    pub manifest: &'a RunManifest,
    pub manifest_bytes: &'a [u8],
    pub metrics_bytes: &'a [u8],
    pub diagnosis: &'a RunDiagnosis,
    pub diagnosis_bytes: &'a [u8],
    pub observation_bytes: &'a [u8],
    pub bottleneck_bytes: &'a [u8],
    pub bottlenecks: &'a hunteval_domain::BottleneckAnalysis,
}

pub(super) fn build_report(input: ReportInputs<'_>) -> DiagnosticReport {
    let mut claims = input
        .diagnosis
        .classifications
        .iter()
        .map(|item| DiagnosticClaim {
            id: item.id.clone(),
            stage: DiagnosticClaimStage::Classification,
            code: item.code.clone(),
            summary: format!("Observed failure classification: {}.", item.code),
            sources: item.evidence_sources.clone(),
            validation_status: DiagnosticValidationStatus::NotApplicable,
        })
        .collect::<Vec<_>>();
    claims.extend(
        input
            .diagnosis
            .recommendation_hypotheses
            .iter()
            .map(|item| DiagnosticClaim {
                id: item.id.clone(),
                stage: DiagnosticClaimStage::Hypothesis,
                code: item.hypothesis_code.clone(),
                summary: item.rationale.clone(),
                sources: item.affected_sources.clone(),
                validation_status: DiagnosticValidationStatus::Unvalidated,
            }),
    );
    let bottleneck_digest = Sha256Digest::from_bytes(input.bottleneck_bytes);
    claims.extend(
        input
            .bottlenecks
            .metrics
            .iter()
            .enumerate()
            .filter_map(|(index, metric)| {
                metric.value.map(|value| DiagnosticClaim {
                    id: format!("bottleneck-claim-{index:04}"),
                    stage: DiagnosticClaimStage::Observation,
                    code: metric.name.clone(),
                    summary: format!(
                        "The runner-authoritative {} measurement is {} {:?}.",
                        metric.name, value, metric.unit
                    ),
                    sources: [hunteval_domain::DiagnosticSourceReference::Artifact {
                        path: "bottleneck-analysis.json".into(),
                        artifact_sha256: bottleneck_digest,
                        pointer: Some(format!("/metrics/{index}")),
                    }]
                    .into_iter()
                    .collect(),
                    validation_status: DiagnosticValidationStatus::NotApplicable,
                })
            }),
    );
    DiagnosticReport {
        schema_version: SchemaVersion::new(0, 7),
        report_id: format!(
            "diagnostic-report:{}",
            Sha256Digest::from_bytes(input.diagnosis_bytes)
        ),
        scope: DiagnosticReportScope::Run,
        subject_id: input.manifest.run_id.to_string(),
        source_manifest_sha256: Sha256Digest::from_bytes(input.manifest_bytes),
        metric_vector_sha256: Some(Sha256Digest::from_bytes(input.metrics_bytes)),
        scoring_profile_sha256: None,
        claims,
        artifacts: vec![
            artifact(
                DiagnosticArtifactKind::RunDiagnosis,
                "run-diagnosis.json",
                input.diagnosis_bytes,
            ),
            artifact(
                DiagnosticArtifactKind::BottleneckObservations,
                "bottleneck-observations.json",
                input.observation_bytes,
            ),
            artifact(
                DiagnosticArtifactKind::BottleneckAnalysis,
                "bottleneck-analysis.json",
                input.bottleneck_bytes,
            ),
        ],
        limitations: input.diagnosis.limitations.clone(),
    }
}

fn artifact(kind: DiagnosticArtifactKind, path: &str, bytes: &[u8]) -> DiagnosticArtifactReference {
    DiagnosticArtifactReference {
        kind,
        path: path.into(),
        sha256: Sha256Digest::from_bytes(bytes),
    }
}

pub(super) fn hypotheses(
    classifications: &[hunteval_domain::FailureClassification],
) -> Vec<DiagnosticHypothesis> {
    classifications
        .iter()
        .map(|classification| DiagnosticHypothesis {
            id: format!(
                "hypothesis:{}",
                Sha256Digest::from_bytes(classification.id.as_bytes())
            ),
            classification_ids: [classification.id.clone()].into_iter().collect(),
            affected_sources: classification.attribution_targets.clone(),
            hypothesis_code: format!("{}_improvement_hypothesis", classification.code),
            rationale: format!(
                "A controlled change may reduce the observable {} pattern; validation is required.",
                classification.code
            ),
            validation_required: true,
            status: HypothesisStatus::Unvalidated,
        })
        .collect()
}
