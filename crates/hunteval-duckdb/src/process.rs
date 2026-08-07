use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use crate::{
    SqlRequest, TableRegistration, ToolError, ToolErrorCode, ToolResult,
    worker::{WorkerCommand, WorkerResponse},
};

const MAX_COMMAND_BYTES: usize = 1_048_576;
const MAX_WIRE_RESPONSE_BYTES: u64 = 4_200_000;

/// Runner-side adapter for one isolated DuckDB worker process per query.
#[derive(Debug, Clone)]
pub struct DuckDbWorker {
    executable: PathBuf,
    arguments: Vec<String>,
    tables: Vec<TableRegistration>,
}

impl DuckDbWorker {
    #[must_use]
    pub fn new(executable: impl Into<PathBuf>, tables: Vec<TableRegistration>) -> Self {
        Self {
            executable: executable.into(),
            arguments: Vec::new(),
            tables,
        }
    }

    /// Adds fixed runner-controlled arguments used by a multi-call executable.
    #[must_use]
    pub fn with_arguments(mut self, arguments: Vec<String>) -> Self {
        self.arguments = arguments;
        self
    }

    /// Executes a request without allowing worker failure to terminate the caller.
    pub fn execute(&self, request: SqlRequest) -> Result<ToolResult, ToolError> {
        request.validate()?;
        validate_executable(&self.executable)?;
        let command = WorkerCommand {
            tables: self.tables.clone(),
            request,
        };
        let input = serde_json::to_vec(&command).map_err(|_| worker_protocol())?;
        if input.len() > MAX_COMMAND_BYTES {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "worker command exceeds the protocol bound",
            ));
        }

        let timeout = Duration::from_millis(command.request.limits.timeout_ms);
        let mut child = ChildGuard::spawn(&self.executable, &self.arguments)?;
        let mut stdin = child.0.stdin.take().ok_or_else(unavailable)?;
        if stdin.write_all(&input).is_err() {
            drop(stdin);
            return match child.0.wait() {
                Ok(status) if !status.success() => Err(worker_crashed()),
                _ => Err(unavailable()),
            };
        }
        drop(stdin);
        let stdout = child.0.stdout.take().ok_or_else(unavailable)?;
        let reader = thread::spawn(move || {
            let mut output = Vec::new();
            stdout
                .take(MAX_WIRE_RESPONSE_BYTES + 1)
                .read_to_end(&mut output)
                .map(|_| output)
        });

        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = child.0.try_wait().map_err(|_| unavailable())? {
                break status;
            }
            if Instant::now() >= deadline {
                child.terminate();
                let _ = reader.join();
                return Err(ToolError::new(
                    ToolErrorCode::Timeout,
                    "managed SQL worker exceeded its time limit",
                ));
            }
            thread::sleep(Duration::from_millis(2));
        };
        let output = reader
            .join()
            .map_err(|_| worker_protocol())?
            .map_err(|_| worker_protocol())?;
        if !status.success() {
            return Err(worker_crashed());
        }
        if output.len() > MAX_WIRE_RESPONSE_BYTES as usize {
            return Err(worker_protocol());
        }
        match serde_json::from_slice::<WorkerResponse>(&output).map_err(|_| worker_protocol())? {
            WorkerResponse::Success { result } => Ok(result),
            WorkerResponse::Failure { error } => Err(error),
        }
    }
}

#[derive(Debug)]
struct ChildGuard(Child);

impl ChildGuard {
    fn spawn(executable: &Path, arguments: &[String]) -> Result<Self, ToolError> {
        Command::new(executable)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map(Self)
            .map_err(|_| unavailable())
    }

    fn terminate(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn validate_executable(path: &Path) -> Result<(), ToolError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| unavailable())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> ToolError {
    ToolError::new(
        ToolErrorCode::WorkerUnavailable,
        "managed SQL worker is unavailable",
    )
}

fn worker_protocol() -> ToolError {
    ToolError::worker_protocol()
}

fn worker_crashed() -> ToolError {
    ToolError::new(
        ToolErrorCode::WorkerCrashed,
        "managed SQL worker exited unsuccessfully",
    )
}
