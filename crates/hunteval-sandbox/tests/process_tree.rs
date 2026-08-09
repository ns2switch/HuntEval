use std::{collections::BTreeMap, io::Read, path::PathBuf, sync::mpsc, thread, time::Duration};

use hunteval_sandbox::{
    LimitKind, ResolvedExecutionPolicy, SandboxSpec, classify_exit_status, probe_linux_sandbox,
    spawn,
};

#[test]
fn termination_closes_descendant_pipe_holders() -> Result<(), Box<dyn std::error::Error>> {
    if !probe_linux_sandbox().supported {
        return Ok(());
    }
    let executable = PathBuf::from("/bin/sh").canonicalize()?;
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.wall_time_ms = 2_000;
    let spec = SandboxSpec {
        executable,
        arguments: vec!["-c".to_owned(), "sleep 30 & wait".to_owned()],
        mounts: Vec::new(),
        working_directory: "/tmp".to_owned(),
        environment: BTreeMap::new(),
        policy,
    };
    let mut child = spawn(&spec)?;
    let stdout = child.take_stdout()?;
    let stderr = child.take_stderr()?;
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let stdout_closed = stdout.take(1024).read_to_end(&mut bytes).is_ok();
        let stderr_closed = stderr.take(1024).read_to_end(&mut bytes).is_ok();
        let _ = sender.send(stdout_closed && stderr_closed);
    });
    thread::sleep(Duration::from_millis(30));
    child.terminate()?;
    assert_eq!(receiver.recv_timeout(Duration::from_secs(1)), Ok(true));
    Ok(())
}

#[test]
fn file_size_limit_is_enforced_by_the_operating_system() -> Result<(), Box<dyn std::error::Error>> {
    if !probe_linux_sandbox().supported {
        return Ok(());
    }
    let executable = PathBuf::from("/bin/sh").canonicalize()?;
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.file_size_bytes = 1024;
    policy.limits.wall_time_ms = 2_000;
    let spec = SandboxSpec {
        executable,
        arguments: vec![
            "-c".to_owned(),
            "exec dd if=/dev/zero of=/tmp/too-large bs=2048 count=1 2>/dev/null".to_owned(),
        ],
        mounts: Vec::new(),
        working_directory: "/tmp".to_owned(),
        environment: BTreeMap::new(),
        policy,
    };
    let mut child = spawn(&spec)?;
    drop(child.take_stdin()?);
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(status) = child.try_wait()? {
            assert!(!status.success());
            assert_eq!(classify_exit_status(status), Some(LimitKind::FileSize));
            break;
        }
        if std::time::Instant::now() >= deadline {
            child.terminate()?;
            return Err(std::io::Error::other("resource probe timed out").into());
        }
        thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}
