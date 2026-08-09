use std::path::Path;

use crate::{GuestMount, ResolvedExecutionPolicy, SandboxError};

pub(crate) const BUBBLEWRAP: &str = "/usr/bin/bwrap";
pub(crate) const PRLIMIT: &str = "/usr/bin/prlimit";
const GUEST_EXECUTABLE: &str = "/hunteval/program";

pub(crate) fn arguments(
    executable: &Path,
    program_arguments: &[String],
    mounts: &[GuestMount],
    working_directory: &str,
    environment: &std::collections::BTreeMap<String, String>,
    policy: &ResolvedExecutionPolicy,
) -> Result<Vec<String>, SandboxError> {
    policy.validate().map_err(SandboxError::Policy)?;
    require_regular_file(Path::new(BUBBLEWRAP), "sandbox backend")?;
    require_regular_file(Path::new(PRLIMIT), "resource-limit launcher")?;
    require_regular_file(executable, "sandbox executable")?;
    crate::validate_safe_guest_path(working_directory, true)?;

    let mut output = base_arguments();
    for root in ["/usr", "/bin", "/lib", "/lib64"] {
        if Path::new(root).exists() {
            output.extend(["--ro-bind".to_owned(), root.to_owned(), root.to_owned()]);
        }
    }
    output.extend([
        "--dir".to_owned(),
        "/hunteval".to_owned(),
        "--ro-bind".to_owned(),
        path_text(executable)?.to_owned(),
        GUEST_EXECUTABLE.to_owned(),
    ]);
    for mount in mounts {
        mount.validate()?;
        output.extend([
            "--ro-bind".to_owned(),
            path_text(&mount.host_path)?.to_owned(),
            mount.guest_path.clone(),
        ]);
    }
    for (key, value) in environment {
        validate_environment(key, value)?;
        output.extend(["--setenv".to_owned(), key.clone(), value.clone()]);
    }
    output.extend([
        "--chdir".to_owned(),
        working_directory.to_owned(),
        "--".to_owned(),
        PRLIMIT.to_owned(),
    ]);
    let limits = policy.limits;
    output.extend([
        format!("--cpu={}", limits.cpu_time_seconds),
        format!("--as={}", limits.address_space_bytes),
        format!("--fsize={}", limits.file_size_bytes),
        format!("--nofile={}", limits.open_files),
        format!("--nproc={}", limits.processes),
        "--core=0".to_owned(),
        "--".to_owned(),
        GUEST_EXECUTABLE.to_owned(),
    ]);
    output.extend(program_arguments.iter().cloned());
    Ok(output)
}

fn base_arguments() -> Vec<String> {
    [
        "--unshare-all",
        "--unshare-user",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn require_regular_file(path: &Path, label: &'static str) -> Result<(), SandboxError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|source| SandboxError::Unavailable { label, source })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(SandboxError::UnsafeFile(label));
    }
    Ok(())
}

fn path_text(path: &Path) -> Result<&str, SandboxError> {
    path.to_str().ok_or(SandboxError::NonUtf8Path)
}

fn validate_environment(key: &str, value: &str) -> Result<(), SandboxError> {
    let key_valid = !key.is_empty()
        && key.len() <= 128
        && key
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        && !key.bytes().next().is_some_and(|byte| byte.is_ascii_digit());
    if !key_valid || value.len() > 16 * 1024 || value.contains('\0') {
        return Err(SandboxError::InvalidEnvironment);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{base_arguments, validate_environment};

    #[test]
    fn base_arguments_use_the_supported_isolation_baseline() {
        let arguments = base_arguments();

        for required in [
            "--unshare-all",
            "--unshare-user",
            "--die-with-parent",
            "--new-session",
            "--clearenv",
        ] {
            assert!(arguments.iter().any(|argument| argument == required));
        }
        assert!(
            !arguments
                .iter()
                .any(|argument| argument == "--disable-userns")
        );
    }

    #[test]
    fn environment_validation_is_bounded() {
        assert!(validate_environment("SAFE_NAME", "value").is_ok());
        assert!(validate_environment("1BAD", "value").is_err());
        assert!(validate_environment("BAD-NAME", "value").is_err());
        assert!(validate_environment("SAFE", "bad\0value").is_err());
    }
}
