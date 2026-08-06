use std::io::{self, Read};

use hunteval_duckdb::{ToolError, WorkerCommand, execute_command};
use serde::Serialize;

const MAX_COMMAND_BYTES: u64 = 1_048_576;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Response<'a> {
    Success {
        result: &'a hunteval_duckdb::ToolResult,
    },
    Failure {
        error: &'a ToolError,
    },
}

fn main() {
    let response = read_command().and_then(execute_command);
    let wire = match &response {
        Ok(result) => Response::Success { result },
        Err(error) => Response::Failure { error },
    };
    if serde_json::to_writer(io::stdout().lock(), &wire).is_err() {
        std::process::exit(1);
    }
}

fn read_command() -> Result<WorkerCommand, ToolError> {
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(MAX_COMMAND_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ToolError::worker_protocol())?;
    if bytes.len() > MAX_COMMAND_BYTES as usize {
        return Err(ToolError::worker_protocol());
    }
    serde_json::from_slice(&bytes).map_err(|_| ToolError::worker_protocol())
}
