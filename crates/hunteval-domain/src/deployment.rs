use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{AgentId, ContractValidationError, DeploymentId, Sha256Digest};

/// Declared communication topology of an evaluated deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentArchitecture {
    SingleAgent,
    SupervisorWorker,
    Hierarchical,
    PeerToPeer,
    External,
}

/// Observable identity and immutable configuration of one agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRegistration {
    pub id: AgentId,
    pub role: String,
    pub capabilities: BTreeSet<String>,
    pub prompt_version: String,
    pub prompt_sha256: Sha256Digest,
    pub model: String,
    #[serde(default)]
    pub model_parameters: BTreeMap<String, serde_json::Value>,
}

/// Complete deployment registration submitted during protocol negotiation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentRegistration {
    pub id: DeploymentId,
    pub architecture: DeploymentArchitecture,
    pub version: String,
    pub agents: Vec<AgentRegistration>,
}

impl DeploymentRegistration {
    /// Validates identifiers, configuration text, capabilities, and episode limits.
    pub fn validate(&self, max_agents: u16) -> Result<(), ContractValidationError> {
        require_text(&self.version, "deployment.version")?;
        if self.agents.is_empty() || self.agents.len() > usize::from(max_agents) {
            return Err(ContractValidationError::new(
                "deployment.agents",
                "agent count must be within episode limits",
            ));
        }

        let mut agent_ids = BTreeSet::new();
        for agent in &self.agents {
            if !agent_ids.insert(&agent.id) {
                return Err(ContractValidationError::new(
                    "deployment.agents.id",
                    "agent identifiers must be unique",
                ));
            }
            require_text(&agent.role, "deployment.agents.role")?;
            require_text(&agent.prompt_version, "deployment.agents.prompt_version")?;
            require_text(&agent.model, "deployment.agents.model")?;
            if agent.capabilities.is_empty()
                || agent
                    .capabilities
                    .iter()
                    .any(|value| value.trim().is_empty())
            {
                return Err(ContractValidationError::new(
                    "deployment.agents.capabilities",
                    "at least one nonempty capability is required",
                ));
            }
        }
        Ok(())
    }
}

fn require_text(value: &str, field: &'static str) -> Result<(), ContractValidationError> {
    if value.trim().is_empty() {
        return Err(ContractValidationError::new(
            field,
            "value must not be empty",
        ));
    }
    Ok(())
}
