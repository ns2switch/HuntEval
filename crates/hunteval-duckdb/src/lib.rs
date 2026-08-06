//! Constrained DuckDB execution behind a short-lived worker process.

mod policy;
mod process;
mod types;
mod worker;

pub use policy::{SqlPolicy, SqlPolicyError};
pub use process::DuckDbWorker;
pub use types::{
    QueryLimits, SqlParameter, SqlRequest, SqlValue, TableRegistration, ToolError, ToolErrorCode,
    ToolResult,
};

#[doc(hidden)]
pub use worker::{WorkerCommand, execute_command};
