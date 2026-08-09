//! Shared fail-closed process isolation for untrusted HuntEval components.

mod capability;
mod command;
mod policy;
mod process;
mod redaction;
mod secret_scan;

pub use capability::{
    SandboxCapability, SandboxCapabilityReport, SandboxRequirement, probe_linux_sandbox,
};
pub use policy::{
    NetworkPolicy, PolicyError, ResolvedExecutionPolicy, ResourceLimits, SandboxBackendId,
};
pub use process::{
    GuestMount, LimitKind, SandboxError, SandboxSpec, SupervisedChild, classify_exit_status, spawn,
    validate_safe_guest_path,
};
pub use redaction::{RedactedText, RedactionError, RedactionPolicy, Redactor};
pub use secret_scan::{
    SecretScanFinding, SecretScanPolicy, SecretScanResult, SecretScanStatus, scan_paths,
};

/// Returns the exact supported sandbox backend path.
#[must_use]
pub fn backend_executable() -> &'static std::path::Path {
    std::path::Path::new(command::BUBBLEWRAP)
}

/// Returns the exact supported resource-limit launcher path.
#[must_use]
pub fn resource_launcher_executable() -> &'static std::path::Path {
    std::path::Path::new(command::PRLIMIT)
}
