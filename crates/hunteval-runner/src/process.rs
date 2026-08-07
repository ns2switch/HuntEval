use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use thiserror::Error;

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
        let mut stderr = String::from_utf8_lossy(&raw_stderr).into_owned();
        for secret in spec
            .redacted_values
            .iter()
            .filter(|value| !value.is_empty())
        {
            stderr = stderr.replace(secret, "[REDACTED]");
        }
        let exit_code = status.code().ok_or(ProcessError::Signalled)?;
        if !status.success() {
            return Err(ProcessError::Exit {
                code: exit_code,
                stderr,
            });
        }
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
        let executable = executable
            .canonicalize()
            .map_err(ProcessError::Executable)?;
        let mut sandbox_arguments = vec![
            "--unshare-all",
            "--die-with-parent",
            "--new-session",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/bin",
            "/bin",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "--ro-bind",
            path_text(policy.public_root())?,
            "/episode",
            "--ro-bind",
            path_text(&executable)?,
            "/deployment",
            "--chdir",
            "/episode",
            "--",
            "/deployment",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        sandbox_arguments.extend(arguments.iter().cloned());
        DeploymentProcess::run(&ProcessSpec {
            executable: PathBuf::from("/usr/bin/bwrap"),
            arguments: sandbox_arguments,
            working_directory: PathBuf::from("/"),
            environment: policy.environment().clone(),
            timeout,
            output_limit_bytes,
            redacted_values: policy.environment().values().cloned().collect(),
        })
    }
}

fn path_text(path: &std::path::Path) -> Result<&str, ProcessError> {
    path.to_str().ok_or(ProcessError::NonUtf8Path)
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
    #[error("deployment executable is unavailable: {0}")]
    Executable(io::Error),
    #[error("sandbox paths must be valid UTF-8")]
    NonUtf8Path,
    #[error("deployment process output could not be read: {0}")]
    Read(io::Error),
    #[error("deployment output reader failed")]
    ReaderPanicked,
    #[error("deployment terminated without an exit code")]
    Signalled,
    #[error("deployment exited with code {code}: {stderr}")]
    Exit { code: i32, stderr: String },
}
