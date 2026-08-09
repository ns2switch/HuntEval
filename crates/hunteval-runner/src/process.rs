use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use thiserror::Error;

use hunteval_sandbox::{
    GuestMount, LimitKind, RedactionPolicy, Redactor, ResolvedExecutionPolicy, SandboxSpec,
    classify_exit_status,
};

use crate::IsolationPolicy;

/// Explicit child-process launch description.
#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub working_directory: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub timeout: Duration,
    pub output_limit_bytes: usize,
    pub redacted_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub exit_code: i32,
}

/// Adapter that launches a deployment with inherited state removed.
#[derive(Debug, Default)]
pub struct DeploymentProcess;

impl DeploymentProcess {
    pub fn run(spec: &ProcessSpec) -> Result<ProcessOutput, ProcessError> {
        if spec.output_limit_bytes == 0 {
            return Err(ProcessError::InvalidOutputLimit);
        }
        let mut child = Command::new(&spec.executable)
            .args(&spec.arguments)
            .current_dir(&spec.working_directory)
            .env_clear()
            .envs(&spec.environment)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(ProcessError::Spawn)?;
        let stdout = child.stdout.take().ok_or(ProcessError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(ProcessError::MissingPipe)?;
        let limit = spec.output_limit_bytes;
        let out_reader = thread::spawn(move || read_bounded(stdout, limit));
        let err_reader = thread::spawn(move || read_bounded(stderr, limit));
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(ProcessError::Wait)? {
                break status;
            }
            if started.elapsed() >= spec.timeout {
                child.kill().map_err(ProcessError::Kill)?;
                let _ = child.wait().map_err(ProcessError::Wait)?;
                return Err(ProcessError::Timeout);
            }
            thread::sleep(Duration::from_millis(5));
        };
        let stdout = join_reader(out_reader)?;
        let raw_stderr = join_reader(err_reader)?;
        let stderr = redact(&raw_stderr, &spec.redacted_values)?;
        if !status.success() {
            if let Some(limit) = classify_exit_status(status) {
                return Err(ProcessError::ResourceLimit(limit));
            }
            return Err(ProcessError::Exit {
                code: status.code().ok_or(ProcessError::Signalled)?,
                stderr,
            });
        }
        let exit_code = status.code().ok_or(ProcessError::Signalled)?;
        Ok(ProcessOutput {
            stdout,
            stderr,
            exit_code,
        })
    }
}

/// Linux bubblewrap backend with a read-only public root and no network namespace.
#[derive(Debug, Default)]
pub struct LinuxSandbox;

impl LinuxSandbox {
    pub fn run(
        executable: &std::path::Path,
        arguments: &[String],
        policy: &IsolationPolicy,
        timeout: Duration,
        output_limit_bytes: usize,
    ) -> Result<ProcessOutput, ProcessError> {
        if output_limit_bytes == 0 {
            return Err(ProcessError::InvalidOutputLimit);
        }
        let mut execution_policy = ResolvedExecutionPolicy::hardened_default();
        execution_policy.limits.wall_time_ms =
            u64::try_from(timeout.as_millis()).map_err(|_| ProcessError::InvalidTimeout)?;
        execution_policy.limits.stdout_bytes = output_limit_bytes;
        execution_policy.limits.stderr_bytes = output_limit_bytes;
        let public_root = policy
            .public_root()
            .canonicalize()
            .map_err(ProcessError::Executable)?;
        let executable = executable
            .canonicalize()
            .map_err(ProcessError::Executable)?;
        let spec = SandboxSpec {
            executable,
            arguments: arguments.to_vec(),
            mounts: vec![GuestMount::read_only(public_root, "/episode")],
            working_directory: "/episode".to_owned(),
            environment: policy.environment().clone(),
            policy: execution_policy,
        };
        let mut child = hunteval_sandbox::spawn(&spec).map_err(ProcessError::Sandbox)?;
        let stdout = child.take_stdout().map_err(ProcessError::Sandbox)?;
        let stderr = child.take_stderr().map_err(ProcessError::Sandbox)?;
        drop(child.take_stdin().map_err(ProcessError::Sandbox)?);
        let out_reader = thread::spawn(move || read_bounded(stdout, output_limit_bytes));
        let err_reader = thread::spawn(move || read_bounded(stderr, output_limit_bytes));
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(ProcessError::Sandbox)? {
                break status;
            }
            if started.elapsed() >= timeout {
                child.terminate().map_err(ProcessError::Sandbox)?;
                return Err(ProcessError::Timeout);
            }
            thread::sleep(Duration::from_millis(5));
        };
        let stdout = join_reader(out_reader)?;
        let stderr = redact(
            &join_reader(err_reader)?,
            &policy.environment().values().cloned().collect::<Vec<_>>(),
        )?;
        if !status.success() {
            if let Some(limit) = classify_exit_status(status) {
                return Err(ProcessError::ResourceLimit(limit));
            }
            return Err(ProcessError::Exit {
                code: status.code().ok_or(ProcessError::Signalled)?,
                stderr,
            });
        }
        let exit_code = status.code().ok_or(ProcessError::Signalled)?;
        Ok(ProcessOutput {
            stdout,
            stderr,
            exit_code,
        })
    }
}

fn redact(bytes: &[u8], values: &[String]) -> Result<String, ProcessError> {
    Redactor::new(
        RedactionPolicy::default(),
        values.iter().filter(|value| !value.is_empty()).cloned(),
    )
    .map(|redactor| redactor.redact_bytes(bytes).text)
    .map_err(ProcessError::Redaction)
}

fn read_bounded(reader: impl Read, limit: usize) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    reader.take((limit + 1) as u64).read_to_end(&mut output)?;
    if output.len() > limit {
        output.truncate(limit);
    }
    Ok(output)
}

fn join_reader(
    handle: thread::JoinHandle<Result<Vec<u8>, io::Error>>,
) -> Result<Vec<u8>, ProcessError> {
    handle
        .join()
        .map_err(|_| ProcessError::ReaderPanicked)?
        .map_err(ProcessError::Read)
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("output limit must be positive")]
    InvalidOutputLimit,
    #[error("process timeout is outside the supported range")]
    InvalidTimeout,
    #[error("could not launch deployment: {0}")]
    Spawn(io::Error),
    #[error("child process pipe was unavailable")]
    MissingPipe,
    #[error("could not inspect deployment process: {0}")]
    Wait(io::Error),
    #[error("could not terminate deployment process: {0}")]
    Kill(io::Error),
    #[error("deployment process exceeded its time budget")]
    Timeout,
    #[error("deployment exceeded an operating-system resource limit: {0:?}")]
    ResourceLimit(LimitKind),
    #[error("deployment executable is unavailable: {0}")]
    Executable(io::Error),
    #[error("deployment sandbox failed")]
    Sandbox(#[source] hunteval_sandbox::SandboxError),
    #[error("deployment diagnostic redaction failed")]
    Redaction(#[source] hunteval_sandbox::RedactionError),
    #[error("deployment process output could not be read: {0}")]
    Read(io::Error),
    #[error("deployment output reader failed")]
    ReaderPanicked,
    #[error("deployment terminated without an exit code")]
    Signalled,
    #[error("deployment exited with code {code}: {stderr}")]
    Exit { code: i32, stderr: String },
}
