use hunteval_duckdb::{DuckDbWorker, SqlRequest, ToolError, ToolResult};

/// Runner-owned boundary through which deployments invoke scored tools.
pub trait ManagedTool: Send + Sync {
    fn execute_sql(&self, request: SqlRequest) -> Result<ToolResult, ToolError>;
}

impl ManagedTool for DuckDbWorker {
    fn execute_sql(&self, request: SqlRequest) -> Result<ToolResult, ToolError> {
        self.execute(request)
    }
}
