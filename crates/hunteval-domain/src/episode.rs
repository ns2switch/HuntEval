use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{ContractValidationError, EpisodeId, SchemaVersion};

mod ground_truth;

pub use ground_truth::{ExpectedTimelineWindow, GroundTruth};

/// Cloud provider represented by an episode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Aws,
    Azure,
    Gcp,
}

/// Public investigation objective supplied to the deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeObjective {
    pub primary: String,
    #[serde(default)]
    pub secondary: Vec<String>,
}

/// One provider-native telemetry table exposed under a logical name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryTable {
    pub name: String,
    pub path: String,
}

/// Public telemetry configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    pub tables: Vec<TelemetryTable>,
}

/// Optional author-supplied knowledge paths. Empty for the vertical slice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeConfig {
    #[serde(default)]
    pub documents: Vec<String>,
}

/// Deployment-level limits resolved before a run starts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeLimits {
    pub max_agents: u16,
    pub max_parallel_agents: u16,
    pub max_parallel_tool_calls: u16,
    pub max_outstanding_tasks: u32,
    pub max_delegation_depth: u16,
    pub max_tool_calls: u32,
    pub max_sql_queries: u32,
    pub max_retrieved_documents: u32,
    pub max_messages: u32,
    pub max_duration_seconds: u64,
    pub max_tokens: u64,
    pub max_estimated_cost: Option<f64>,
}

/// Deployment-visible episode manifest. It cannot represent private ground truth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeManifest {
    pub schema_version: SchemaVersion,
    pub id: EpisodeId,
    pub title: String,
    pub provider: Provider,
    pub category: String,
    pub objective: EpisodeObjective,
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub knowledge: KnowledgeConfig,
    pub limits: EpisodeLimits,
    pub fault_profile: Option<String>,
    #[serde(default)]
    pub benign_evaluation: bool,
}

impl EpisodeManifest {
    /// Validates cross-field and path invariants before the manifest is trusted.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_text(&self.title, "title")?;
        require_text(&self.category, "category")?;
        require_text(&self.objective.primary, "objective.primary")?;
        if self.telemetry.tables.is_empty() {
            return Err(ContractValidationError::new(
                "telemetry.tables",
                "at least one table is required",
            ));
        }

        let mut table_names = BTreeSet::new();
        for table in &self.telemetry.tables {
            require_text(&table.name, "telemetry.tables.name")?;
            validate_relative_path(&table.path, "telemetry.tables.path")?;
            if !table_names.insert(table.name.as_str()) {
                return Err(ContractValidationError::new(
                    "telemetry.tables.name",
                    "table names must be unique",
                ));
            }
        }
        for document in &self.knowledge.documents {
            validate_relative_path(document, "knowledge.documents")?;
        }
        self.limits.validate()
    }
}

impl EpisodeLimits {
    /// Validates nonzero and relational budget invariants.
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        if self.max_agents == 0 || self.max_parallel_agents == 0 {
            return Err(ContractValidationError::new(
                "limits.max_agents",
                "agent limits must be positive",
            ));
        }
        if self.max_parallel_agents > self.max_agents {
            return Err(ContractValidationError::new(
                "limits.max_parallel_agents",
                "parallel agents cannot exceed registered agents",
            ));
        }
        if self.max_tool_calls == 0 || self.max_sql_queries > self.max_tool_calls {
            return Err(ContractValidationError::new(
                "limits.max_sql_queries",
                "SQL queries must fit within the positive tool-call budget",
            ));
        }
        if self.max_messages == 0 || self.max_duration_seconds == 0 {
            return Err(ContractValidationError::new(
                "limits",
                "message and duration limits must be positive",
            ));
        }
        if self
            .max_estimated_cost
            .is_some_and(|cost| !cost.is_finite() || cost < 0.0)
        {
            return Err(ContractValidationError::new(
                "limits.max_estimated_cost",
                "cost must be finite and nonnegative",
            ));
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

fn validate_relative_path(value: &str, field: &'static str) -> Result<(), ContractValidationError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.starts_with('\\')
        || value.contains('\0')
        || value
            .split(['/', '\\'])
            .any(|part| part == ".." || part.is_empty())
    {
        return Err(ContractValidationError::new(
            field,
            "path must be relative and traversal-free",
        ));
    }
    Ok(())
}
