use std::collections::{BTreeMap, BTreeSet};

use hunteval_domain::{
    DeploymentArchitecture, DeploymentRegistration, DeploymentTopology, EquivalenceStatus,
    Sha256Digest, TopologyAnalysis, TopologyAnalysisKind, TopologyEquivalenceResult,
    TopologyExperiment, TopologyKind, TopologyMetricValue,
};
use hunteval_statistics::PolicyComparisonError;
use serde_json::Value;
use thiserror::Error;

const RESULT_SCHEMA_VERSION: hunteval_domain::SchemaVersion =
    hunteval_domain::SchemaVersion::new(0, 6);

/// Fail-closed errors while evaluating a controlled topology experiment.
#[derive(Debug, Clone, Copy, Error)]
pub enum TopologyControlError {
    #[error("topology experiment is invalid")]
    InvalidExperiment,
    #[error("baseline topology is invalid")]
    InvalidBaseline,
    #[error("candidate topology is invalid")]
    InvalidCandidate,
    #[error("topology serialization failed")]
    Serialization,
    #[error("topology experiment is not eligible for controlled analysis")]
    IneligibleExperiment,
    #[error("controlled ablation statistical input is invalid")]
    Statistical(#[from] PolicyComparisonError),
    #[error("controlled ablation observations are incomplete or unbounded")]
    InvalidObservations,
}

/// Creates an explicitly experimental, topology-dependent analysis after equivalence passes.
pub fn build_controlled_topology_analysis(
    experiment: &TopologyExperiment,
    equivalence: &TopologyEquivalenceResult,
    metrics: BTreeMap<String, TopologyMetricValue>,
) -> Result<TopologyAnalysis, TopologyControlError> {
    experiment
        .validate()
        .map_err(|_| TopologyControlError::InvalidExperiment)?;
    equivalence
        .validate()
        .map_err(|_| TopologyControlError::IneligibleExperiment)?;
    if equivalence.status != EquivalenceStatus::Eligible || metrics.is_empty() {
        return Err(TopologyControlError::IneligibleExperiment);
    }
    let analysis = TopologyAnalysis {
        schema_version: RESULT_SCHEMA_VERSION,
        baseline_topology_sha256: experiment.baseline_topology_sha256,
        candidate_topology_sha256: experiment.candidate_topology_sha256,
        experiment_sha256: Some(equivalence.experiment_sha256),
        analysis_kind: TopologyAnalysisKind::ControlledAblation,
        topology_dependent: true,
        metrics,
        limitations: BTreeSet::from(["experimental_topology_dependent".to_owned()]),
    };
    analysis
        .validate()
        .map_err(|_| TopologyControlError::IneligibleExperiment)?;
    Ok(analysis)
}

/// Checks operational protocol registration against the normative authored topology.
pub fn registration_conforms_to_topology(
    registration: &DeploymentRegistration,
    topology: &DeploymentTopology,
) -> bool {
    if architecture_kind(registration.architecture) != Some(topology.kind) {
        return false;
    }
    let registered = registration
        .agents
        .iter()
        .map(|agent| (&agent.id, agent.role.as_str()))
        .collect::<BTreeMap<_, _>>();
    let declared = topology
        .agents
        .iter()
        .map(|agent| (&agent.id, agent.role.as_str()))
        .collect::<BTreeMap<_, _>>();
    registered == declared
}

/// Compares exact topology artifacts and permits only explicitly declared changes.
pub fn evaluate_topology_equivalence(
    experiment: &TopologyExperiment,
    experiment_sha256: Sha256Digest,
    baseline_bytes: &[u8],
    candidate_bytes: &[u8],
) -> Result<TopologyEquivalenceResult, TopologyControlError> {
    experiment
        .validate()
        .map_err(|_| TopologyControlError::InvalidExperiment)?;
    let baseline = parse_topology(baseline_bytes, TopologyControlError::InvalidBaseline)?;
    let candidate = parse_topology(candidate_bytes, TopologyControlError::InvalidCandidate)?;

    let mut reasons = BTreeSet::new();
    if Sha256Digest::from_bytes(baseline_bytes) != experiment.baseline_topology_sha256 {
        reasons.insert("baseline_hash_mismatch".to_owned());
    }
    if Sha256Digest::from_bytes(candidate_bytes) != experiment.candidate_topology_sha256 {
        reasons.insert("candidate_hash_mismatch".to_owned());
    }

    let baseline_value =
        serde_json::to_value(baseline).map_err(|_| TopologyControlError::Serialization)?;
    let candidate_value =
        serde_json::to_value(candidate).map_err(|_| TopologyControlError::Serialization)?;
    let mut observed = BTreeSet::new();
    collect_changes("", &baseline_value, &candidate_value, &mut observed);
    if observed != experiment.changed_variables {
        reasons.insert("changed_variable_mismatch".to_owned());
    }

    let status = if reasons.is_empty() {
        EquivalenceStatus::Eligible
    } else {
        EquivalenceStatus::Ineligible
    };
    let result = TopologyEquivalenceResult {
        schema_version: RESULT_SCHEMA_VERSION,
        experiment_sha256,
        status,
        declared_changes: experiment.changed_variables.clone(),
        observed_changes: observed,
        mismatch_reason_codes: reasons,
    };
    result
        .validate()
        .map_err(|_| TopologyControlError::InvalidExperiment)?;
    Ok(result)
}

fn parse_topology(
    bytes: &[u8],
    error: TopologyControlError,
) -> Result<DeploymentTopology, TopologyControlError> {
    let topology: DeploymentTopology = serde_json::from_slice(bytes).map_err(|_| error)?;
    topology.validate().map_err(|_| match error {
        TopologyControlError::InvalidBaseline => TopologyControlError::InvalidBaseline,
        _ => TopologyControlError::InvalidCandidate,
    })?;
    Ok(topology)
}

fn collect_changes(
    path: &str,
    baseline: &Value,
    candidate: &Value,
    changes: &mut BTreeSet<String>,
) {
    match (baseline, candidate) {
        (Value::Object(left), Value::Object(right)) => {
            let keys = left.keys().chain(right.keys()).collect::<BTreeSet<_>>();
            for key in keys {
                let child = format!("{path}/{}", escape_pointer(key));
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => collect_changes(&child, left, right, changes),
                    _ => {
                        changes.insert(child);
                    }
                }
            }
        }
        (Value::Array(left), Value::Array(right)) if left == right => {}
        (Value::Array(_), Value::Array(_)) => {
            changes.insert(path.to_owned());
        }
        _ if baseline != candidate => {
            changes.insert(path.to_owned());
        }
        _ => {}
    }
}

fn escape_pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn architecture_kind(architecture: DeploymentArchitecture) -> Option<TopologyKind> {
    match architecture {
        DeploymentArchitecture::SingleAgent => Some(TopologyKind::SingleAgent),
        DeploymentArchitecture::SupervisorWorker => Some(TopologyKind::SupervisorWorker),
        DeploymentArchitecture::Hierarchical => Some(TopologyKind::Hierarchical),
        DeploymentArchitecture::PeerToPeer => Some(TopologyKind::PeerToPeer),
        DeploymentArchitecture::External => None,
    }
}
