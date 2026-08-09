use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{DiagnosticSourceReference, RunId, Sha256Digest};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticArtifactSet {
    pub run_id: RunId,
    pub run_artifact_sha256: Sha256Digest,
    pub event_sequences: BTreeSet<u64>,
    pub agent_ids: BTreeSet<String>,
    pub action_ids: BTreeSet<String>,
    pub task_ids: BTreeSet<String>,
    pub evidence_ids: BTreeSet<String>,
    pub finding_ids: BTreeSet<String>,
    pub message_ids: BTreeSet<String>,
    pub metric_names: BTreeSet<String>,
    pub public_artifacts: BTreeMap<String, Sha256Digest>,
    pub external_digests: BTreeSet<Sha256Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDiagnosticSource(pub DiagnosticSourceReference);

pub fn resolve_sources(
    artifacts: &DiagnosticArtifactSet,
    sources: &BTreeSet<DiagnosticSourceReference>,
) -> Result<Vec<ResolvedDiagnosticSource>, DiagnosticResolutionError> {
    if sources.len() > 128 {
        return Err(DiagnosticResolutionError::TooManySources);
    }
    sources
        .iter()
        .map(|source| {
            resolve_source(artifacts, source).map(|()| ResolvedDiagnosticSource(source.clone()))
        })
        .collect()
}

fn resolve_source(
    artifacts: &DiagnosticArtifactSet,
    source: &DiagnosticSourceReference,
) -> Result<(), DiagnosticResolutionError> {
    if !source.has_safe_shape() {
        return Err(DiagnosticResolutionError::UnsafeReference);
    }
    match source {
        DiagnosticSourceReference::TrajectoryEvent {
            run_id,
            event_sequence,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.event_sequences.contains(event_sequence))
        }
        DiagnosticSourceReference::Agent {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.agent_ids.contains(entity_id))
        }
        DiagnosticSourceReference::Action {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.action_ids.contains(entity_id))
        }
        DiagnosticSourceReference::Task {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.task_ids.contains(entity_id))
        }
        DiagnosticSourceReference::Evidence {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.evidence_ids.contains(entity_id))
        }
        DiagnosticSourceReference::Finding {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.finding_ids.contains(entity_id))
        }
        DiagnosticSourceReference::OperationalMessage {
            run_id,
            entity_id,
            artifact_sha256,
        } => {
            verify_run(artifacts, run_id, artifact_sha256)?;
            contains(artifacts.message_ids.contains(entity_id))
        }
        DiagnosticSourceReference::Metric {
            run_id,
            metric_name,
            artifact_sha256,
            ..
        } => {
            if let Some(run_id) = run_id {
                verify_run(artifacts, run_id, artifact_sha256)?;
            } else if artifact_sha256 != &artifacts.run_artifact_sha256 {
                return Err(DiagnosticResolutionError::StaleDigest);
            }
            contains(artifacts.metric_names.contains(metric_name))
        }
        DiagnosticSourceReference::Artifact {
            path,
            artifact_sha256,
            ..
        } => match artifacts.public_artifacts.get(path) {
            Some(expected) if expected == artifact_sha256 => Ok(()),
            Some(_) => Err(DiagnosticResolutionError::StaleDigest),
            None => Err(DiagnosticResolutionError::UnknownSource),
        },
        _ => artifact_digest(source)
            .ok_or(DiagnosticResolutionError::UnknownSource)
            .and_then(|digest| contains(artifacts.external_digests.contains(&digest))),
    }
}

fn verify_run(
    artifacts: &DiagnosticArtifactSet,
    run_id: &RunId,
    digest: &Sha256Digest,
) -> Result<(), DiagnosticResolutionError> {
    if run_id != &artifacts.run_id {
        return Err(DiagnosticResolutionError::CrossRun);
    }
    if digest != &artifacts.run_artifact_sha256 {
        return Err(DiagnosticResolutionError::StaleDigest);
    }
    Ok(())
}

fn artifact_digest(source: &DiagnosticSourceReference) -> Option<Sha256Digest> {
    match source {
        DiagnosticSourceReference::BenchmarkCell {
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
        _ => None,
    }
}

fn contains(present: bool) -> Result<(), DiagnosticResolutionError> {
    if present {
        Ok(())
    } else {
        Err(DiagnosticResolutionError::UnknownSource)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DiagnosticResolutionError {
    #[error("diagnostic source set exceeds its bound")]
    TooManySources,
    #[error("diagnostic source has an unsafe shape")]
    UnsafeReference,
    #[error("diagnostic source references another run")]
    CrossRun,
    #[error("diagnostic source digest does not match the verified artifact")]
    StaleDigest,
    #[error("diagnostic source does not exist in the verified artifact set")]
    UnknownSource,
}
