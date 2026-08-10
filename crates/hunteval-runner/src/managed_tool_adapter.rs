use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
    thread,
    time::{Duration, Instant},
};

use hunteval_domain::{
    ExtensionManifest, ManagedToolAdapterRequest, ManagedToolAdapterResponse, SchemaVersion,
    Sha256Digest,
};
use hunteval_sandbox::{ResolvedExecutionPolicy, SandboxSpec};

const MAX_STDERR_BYTES: usize = 64 * 1024;

pub(crate) fn conform_managed_tool(
    executable: &Path,
    manifest: &ExtensionManifest,
) -> Result<Sha256Digest, &'static str> {
    let tool = manifest
        .tools
        .iter()
        .next()
        .ok_or("managed_tool_inventory")?;
    let request = ManagedToolAdapterRequest {
        schema_version: SchemaVersion::new(0, 9),
        request_id: "conformance-request-001".to_owned(),
        tool: tool.clone(),
        arguments: serde_json::json!({"conformance":true}),
    };
    request.validate().map_err(|_| "managed_tool_request")?;
    let request_bytes = serde_json::to_vec(&request).map_err(|_| "managed_tool_request")?;
    if request_bytes.len() as u64 > manifest.limits.max_input_bytes {
        return Err("managed_tool_input_limit");
    }
    let output_limit = usize::try_from(manifest.limits.max_output_bytes)
        .map_err(|_| "managed_tool_output_limit")?;
    let mut policy = ResolvedExecutionPolicy::hardened_default();
    policy.limits.wall_time_ms = manifest.limits.wall_time_ms;
    policy.limits.cpu_time_seconds = manifest.limits.wall_time_ms.div_ceil(1_000).max(1);
    policy.limits.processes = u64::from(manifest.limits.max_processes);
    policy.limits.stdout_bytes = output_limit.max(256);
    let spec = SandboxSpec {
        executable: executable.to_path_buf(),
        arguments: Vec::new(),
        mounts: Vec::new(),
        working_directory: "/".to_owned(),
        environment: BTreeMap::new(),
        policy,
    };
    let mut child = hunteval_sandbox::spawn(&spec).map_err(|_| "managed_tool_sandbox")?;
    let mut stdin = child
        .take_stdin()
        .map_err(|_| "managed_tool_process_pipe")?;
    let stdout = child
        .take_stdout()
        .map_err(|_| "managed_tool_process_pipe")?;
    let stderr = child
        .take_stderr()
        .map_err(|_| "managed_tool_process_pipe")?;
    let writer = thread::spawn(move || {
        stdin.write_all(&request_bytes)?;
        stdin.flush()
    });
    let output_reader = thread::spawn(move || read_bounded(stdout, output_limit));
    let error_reader = thread::spawn(move || read_bounded(stderr, MAX_STDERR_BYTES));
    let deadline = Duration::from_millis(manifest.limits.wall_time_ms);
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|_| "managed_tool_wait")? {
            break status;
        }
        if started.elapsed() >= deadline {
            child.terminate().map_err(|_| "managed_tool_termination")?;
            return Err("managed_tool_timeout");
        }
        thread::sleep(Duration::from_millis(5));
    };
    writer
        .join()
        .map_err(|_| "managed_tool_process_pipe")?
        .map_err(|_| "managed_tool_process_pipe")?;
    let output = output_reader
        .join()
        .map_err(|_| "managed_tool_process_pipe")?
        .map_err(|_| "managed_tool_output_limit")?;
    let _stderr = error_reader
        .join()
        .map_err(|_| "managed_tool_process_pipe")?
        .map_err(|_| "managed_tool_stderr_limit")?;
    if !status.success() {
        return Err("managed_tool_process_failure");
    }
    let response: ManagedToolAdapterResponse =
        serde_json::from_slice(&output).map_err(|_| "managed_tool_response")?;
    response.validate().map_err(|_| "managed_tool_response")?;
    if response.request_id() != request.request_id {
        return Err("managed_tool_correlation");
    }
    let mut transcript = serde_json::to_vec(&request).map_err(|_| "managed_tool_request")?;
    transcript.push(b'\n');
    transcript.extend_from_slice(&output);
    Ok(Sha256Digest::from_bytes(transcript))
}

fn read_bounded(reader: impl Read, limit: usize) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    let read_limit = limit
        .checked_add(1)
        .ok_or_else(|| std::io::Error::other("adapter output limit is unsupported"))?;
    reader.take(read_limit as u64).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(std::io::Error::other("adapter output exceeded its bound"));
    }
    Ok(bytes)
}
