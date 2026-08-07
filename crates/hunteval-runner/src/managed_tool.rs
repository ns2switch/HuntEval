use serde_json::Value;
use thiserror::Error;

/// Runner-owned boundary through which deployments invoke scored tools.
pub trait ManagedTool: Send + Sync {
    fn execute(&self, tool: &str, arguments: &Value) -> Result<Value, ManagedToolError>;
}

/// Stable adapter error that does not couple orchestration to a tool engine.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManagedToolError {
    #[error("unknown managed tool")]
    UnknownTool,
    #[error("managed tool request was invalid: {0}")]
    InvalidRequest(String),
    #[error("managed tool execution failed: {0}")]
    Execution(String),
}
