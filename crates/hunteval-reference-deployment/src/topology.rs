use std::collections::{BTreeMap, BTreeSet};

use clap::ValueEnum;
use hunteval_domain::{
    AgentId, AgentRegistration, DeploymentArchitecture, DeploymentId, DeploymentRegistration,
    IdValidationError, Sha256Digest,
};

/// Supported deterministic reference deployment topologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReferenceTopology {
    SingleAgent,
    SupervisorWorker,
    SupervisorSpecialists,
}

impl ReferenceTopology {
    pub(super) fn registration(self) -> Result<DeploymentRegistration, IdValidationError> {
        let (deployment, architecture, agents) = match self {
            Self::SingleAgent => (
                "single-agent-scripted",
                DeploymentArchitecture::SingleAgent,
                vec![(
                    "investigator",
                    "investigator",
                    &["iam_analysis", "sql_query"][..],
                )],
            ),
            Self::SupervisorWorker => (
                "two-agent-scripted",
                DeploymentArchitecture::SupervisorWorker,
                vec![
                    (
                        "supervisor",
                        "orchestrator",
                        &["delegation", "synthesis"][..],
                    ),
                    (
                        "investigator",
                        "investigator",
                        &["iam_analysis", "sql_query"][..],
                    ),
                ],
            ),
            Self::SupervisorSpecialists => (
                "supervisor-specialists-scripted",
                DeploymentArchitecture::Hierarchical,
                vec![
                    (
                        "supervisor",
                        "orchestrator",
                        &["delegation", "synthesis"][..],
                    ),
                    (
                        "identity-specialist",
                        "identity_investigator",
                        &["iam_analysis", "sql_query"][..],
                    ),
                    (
                        "persistence-specialist",
                        "persistence_investigator",
                        &["persistence_analysis", "sql_query"][..],
                    ),
                ],
            ),
        };
        let agents = agents
            .into_iter()
            .map(|(id, role, capabilities)| agent(self, id, role, capabilities))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DeploymentRegistration {
            id: DeploymentId::new(deployment)?,
            architecture,
            version: "0.2.0".to_owned(),
            agents,
        })
    }

    pub(super) fn coordinator(self) -> Result<AgentId, IdValidationError> {
        match self {
            Self::SingleAgent => AgentId::new("investigator"),
            Self::SupervisorWorker | Self::SupervisorSpecialists => AgentId::new("supervisor"),
        }
    }

    pub(super) fn investigator(self) -> Result<AgentId, IdValidationError> {
        match self {
            Self::SingleAgent | Self::SupervisorWorker => AgentId::new("investigator"),
            Self::SupervisorSpecialists => AgentId::new("identity-specialist"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SingleAgent => "single-agent",
            Self::SupervisorWorker => "supervisor-worker",
            Self::SupervisorSpecialists => "supervisor-specialists",
        }
    }
}

fn agent(
    topology: ReferenceTopology,
    id: &str,
    role: &str,
    capabilities: &[&str],
) -> Result<AgentRegistration, IdValidationError> {
    let capabilities = capabilities
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    Ok(AgentRegistration {
        id: AgentId::new(id)?,
        role: role.to_owned(),
        capabilities,
        prompt_version: "reference-1.0.0".to_owned(),
        prompt_sha256: Sha256Digest::from_bytes(format!(
            "hunteval-reference:{}:{id}",
            topology.label()
        )),
        model: "scripted/reference".to_owned(),
        model_parameters: BTreeMap::new(),
    })
}
