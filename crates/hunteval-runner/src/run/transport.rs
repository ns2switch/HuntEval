use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::ChildStdin,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use hunteval_protocol::{JsonlDecoder, ProtocolEnvelope};
use hunteval_sandbox::{
    GuestMount, LimitKind, RedactionPolicy, Redactor, ResolvedExecutionPolicy, SandboxSpec,
    SupervisedChild, classify_exit_status,
};
use thiserror::Error;

const MAX_STDERR_BYTES: usize = 64 * 1024;

pub(super) struct ProtocolProcess {
    child: SupervisedChild,
    input: Option<ChildStdin>,
    lines: Receiver<Result<Vec<u8>, std::io::Error>>,
    stderr: Receiver<Result<Vec<u8>, std::io::Error>>,
    decoder: JsonlDecoder,
    deadline: Instant,
    redactor: Redactor,
}

impl ProtocolProcess {
    pub(super) fn spawn(
        executable: &Path,
        arguments: &[String],
        working_directory: &Path,
        environment: &BTreeMap<String, String>,
        policy: ResolvedExecutionPolicy,
        maximum_line_bytes: usize,
    ) -> Result<Self, TransportError> {
        let decoder = JsonlDecoder::new(maximum_line_bytes).map_err(TransportError::Protocol)?;
        let read_limit = maximum_line_bytes
            .checked_add(1)
            .ok_or(TransportError::InvalidLimit)?;
        let timeout = policy.limits.wall_time();
        let public_root = working_directory
            .canonicalize()
            .map_err(TransportError::PublicRoot)?;
        let executable = executable
            .canonicalize()
            .map_err(TransportError::Executable)?;
        let spec = SandboxSpec {
            executable,
            arguments: arguments.to_vec(),
            mounts: vec![GuestMount::read_only(public_root, "/episode")],
            working_directory: "/episode".to_owned(),
            environment: environment.clone(),
            policy,
        };
        let mut child = hunteval_sandbox::spawn(&spec).map_err(TransportError::Sandbox)?;
        let input = child.take_stdin().map_err(TransportError::Sandbox)?;
        let output = child.take_stdout().map_err(TransportError::Sandbox)?;
        let stderr = child.take_stderr().map_err(TransportError::Sandbox)?;
        let lines = spawn_line_reader(output, read_limit);
        let stderr = spawn_stderr_reader(stderr);
        let redactor = Redactor::new(
            RedactionPolicy::default(),
            environment
                .values()
                .filter(|value| !value.is_empty())
                .cloned(),
        )
        .map_err(TransportError::Redaction)?;
        Ok(Self {
            child,
            input: Some(input),
            lines,
            stderr,
            decoder,
            deadline: Instant::now() + timeout,
            redactor,
        })
    }

    pub(super) fn send(&mut self, message: &ProtocolEnvelope) -> Result<(), TransportError> {
        let mut bytes = serde_json::to_vec(message).map_err(TransportError::Encode)?;
        bytes.push(b'\n');
        let input = self.input.as_mut().ok_or(TransportError::MissingPipe)?;
        input.write_all(&bytes).map_err(TransportError::Write)?;
        input.flush().map_err(TransportError::Write)
    }

    pub(super) fn receive(&mut self) -> Result<ProtocolEnvelope, TransportError> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or(TransportError::Timeout)?;
        let line = self
            .lines
            .recv_timeout(remaining)
            .map_err(|error| self.receive_error(error))??;
        self.decoder.decode(&line).map_err(TransportError::Protocol)
    }

    pub(super) fn finish(mut self) -> Result<(), TransportError> {
        drop(self.input.take());
        loop {
            if let Some(status) = self.child.try_wait().map_err(TransportError::Sandbox)? {
                let diagnostics = self.read_stderr();
                return if status.success() {
                    Ok(())
                } else if let Some(limit) = classify_exit_status(status) {
                    Err(TransportError::ResourceLimit(limit))
                } else {
                    Err(TransportError::Exit {
                        code: status.code(),
                        diagnostics,
                    })
                };
            }
            if Instant::now() >= self.deadline {
                self.terminate();
                return Err(TransportError::Timeout);
            }
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn read_stderr(&self) -> String {
        self.stderr
            .recv_timeout(Duration::from_millis(50))
            .ok()
            .and_then(Result::ok)
            .map(|bytes| self.redactor.redact_bytes(&bytes).text)
            .unwrap_or_default()
    }

    fn terminate(&mut self) {
        let _ = self.child.terminate();
    }

    fn receive_error(&mut self, error: mpsc::RecvTimeoutError) -> TransportError {
        match error {
            mpsc::RecvTimeoutError::Timeout => TransportError::Timeout,
            mpsc::RecvTimeoutError::Disconnected => self
                .child
                .try_wait()
                .ok()
                .flatten()
                .and_then(classify_exit_status)
                .map_or(TransportError::EarlyEof, TransportError::ResourceLimit),
        }
    }
}

impl Drop for ProtocolProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn spawn_line_reader(
    output: impl Read + Send + 'static,
    limit: usize,
) -> Receiver<Result<Vec<u8>, std::io::Error>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut reader = BufReader::new(output);
        loop {
            let mut line = Vec::new();
            let result = reader
                .by_ref()
                .take(limit as u64)
                .read_until(b'\n', &mut line);
            match result {
                Ok(0) => break,
                Ok(_) if sender.send(Ok(line)).is_err() => break,
                Ok(_) => {}
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    });
    receiver
}

fn spawn_stderr_reader(
    stderr: impl Read + Send + 'static,
) -> Receiver<Result<Vec<u8>, std::io::Error>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stderr
            .take((MAX_STDERR_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    receiver
}

pub(super) fn execution_policy(
    duration: Duration,
    maximum_line_bytes: usize,
) -> Result<ResolvedExecutionPolicy, TransportError> {
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.wall_time_ms = duration_millis(duration)?;
    policy.limits.stdout_bytes = maximum_line_bytes;
    policy.limits.stderr_bytes = MAX_STDERR_BYTES;
    policy.validate().map_err(TransportError::Policy)?;
    Ok(policy)
}

fn duration_millis(duration: Duration) -> Result<u64, TransportError> {
    u64::try_from(duration.as_millis()).map_err(|_| TransportError::InvalidLimit)
}

#[derive(Debug, Error)]
pub(super) enum TransportError {
    #[error("deployment protocol line limit is invalid")]
    InvalidLimit,
    #[error("deployment execution policy is invalid")]
    Policy(#[source] hunteval_sandbox::PolicyError),
    #[error("deployment public root is unavailable")]
    PublicRoot(#[source] std::io::Error),
    #[error("deployment executable is unavailable")]
    Executable(#[source] std::io::Error),
    #[error("deployment sandbox failed")]
    Sandbox(#[source] hunteval_sandbox::SandboxError),
    #[error("deployment diagnostic redaction failed")]
    Redaction(#[source] hunteval_sandbox::RedactionError),
    #[error("deployment process pipe is unavailable")]
    MissingPipe,
    #[error("could not encode runner protocol message")]
    Encode(#[source] serde_json::Error),
    #[error("could not write runner protocol message")]
    Write(#[source] std::io::Error),
    #[error("could not read deployment protocol message")]
    Read(#[from] std::io::Error),
    #[error("deployment protocol message is invalid")]
    Protocol(#[source] hunteval_protocol::ProtocolError),
    #[error("deployment closed the protocol before termination")]
    EarlyEof,
    #[error("deployment exceeded the run deadline")]
    Timeout,
    #[error("deployment exceeded an operating-system resource limit: {0:?}")]
    ResourceLimit(LimitKind),
    #[error("deployment exited unsuccessfully ({code:?}): {diagnostics}")]
    Exit {
        code: Option<i32>,
        diagnostics: String,
    },
}

impl TransportError {
    pub(super) const fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout)
    }

    pub(super) const fn is_process_failure(&self) -> bool {
        matches!(
            self,
            Self::Sandbox(_) | Self::EarlyEof | Self::Exit { .. } | Self::ResourceLimit(_)
        )
    }
}
