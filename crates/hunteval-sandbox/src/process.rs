use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{ResolvedExecutionPolicy, command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestMount {
    pub host_path: PathBuf,
    pub guest_path: String,
}

impl GuestMount {
    pub fn read_only(host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        Self {
            host_path: host_path.into(),
            guest_path: guest_path.into(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), SandboxError> {
        validate_safe_guest_path(&self.guest_path, false)?;
        let metadata = std::fs::symlink_metadata(&self.host_path).map_err(|source| {
            SandboxError::Unavailable {
                label: "sandbox mount",
                source,
            }
        })?;
        if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
            return Err(SandboxError::UnsafeFile("sandbox mount"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SandboxSpec {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub mounts: Vec<GuestMount>,
    pub working_directory: String,
    pub environment: BTreeMap<String, String>,
    pub policy: ResolvedExecutionPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitKind {
    CpuTime,
    FileSize,
}

#[must_use]
pub fn classify_exit_status(status: ExitStatus) -> Option<LimitKind> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        match status.signal() {
            Some(libc::SIGXCPU) => Some(LimitKind::CpuTime),
            Some(libc::SIGXFSZ) => Some(LimitKind::FileSize),
            _ => match status.code() {
                Some(code) if code == 128 + libc::SIGXCPU => Some(LimitKind::CpuTime),
                Some(code) if code == 128 + libc::SIGXFSZ => Some(LimitKind::FileSize),
                _ => None,
            },
        }
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}

#[derive(Debug)]
pub struct SupervisedChild {
    child: Child,
}

impl SupervisedChild {
    pub fn take_stdin(&mut self) -> Result<ChildStdin, SandboxError> {
        self.child.stdin.take().ok_or(SandboxError::MissingPipe)
    }

    pub fn take_stdout(&mut self) -> Result<ChildStdout, SandboxError> {
        self.child.stdout.take().ok_or(SandboxError::MissingPipe)
    }

    pub fn take_stderr(&mut self) -> Result<ChildStderr, SandboxError> {
        self.child.stderr.take().ok_or(SandboxError::MissingPipe)
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, SandboxError> {
        self.child.try_wait().map_err(SandboxError::Wait)
    }

    pub fn terminate(&mut self) -> Result<(), SandboxError> {
        match self.child.try_wait().map_err(SandboxError::Wait)? {
            Some(_) => Ok(()),
            None => {
                self.child.kill().map_err(SandboxError::Terminate)?;
                self.child.wait().map_err(SandboxError::Wait)?;
                Ok(())
            }
        }
    }
}

impl Drop for SupervisedChild {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

pub fn spawn(spec: &SandboxSpec) -> Result<SupervisedChild, SandboxError> {
    let arguments = command::arguments(
        &spec.executable,
        &spec.arguments,
        &spec.mounts,
        &spec.working_directory,
        &spec.environment,
        &spec.policy,
    )?;
    let child = Command::new(command::BUBBLEWRAP)
        .args(arguments)
        .current_dir(Path::new("/"))
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SandboxError::Spawn)?;
    Ok(SupervisedChild { child })
}

pub fn validate_safe_guest_path(path: &str, allow_root: bool) -> Result<(), SandboxError> {
    if path.len() > 4096 || !path.starts_with('/') || path.contains('\0') {
        return Err(SandboxError::UnsafeGuestPath);
    }
    let parsed = Path::new(path);
    if (!allow_root && parsed == Path::new("/"))
        || parsed
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(SandboxError::UnsafeGuestPath);
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("sandbox policy is invalid")]
    Policy(#[source] crate::PolicyError),
    #[error("{label} is unavailable")]
    Unavailable {
        label: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("{0} is not a safe regular file or directory")]
    UnsafeFile(&'static str),
    #[error("sandbox paths must be valid UTF-8")]
    NonUtf8Path,
    #[error("sandbox guest path is invalid")]
    UnsafeGuestPath,
    #[error("sandbox environment is invalid")]
    InvalidEnvironment,
    #[error("sandbox process could not be started")]
    Spawn(#[source] std::io::Error),
    #[error("sandbox process pipe is unavailable")]
    MissingPipe,
    #[error("sandbox process state could not be read")]
    Wait(#[source] std::io::Error),
    #[error("sandbox process tree could not be terminated")]
    Terminate(#[source] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::validate_safe_guest_path;

    #[test]
    fn guest_paths_reject_relative_and_traversal_forms() {
        assert!(validate_safe_guest_path("/episode", false).is_ok());
        assert!(validate_safe_guest_path("relative", false).is_err());
        assert!(validate_safe_guest_path("/episode/../private", false).is_err());
        assert!(validate_safe_guest_path("/", false).is_err());
    }
}
