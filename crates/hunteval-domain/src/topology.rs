use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{AgentId, ContractValidationError, SchemaVersion, TopologyId};

const TOPOLOGY_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 6);
const MAX_AGENTS: usize = 1_024;
const MAX_RELATIONSHIPS: usize = 8_192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyKind {
    SingleAgent,
    SupervisorWorker,
    Hierarchical,
    PeerToPeer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationMode {
    Centralized,
    Decentralized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelComposition {
    Homogeneous,
    Heterogeneous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryMode {
    Shared,
    Isolated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskAllocationPolicy {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPattern {
    Sequential,
    Parallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologySpecialization {
    Generalist,
    Supervisor,
    IdentitySpecialist,
    TimelineSpecialist,
    EvidenceReviewer,
    Critic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipKind {
    DelegatesTo,
    CoordinatesWith,
    Reviews,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyAgent {
    pub id: AgentId,
    pub role: String,
    pub specialization: TopologySpecialization,
    pub model_assignment: String,
    pub memory_group: String,
    pub reviewer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyRelationship {
    pub source: AgentId,
    pub target: AgentId,
    pub kind: RelationshipKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentTopology {
    pub schema_version: SchemaVersion,
    pub id: TopologyId,
    pub kind: TopologyKind,
    pub coordination: CoordinationMode,
    pub model_composition: ModelComposition,
    pub memory: MemoryMode,
    pub task_allocation: TaskAllocationPolicy,
    pub execution_pattern: ExecutionPattern,
    pub agents: Vec<TopologyAgent>,
    pub relationships: BTreeSet<TopologyRelationship>,
}

impl DeploymentTopology {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.schema_version != TOPOLOGY_SCHEMA_VERSION {
            return Err(invalid("schema_version", "schema version is unsupported"));
        }
        if self.agents.is_empty() || self.agents.len() > MAX_AGENTS {
            return Err(invalid(
                "agents",
                "agent count is outside the supported bound",
            ));
        }
        if self.relationships.len() > MAX_RELATIONSHIPS {
            return Err(invalid(
                "relationships",
                "relationship count exceeds the supported bound",
            ));
        }

        let agents = self.validate_agents()?;
        self.validate_relationships(&agents)?;
        self.validate_kind(&agents)
    }

    fn validate_agents(
        &self,
    ) -> Result<BTreeMap<&AgentId, &TopologyAgent>, ContractValidationError> {
        let mut agents = BTreeMap::new();
        for agent in &self.agents {
            if !valid_text(&agent.role)
                || !valid_text(&agent.model_assignment)
                || !valid_text(&agent.memory_group)
            {
                return Err(invalid("agents", "agent text is empty or unbounded"));
            }
            if agents.insert(&agent.id, agent).is_some() {
                return Err(invalid("agents.id", "agent identifiers must be unique"));
            }
        }
        Ok(agents)
    }

    fn validate_relationships(
        &self,
        agents: &BTreeMap<&AgentId, &TopologyAgent>,
    ) -> Result<(), ContractValidationError> {
        for relationship in &self.relationships {
            if relationship.source == relationship.target {
                return Err(invalid("relationships", "self relationships are forbidden"));
            }
            if !agents.contains_key(&relationship.source)
                || !agents.contains_key(&relationship.target)
            {
                return Err(invalid(
                    "relationships",
                    "relationship references an unknown agent",
                ));
            }
            if relationship.kind == RelationshipKind::Reviews
                && !agents[&relationship.target].reviewer
            {
                return Err(invalid(
                    "relationships",
                    "review relationship target must be a reviewer",
                ));
            }
        }
        Ok(())
    }

    fn validate_kind(
        &self,
        agents: &BTreeMap<&AgentId, &TopologyAgent>,
    ) -> Result<(), ContractValidationError> {
        let supervisors = agents
            .values()
            .filter(|agent| agent.specialization == TopologySpecialization::Supervisor)
            .count();
        match self.kind {
            TopologyKind::SingleAgent
                if agents.len() != 1 || !self.relationships.is_empty() || supervisors != 0 =>
            {
                Err(invalid("kind", "single-agent topology invariants failed"))
            }
            TopologyKind::SupervisorWorker | TopologyKind::Hierarchical if supervisors != 1 => {
                Err(invalid("kind", "topology requires exactly one supervisor"))
            }
            TopologyKind::PeerToPeer if supervisors != 0 => Err(invalid(
                "kind",
                "peer-to-peer topology cannot declare a supervisor",
            )),
            _ => Ok(()),
        }
    }
}

fn valid_text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 128 && !value.contains(['\0', '\n', '\r'])
}

fn invalid(field: &'static str, reason: &'static str) -> ContractValidationError {
    ContractValidationError::new(field, reason)
}
