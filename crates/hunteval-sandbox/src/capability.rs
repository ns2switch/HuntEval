use std::{
    collections::BTreeMap,
    io::Read,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{GuestMount, ResolvedExecutionPolicy, SandboxSpec, spawn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxRequirement {
    Namespaces,
    ReadOnlyMounts,
    NetworkDenied,
    ProcessTreeTermination,
    ResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxCapability {
    pub requirement: SandboxRequirement,
    pub available: bool,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxCapabilityReport {
    pub schema_version: String,
    pub backend: String,
    pub supported: bool,
    pub capabilities: Vec<SandboxCapability>,
}

pub fn probe_linux_sandbox() -> SandboxCapabilityReport {
    let boundary = boundary_probe();
    let tree = process_tree_probe();
    let resources = resource_probe();
    let outcomes = [boundary, boundary, boundary, tree, resources];
    let requirements = [
        SandboxRequirement::Namespaces,
        SandboxRequirement::ReadOnlyMounts,
        SandboxRequirement::NetworkDenied,
        SandboxRequirement::ProcessTreeTermination,
        SandboxRequirement::ResourceLimits,
    ];
    let capabilities = requirements
        .into_iter()
        .zip(outcomes)
        .map(|(requirement, available)| SandboxCapability {
            requirement,
            available,
            reason_code: (!available).then(|| "probe_failed".to_owned()),
        })
        .collect::<Vec<_>>();
    SandboxCapabilityReport {
        schema_version: "0.5".to_owned(),
        backend: "linux_bubblewrap".to_owned(),
        supported: capabilities.iter().all(|item| item.available),
        capabilities,
    }
}

fn boundary_probe() -> bool {
    let script = "! touch /probe/hunteval-write-probe 2>/dev/null && test \"$(wc -l </proc/net/route)\" -le 1";
    run_to_status(
        shell_spec(
            vec!["-c".to_owned(), script.to_owned()],
            default_probe_policy(),
        ),
        true,
    )
}

fn resource_probe() -> bool {
    let mut policy = default_probe_policy();
    policy.limits.file_size_bytes = 1024;
    let script = "dd if=/dev/zero of=/tmp/limit-probe bs=2048 count=1 2>/dev/null";
    run_to_status(
        shell_spec(vec!["-c".to_owned(), script.to_owned()], policy),
        false,
    )
}

fn process_tree_probe() -> bool {
    let spec = shell_spec(
        vec!["-c".to_owned(), "sleep 30 & wait".to_owned()],
        default_probe_policy(),
    );
    let Ok(mut child) = spawn(&spec) else {
        return false;
    };
    let Ok(stdout) = child.take_stdout() else {
        return false;
    };
    let Ok(stderr) = child.take_stderr() else {
        return false;
    };
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut output = Vec::new();
        let stdout_result = stdout.take(1024).read_to_end(&mut output);
        let stderr_result = stderr.take(1024).read_to_end(&mut output);
        let _ = sender.send(stdout_result.is_ok() && stderr_result.is_ok());
    });
    thread::sleep(Duration::from_millis(20));
    child.terminate().is_ok()
        && receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap_or(false)
}

fn run_to_status(spec: SandboxSpec, expected_success: bool) -> bool {
    let wall_time = spec.policy.limits.wall_time();
    let Ok(mut child) = spawn(&spec) else {
        return false;
    };
    let Ok(stderr) = child.take_stderr() else {
        return false;
    };
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.take(4096).read_to_end(&mut bytes)
    });
    let deadline = Instant::now() + wall_time;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let _ = stderr_reader.join();
                return status.success() == expected_success;
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(2)),
            _ => {
                let _ = child.terminate();
                let _ = stderr_reader.join();
                return false;
            }
        }
    }
}

fn shell_spec(arguments: Vec<String>, policy: ResolvedExecutionPolicy) -> SandboxSpec {
    let executable = PathBuf::from("/bin/sh")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("/bin/sh"));
    SandboxSpec {
        executable,
        arguments,
        mounts: vec![GuestMount::read_only("/usr", "/probe")],
        working_directory: "/tmp".to_owned(),
        environment: BTreeMap::new(),
        policy,
    }
}

fn default_probe_policy() -> ResolvedExecutionPolicy {
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.wall_time_ms = 2_000;
    policy.limits.cpu_time_seconds = 2;
    policy.limits.address_space_bytes = 256 * 1024 * 1024;
    policy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_a_stable_complete_requirement_inventory() {
        let report = probe_linux_sandbox();
        assert_eq!(report.schema_version, "0.5");
        assert_eq!(report.capabilities.len(), 5);
        assert_eq!(
            report.supported,
            report.capabilities.iter().all(|item| item.available)
        );
    }
}
