use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use hunteval_domain::{
    BottleneckAnalysis, DiagnosticApplicability, DiagnosticClaimStrength,
    DiagnosticSourceReference, EvidenceConfidence, HypothesisStatus, RunDiagnosis, SchemaVersion,
    Sha256Digest,
};
use hunteval_evaluation::{canonical_taxonomy, resolve_sources};
use hunteval_reporting::DiagnosticReport;

use super::{DiagnosticBundleArtifact, projection::diagnostic_input, service::metric_names};
use crate::{RunManifest, load_observed_run_for_diagnosis};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn diagnosis_sources_resolve(root: &Path, path: &str, diagnosis: &RunDiagnosis) -> bool {
    let parent = Path::new(path).parent().unwrap_or_else(|| Path::new(""));
    let relative = |name: &str| parent.join(name).to_string_lossy().replace('\\', "/");
    let Some(manifest_bytes) = read(root, &relative("manifest.json"), MAX_MANIFEST_BYTES) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_slice::<RunManifest>(&manifest_bytes) else {
        return false;
    };
    if manifest.run_id != diagnosis.run_id
        || Sha256Digest::from_bytes(&manifest_bytes) != diagnosis.run_manifest_sha256
    {
        return false;
    }
    let (Some(trajectory), Some(submission)) = (
        manifest.hashes.get("trajectory").copied(),
        manifest.hashes.get("submission").copied(),
    ) else {
        return false;
    };
    let run_root = root.join(parent);
    let Ok(observed) = load_observed_run_for_diagnosis(
        &run_root,
        &manifest.run_id,
        trajectory,
        submission,
        128 * 1024,
    ) else {
        return false;
    };
    let Some(metrics) = read(root, &relative("metrics.json"), MAX_ARTIFACT_BYTES) else {
        return false;
    };
    let Ok(names) = metric_names(&metrics) else {
        return false;
    };
    let input = diagnostic_input(
        &observed.observed,
        trajectory,
        Sha256Digest::from_bytes(&manifest_bytes),
        names,
    );
    diagnosis.classifications.iter().all(|classification| {
        resolve_sources(&input.artifacts, &classification.evidence_sources).is_ok()
            && resolve_sources(&input.artifacts, &classification.attribution_targets).is_ok()
    })
}

pub(super) fn valid_run_diagnosis(diagnosis: &RunDiagnosis) -> bool {
    let Ok(taxonomy) = canonical_taxonomy() else {
        return false;
    };
    let mut ids = BTreeSet::new();
    if diagnosis.schema_version != SchemaVersion::new(0, 7)
        || diagnosis.classifications.len() > 256
        || diagnosis.omissions.len() > 256
        || diagnosis.recommendation_hypotheses.len() > 256
    {
        return false;
    }
    for classification in &diagnosis.classifications {
        let Some(definition) = taxonomy.definition(&classification.code) else {
            return false;
        };
        if classification.schema_version != SchemaVersion::new(0, 7)
            || classification.run_id != diagnosis.run_id
            || classification.taxonomy_sha256 != diagnosis.taxonomy_sha256
            || classification.classifier_registry_sha256 != diagnosis.classifier_registry_sha256
            || classification.category != definition.category
            || classification.evidence_sources.is_empty()
            || classification.attribution_targets.is_empty()
            || !ids.insert(classification.id.as_str())
            || classification
                .evidence_sources
                .iter()
                .chain(&classification.attribution_targets)
                .any(|source| !source.has_safe_shape())
            || (classification.claim_strength == DiagnosticClaimStrength::Experimental
                && classification.confidence != EvidenceConfidence::Controlled)
        {
            return false;
        }
    }
    diagnosis
        .recommendation_hypotheses
        .iter()
        .all(|hypothesis| {
            hypothesis.status == HypothesisStatus::Unvalidated
                && hypothesis.validation_required
                && !hypothesis.classification_ids.is_empty()
                && hypothesis
                    .classification_ids
                    .iter()
                    .all(|id| ids.contains(id.as_str()))
                && !hypothesis.affected_sources.is_empty()
        })
}

pub(super) fn valid_bottleneck_analysis(analysis: &BottleneckAnalysis) -> bool {
    analysis.schema_version == SchemaVersion::new(0, 7)
        && !analysis.metrics.is_empty()
        && analysis.metrics.len() <= 128
        && analysis.metrics.iter().all(|metric| {
            metric.version == SchemaVersion::new(0, 7)
                && metric.range.minimum.is_finite()
                && metric.range.maximum.is_none_or(f64::is_finite)
                && match metric.applicability {
                    DiagnosticApplicability::Available => metric.value.is_some_and(|value| {
                        value.is_finite()
                            && value >= metric.range.minimum
                            && metric.range.maximum.is_none_or(|maximum| value <= maximum)
                            && metric.reason_code.is_none()
                            && !metric.sources.is_empty()
                    }),
                    _ => {
                        metric.value.is_none()
                            && metric.numerator.is_none()
                            && metric.denominator.is_none()
                            && metric.reason_code.is_some()
                    }
                }
        })
}

pub(super) fn report_references_match(
    root: &Path,
    report: &DiagnosticReport,
    artifacts: &[DiagnosticBundleArtifact],
    report_path: &str,
) -> bool {
    let inventory: BTreeMap<_, _> = artifacts
        .iter()
        .map(|artifact| (artifact.path.as_str(), artifact.sha256))
        .collect();
    let parent = Path::new(report_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let resolve = |path: &str| parent.join(path).to_string_lossy().replace('\\', "/");
    let has_digest = |digest: Sha256Digest| inventory.values().any(|value| *value == digest);
    has_digest(report.source_manifest_sha256)
        && report.metric_vector_sha256.is_none_or(&has_digest)
        && report.artifacts.iter().all(|artifact| {
            inventory.get(resolve(&artifact.path).as_str()).copied() == Some(artifact.sha256)
        })
        && report.claims.iter().all(|claim| {
            claim.sources.iter().all(|source| match source {
                DiagnosticSourceReference::Artifact {
                    path,
                    artifact_sha256,
                    pointer,
                } => {
                    let resolved = resolve(path);
                    inventory.get(resolved.as_str()).copied() == Some(*artifact_sha256)
                        && pointer.as_deref().is_none_or(|pointer| {
                            read(root, &resolved, MAX_ARTIFACT_BYTES)
                                .and_then(|bytes| {
                                    serde_json::from_slice::<serde_json::Value>(&bytes).ok()
                                })
                                .is_some_and(|value| value.pointer(pointer).is_some())
                        })
                }
                _ => reference_digest(source).is_some_and(&has_digest),
            })
        })
}

fn reference_digest(source: &DiagnosticSourceReference) -> Option<Sha256Digest> {
    match source {
        DiagnosticSourceReference::TrajectoryEvent {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Agent {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Action {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Task {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Evidence {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Finding {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::OperationalMessage {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Metric {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::BenchmarkCell {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::StatisticalComparison {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::Topology {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::TopologyExperiment {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::TopologyEquivalence {
            artifact_sha256, ..
        }
        | DiagnosticSourceReference::TopologyAnalysis {
            artifact_sha256, ..
        } => Some(*artifact_sha256),
        DiagnosticSourceReference::Artifact { .. } => None,
    }
}

fn read(root: &Path, relative: &str, maximum: u64) -> Option<Vec<u8>> {
    if !safe_relative(relative) {
        return None;
    }
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path).ok()?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum {
        return None;
    }
    fs::read(path).ok()
}

fn safe_relative(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|item| matches!(item, Component::Normal(_)))
}
