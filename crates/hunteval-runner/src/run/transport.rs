use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use hunteval_protocol::{JsonlDecoder, ProtocolEnvelope};
use thiserror::Error;

const MAX_STDERR_BYTES: usize = 64 * 1024;
const BUBBLEWRAP: &str = "/usr/bin/bwrap";

pub(super) struct ProtocolProcess {
    child: Child,
    input: Option<ChildStdin>,
    lines: Receiver<Result<Vec<u8>, std::io::Error>>,
    stderr: Receiver<Result<Vec<u8>, std::io::Error>>,
    decoder: JsonlDecoder,
    deadline: Instant,
}

impl ProtocolProcess {
    pub(super) fn spawn(
        executable: &Path,
        arguments: &[String],
        working_directory: &Path,
        environment: &BTreeMap<String, String>,
        timeout: Duration,
        maximum_line_bytes: usize,
    ) -> Result<Self, TransportError> {
        let decoder = JsonlDecoder::new(maximum_line_bytes).map_err(TransportError::Protocol)?;
        let read_limit = maximum_line_bytes
            .checked_add(1)
            .ok_or(TransportError::InvalidLimit)?;
        let sandbox_arguments = sandbox_arguments(executable, arguments, working_directory)?;
        let mut child = Command::new(BUBBLEWRAP)
            .args(sandbox_arguments)
            .current_dir(Path::new("/"))
            .env_clear()
            .envs(environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(TransportError::Spawn)?;
        let input = child.stdin.take().ok_or(TransportError::MissingPipe)?;
        let output = child.stdout.take().ok_or(TransportError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(TransportError::MissingPipe)?;
        let (line_sender, lines) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut reader = BufReader::new(output);
            loop {
                let mut line = Vec::new();
                let result = reader
                    .by_ref()
                    .take(read_limit as u64)
                    .read_until(b'\n', &mut line);
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        if line_sender.send(Ok(line)).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = line_sender.send(Err(error));
                        break;
                    }
                }
            }
        });
        let (stderr_sender, stderr_receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = stderr
                .take((MAX_STDERR_BYTES + 1) as u64)
                .read_to_end(&mut bytes)
                .map(|_| bytes);
            let _ = stderr_sender.send(result);
        });
        Ok(Self {
            child,
            input: Some(input),
            lines,
            stderr: stderr_receiver,
            decoder,
            deadline: Instant::now() + timeout,
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
            .map_err(|error| match error {
                mpsc::RecvTimeoutError::Timeout => TransportError::Timeout,
                mpsc::RecvTimeoutError::Disconnected => TransportError::EarlyEof,
            })??;
        self.decoder.decode(&line).map_err(TransportError::Protocol)
    }

    pub(super) fn finish(mut self) -> Result<(), TransportError> {
        drop(self.input.take());
        if Instant::now() >= self.deadline {
            return Err(TransportError::Timeout);
        }
        loop {
            if let Some(status) = self.child.try_wait().map_err(TransportError::Wait)? {
                let diagnostics = self.read_stderr();
                return if status.success() {
                    Ok(())
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
            .map(|mut bytes| {
                bytes.truncate(MAX_STDERR_BYTES);
                String::from_utf8_lossy(&bytes).into_owned()
            })
            .unwrap_or_default()
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn sandbox_arguments(
    executable: &Path,
    arguments: &[String],
    public_root: &Path,
) -> Result<Vec<String>, TransportError> {
    if !Path::new(BUBBLEWRAP).is_file() {
        return Err(TransportError::SandboxUnavailable);
    }
    let executable = executable.to_str().ok_or(TransportError::NonUtf8Path)?;
    let public_root = public_root.to_str().ok_or(TransportError::NonUtf8Path)?;
    let mut sandbox = vec![
        "--unshare-all".to_owned(),
        "--die-with-parent".to_owned(),
        "--new-session".to_owned(),
        "--proc".to_owned(),
        "/proc".to_owned(),
        "--dev".to_owned(),
        "/dev".to_owned(),
        "--tmpfs".to_owned(),
        "/tmp".to_owned(),
    ];
    for system_root in ["/usr", "/bin", "/lib", "/lib64"] {
        if Path::new(system_root).exists() {
            sandbox.extend([
                "--ro-bind".to_owned(),
                system_root.to_owned(),
                system_root.to_owned(),
            ]);
        }
    }
    sandbox.extend([
        "--ro-bind".to_owned(),
        public_root.to_owned(),
        "/episode".to_owned(),
        "--ro-bind".to_owned(),
        executable.to_owned(),
        "/deployment".to_owned(),
        "--chdir".to_owned(),
        "/episode".to_owned(),
        "--".to_owned(),
        "/deployment".to_owned(),
    ]);
    sandbox.extend(arguments.iter().cloned());
    Ok(sandbox)
}

impl Drop for ProtocolProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[derive(Debug, Error)]
pub(super) enum TransportError {
    #[error("the required Linux process sandbox is unavailable")]
    SandboxUnavailable,
    #[error("sandbox paths must be valid UTF-8")]
    NonUtf8Path,
    #[error("deployment protocol line limit is invalid")]
    InvalidLimit,
    #[error("could not start deployment process")]
    Spawn(#[source] std::io::Error),
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
    #[error("could not inspect deployment process")]
    Wait(#[source] std::io::Error),
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
            Self::SandboxUnavailable
                | Self::Spawn(_)
                | Self::EarlyEof
                | Self::Wait(_)
                | Self::Exit { .. }
        )
    }
}
