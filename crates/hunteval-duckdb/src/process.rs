use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use hunteval_sandbox::{
    GuestMount, ResolvedExecutionPolicy, SandboxSpec, SupervisedChild, classify_exit_status,
};

use crate::{
    SqlRequest, TableRegistration, ToolError, ToolErrorCode, ToolResult,
    worker::{WorkerCommand, WorkerResponse},
};

const MAX_COMMAND_BYTES: usize = 1_048_576;
const MAX_WIRE_RESPONSE_BYTES: u64 = 4_200_000;
const MAX_STDERR_BYTES: u64 = 64 * 1024;

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
        let (tables, mounts) = sandbox_tables(&self.tables)?;
        let command = WorkerCommand { tables, request };
        let input = serde_json::to_vec(&command).map_err(|_| worker_protocol())?;
        if input.len() > MAX_COMMAND_BYTES {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "worker command exceeds the protocol bound",
            ));
        }

        let timeout = Duration::from_millis(command.request.limits.timeout_ms);
        let mut child = spawn_worker(
            &self.executable,
            &self.arguments,
            mounts,
            command.request.limits.memory_limit_mb,
            timeout,
        )?;
        let mut stdin = child.take_stdin().map_err(|_| unavailable())?;
        if stdin.write_all(&input).is_err() {
            drop(stdin);
            let _ = child.terminate();
            return Err(unavailable());
        }
        drop(stdin);
        let stdout = child.take_stdout().map_err(|_| unavailable())?;
        let stderr = child.take_stderr().map_err(|_| unavailable())?;
        let reader = thread::spawn(move || read_bounded(stdout, MAX_WIRE_RESPONSE_BYTES));
        let stderr_reader = thread::spawn(move || read_bounded(stderr, MAX_STDERR_BYTES));

        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|_| unavailable())? {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.terminate();
                let _ = reader.join();
                let _ = stderr_reader.join();
                return Err(ToolError::new(
                    ToolErrorCode::Timeout,
                    "managed SQL worker exceeded its time limit",
                ));
            }
            thread::sleep(Duration::from_millis(2));
        };
        let output = join_reader(reader)?;
        let _ = stderr_reader.join();
        if !status.success() {
            if classify_exit_status(status).is_some() {
                return Err(ToolError::new(
                    ToolErrorCode::ResourceLimit,
                    "managed SQL worker exceeded an operating-system resource limit",
                ));
            }
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

fn spawn_worker(
    executable: &Path,
    arguments: &[String],
    mounts: Vec<GuestMount>,
    memory_limit_mb: u32,
    timeout: Duration,
) -> Result<SupervisedChild, ToolError> {
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.wall_time_ms = u64::try_from(timeout.as_millis()).map_err(|_| unavailable())?;
    policy.limits.cpu_time_seconds = timeout.as_secs().saturating_add(1).max(1);
    policy.limits.address_space_bytes = u64::from(memory_limit_mb)
        .saturating_mul(8)
        .saturating_mul(1024 * 1024)
        .max(1024 * 1024 * 1024);
    policy.limits.stdout_bytes = MAX_WIRE_RESPONSE_BYTES as usize;
    policy.limits.stderr_bytes = MAX_STDERR_BYTES as usize;
    let executable = executable.canonicalize().map_err(|_| unavailable())?;
    let spec = SandboxSpec {
        executable,
        arguments: arguments.to_vec(),
        mounts,
        working_directory: "/tmp".to_owned(),
        environment: BTreeMap::new(),
        policy,
    };
    hunteval_sandbox::spawn(&spec).map_err(|_| unavailable())
}

fn sandbox_tables(
    tables: &[TableRegistration],
) -> Result<(Vec<TableRegistration>, Vec<GuestMount>), ToolError> {
    let mut mapped = Vec::with_capacity(tables.len());
    let mut mounts = Vec::with_capacity(tables.len());
    for (index, table) in tables.iter().enumerate() {
        table.validate()?;
        let metadata = fs::symlink_metadata(&table.parquet_path).map_err(|_| unavailable())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(unavailable());
        }
        let host_path = table
            .parquet_path
            .canonicalize()
            .map_err(|_| unavailable())?;
        let guest_path = format!("/hunteval/input-{index}.parquet");
        mounts.push(GuestMount::read_only(&host_path, &guest_path));
        mapped.push(TableRegistration {
            name: table.name.clone(),
            parquet_path: PathBuf::from(guest_path),
        });
    }
    Ok((mapped, mounts))
}

fn read_bounded(reader: impl Read, limit: u64) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    reader.take(limit + 1).read_to_end(&mut output)?;
    Ok(output)
}

fn join_reader(
    reader: thread::JoinHandle<Result<Vec<u8>, std::io::Error>>,
) -> Result<Vec<u8>, ToolError> {
    reader
        .join()
        .map_err(|_| worker_protocol())?
        .map_err(|_| worker_protocol())
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
