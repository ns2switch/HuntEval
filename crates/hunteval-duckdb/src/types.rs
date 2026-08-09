use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Scalar values accepted as bound SQL parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SqlParameter {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

/// Scalar values returned by the managed tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum SqlValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

/// Per-query resource limits enforced by the process and engine boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryLimits {
    pub timeout_ms: u64,
    pub memory_limit_mb: u32,
    pub max_rows: usize,
    pub max_output_bytes: usize,
}

impl Default for QueryLimits {
    fn default() -> Self {
        Self {
            timeout_ms: 2_000,
            memory_limit_mb: 128,
            max_rows: 1_000,
            max_output_bytes: 1_048_576,
        }
    }
}

impl QueryLimits {
    pub(crate) fn validate(self) -> Result<(), ToolError> {
        if !(10..=30_000).contains(&self.timeout_ms)
            || !(16..=1_024).contains(&self.memory_limit_mb)
            || !(1..=10_000).contains(&self.max_rows)
            || !(256..=4_194_304).contains(&self.max_output_bytes)
        {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "query limits are outside the supported range",
            ));
        }
        Ok(())
    }
}

/// Deployment-provided SQL with values bound separately from its text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqlRequest {
    pub query: String,
    #[serde(default)]
    pub parameters: Vec<SqlParameter>,
    #[serde(default)]
    pub limits: QueryLimits,
}

impl SqlRequest {
    pub(crate) fn validate(&self) -> Result<(), ToolError> {
        self.limits.validate()?;
        if self.query.trim().is_empty() || self.query.len() > 65_536 || self.parameters.len() > 128
        {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "SQL request exceeds a supported bound",
            ));
        }
        if self
            .parameters
            .iter()
            .any(|parameter| matches!(parameter, SqlParameter::Float(value) if !value.is_finite()))
        {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "SQL parameters must be finite",
            ));
        }
        Ok(())
    }
}

/// Trusted mapping from a logical table name to one public Parquet artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableRegistration {
    pub name: String,
    pub parquet_path: PathBuf,
}

impl TableRegistration {
    pub(crate) fn validate(&self) -> Result<(), ToolError> {
        let mut characters = self.name.chars();
        let starts_safely = characters
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character == '_');
        let rest_is_safe = characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
        if !starts_safely || !rest_is_safe || self.name == "normalized_events" {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "table registration has an invalid logical name",
            ));
        }
        Ok(())
    }
}

/// Deterministically serialized successful query output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<SqlValue>>,
    pub truncated: bool,
}

/// Stable error categories exposed across the worker boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorCode {
    InvalidRequest,
    SqlRejected,
    QueryFailed,
    OutputLimit,
    Timeout,
    WorkerUnavailable,
    WorkerCrashed,
    WorkerProtocol,
    ResourceLimit,
}

/// Safe error without query text, filesystem paths, or engine internals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{message}")]
#[serde(deny_unknown_fields)]
pub struct ToolError {
    pub code: ToolErrorCode,
    pub message: String,
}

impl ToolError {
    pub(crate) fn new(code: ToolErrorCode, message: &'static str) -> Self {
        Self {
            code,
            message: message.to_owned(),
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn worker_protocol() -> Self {
        Self::new(
            ToolErrorCode::WorkerProtocol,
            "worker protocol message is invalid",
        )
    }
}
