use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use hunteval_domain::{EventId, ProtocolVersion, RunId, UtcTimestamp};
use hunteval_runner::{
    ManagedTool, ManagedToolError, ManagedToolOutput, ResolvedRunInputs, RunExecutor,
    RunFailureKind, RunRequest, VerificationStatus, inspect_trajectory, verify_run,
};

struct ObservableTool {
    calls: AtomicUsize,
}

#[cfg(unix)]
#[test]
fn generic_engine_preserves_bounded_partial_failures_and_hides_private_data()
-> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    if !Path::new("/usr/bin/bwrap").is_file() {
        return Ok(());
    }
    let root = workspace_root()?;
    let temporary = tempfile::tempdir()?;
    let deployment_root = temporary.path().join("failure-deployment");
    fs::create_dir_all(deployment_root.join("bin"))?;
    let tool = ObservableTool {
        calls: AtomicUsize::new(0),
    };
    let cases = [
        (
            "malformed",
            "#!/bin/sh\nIFS= read -r started\nprintf '{malformed}\\n'\n",
            RunFailureKind::ProtocolViolation,
            Duration::from_secs(2),
        ),
        (
            "crash",
            "#!/bin/sh\nIFS= read -r started\nexit 9\n",
            RunFailureKind::ProcessCrash,
            Duration::from_secs(2),
        ),
        (
            "early-eof",
            "#!/bin/sh\nexit 0\n",
            RunFailureKind::ProcessCrash,
            Duration::from_secs(2),
        ),
        (
            "invalid-utf8",
            "#!/bin/sh\nIFS= read -r started\nprintf '\\377\\n'\n",
            RunFailureKind::ProtocolViolation,
            Duration::from_secs(2),
        ),
        (
            "oversized-frame",
            "#!/bin/sh\nIFS= read -r started\nhead -c 5000 /dev/zero | tr '\\0' a; printf '\\n'\n",
            RunFailureKind::ProtocolViolation,
            Duration::from_secs(2),
        ),
        (
            "slow-writer",
            "#!/bin/sh\nIFS= read -r started\nsleep 2\n",
            RunFailureKind::Timeout,
            Duration::from_millis(100),
        ),
        (
            "descendant-pipe-holder",
            "#!/bin/sh\nIFS= read -r started\nsleep 30 & wait\n",
            RunFailureKind::Timeout,
            Duration::from_millis(100),
        ),
        (
            "file-limit",
            "#!/bin/sh\nIFS= read -r started\nexec dd if=/dev/zero of=/tmp/too-large bs=1048576 count=20 2>/dev/null\n",
            RunFailureKind::ResourceLimit,
            Duration::from_secs(2),
        ),
        (
            "private-denial",
            "#!/bin/sh\nif [ -e /root/hunteval/datasets/aws/aws-iam-002/private/ground-truth.json ]; then exit 9; fi\nIFS= read -r started\nsleep 2\n",
            RunFailureKind::Timeout,
            Duration::from_millis(100),
        ),
    ];

    for (name, script, expected, timeout) in cases {
        let executable = deployment_root.join("bin/peer");
        fs::write(&executable, script)?;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))?;
        let deployment = deployment_root.join("deployment.yaml");
        fs::write(&deployment, failure_deployment_yaml())?;
        let inputs = ResolvedRunInputs::resolve(
            &root.join("datasets/aws/aws-iam-002"),
            &deployment,
            &root.join("examples/scoring-profile-balanced.yaml"),
            &root.join("schemas/v0.4/submission.schema.json"),
        )?;
        let failure = match RunExecutor.execute(
            &RunRequest {
                run_id: RunId::new(format!("failure-{name}"))?,
                seed: 7,
                output_root: temporary.path().join("failed-runs"),
                started_at: serde_json::from_str::<UtcTimestamp>("\"2026-08-07T00:00:00Z\"")?,
                protocol_version: ProtocolVersion::new(0, 3),
                timeout,
                maximum_line_bytes: 4096,
            },
            &inputs,
            &tool,
        ) {
            Ok(_) => return Err(std::io::Error::other("failure fixture completed").into()),
            Err(failure) => failure,
        };
        assert_eq!(failure.kind, expected);
        assert!(failure.partial_artifacts.join("trajectory.jsonl").is_file());
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(failure.partial_artifacts.join("manifest.json"))?)?;
        assert_eq!(manifest["partial"], true);
        assert_eq!(
            verify_run(&failure.partial_artifacts).status,
            VerificationStatus::Incomplete
        );
    }

    let constrained_episode = temporary.path().join("constrained-episode");
    copy_episode(&root.join("datasets/aws/aws-iam-002"), &constrained_episode)?;
    let public_manifest = constrained_episode.join("public/manifest.yaml");
    let manifest = fs::read_to_string(&public_manifest)?;
    fs::write(
        &public_manifest,
        manifest.replace("max_messages: 40", "max_messages: 1"),
    )?;
    fs::copy(
        env!("CARGO_BIN_EXE_hunteval-reference-deployment"),
        deployment_root.join("bin/hunteval-reference-deployment"),
    )?;
    let deployment = deployment_root.join("deployment.yaml");
    fs::write(&deployment, deployment_yaml("single-agent", "single_agent"))?;
    let inputs = ResolvedRunInputs::resolve(
        &constrained_episode,
        &deployment,
        &root.join("examples/scoring-profile-balanced.yaml"),
        &root.join("schemas/v0.4/submission.schema.json"),
    )?;
    let budget_failure = match RunExecutor.execute(
        &RunRequest {
            run_id: RunId::new("failure-budget")?,
            seed: 7,
            output_root: temporary.path().join("failed-runs"),
            started_at: serde_json::from_str::<UtcTimestamp>("\"2026-08-07T00:00:00Z\"")?,
            protocol_version: ProtocolVersion::new(0, 3),
            timeout: Duration::from_secs(2),
            maximum_line_bytes: 4096,
        },
        &inputs,
        &tool,
    ) {
        Ok(_) => return Err(std::io::Error::other("budget fixture completed").into()),
        Err(failure) => failure,
    };
    assert_eq!(budget_failure.kind, RunFailureKind::BudgetExceeded);
    assert!(
        budget_failure
            .partial_artifacts
            .join("trajectory.jsonl")
            .is_file()
    );
    assert_eq!(tool.calls.load(Ordering::SeqCst), 0);
    Ok(())
}

impl ManagedTool for ObservableTool {
    fn execute(
        &self,
        tool: &str,
        arguments: &serde_json::Value,
    ) -> Result<ManagedToolOutput, ManagedToolError> {
        if tool != "duckdb_sql"
            || arguments["parameters"][0]["value"] != "203.0.113.77"
            || arguments["query"].as_str().is_none_or(|query| {
                query.contains("evt-0004") || query.contains("ground") || query.contains("private")
            })
        {
            return Err(ManagedToolError::InvalidRequest(
                "reference query was not observation-driven".to_owned(),
            ));
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ManagedToolOutput {
            event_ids: ["evt-0004", "evt-0005", "evt-0006"]
                .into_iter()
                .map(EventId::new)
                .collect::<Result<_, _>>()
                .map_err(|_| ManagedToolError::Execution("fixture identifier failed".to_owned()))?,
            result: serde_json::json!({
                "columns": ["event_id", "principal", "event_time", "event_name"],
                "rows": [
                    string_row("evt-0004", "suspected-identity", "AssumeAdmin"),
                    string_row("evt-0005", "suspected-identity", "GrantPrivilege"),
                    string_row("evt-0006", "suspected-identity", "CreateCredential")
                ],
                "truncated": false
            }),
        })
    }
}

fn string_row(event: &str, principal: &str, name: &str) -> serde_json::Value {
    serde_json::json!([
        {"type": "string", "value": event},
        {"type": "string", "value": principal},
        {"type": "string", "value": "2026-01-01T00:03:00Z"},
        {"type": "string", "value": name}
    ])
}

#[test]
fn generic_engine_mediates_every_provider_and_topology() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("/usr/bin/bwrap").is_file() {
        return Ok(());
    }
    let root = workspace_root()?;
    let temporary = tempfile::tempdir()?;
    let deployment_root = temporary.path().join("deployment");
    fs::create_dir_all(deployment_root.join("bin"))?;
    fs::copy(
        env!("CARGO_BIN_EXE_hunteval-reference-deployment"),
        deployment_root.join("bin/hunteval-reference-deployment"),
    )?;
    let tool = ObservableTool {
        calls: AtomicUsize::new(0),
    };
    let mut completed = 0_usize;
    let mut expected_calls = 0_usize;

    for provider in ["aws", "azure", "gcp"] {
        for (topology, architecture) in [
            ("single-agent", "single_agent"),
            ("supervisor-worker", "supervisor_worker"),
            ("supervisor-specialists", "hierarchical"),
        ] {
            let deployment = deployment_root.join("deployment.yaml");
            fs::write(&deployment, deployment_yaml(topology, architecture))?;
            let inputs = ResolvedRunInputs::resolve(
                &root.join(format!(
                    "datasets/{provider}/{provider}-iam-{}",
                    if provider == "aws" { "002" } else { "001" }
                )),
                &deployment,
                &root.join("examples/scoring-profile-balanced.yaml"),
                &root.join("schemas/v0.4/submission.schema.json"),
            )?;
            let run_id = RunId::new(format!("run-{provider}-{}", topology.replace('-', "")))?;
            let request = RunRequest {
                run_id,
                seed: 42,
                output_root: temporary.path().join("runs"),
                started_at: serde_json::from_str::<UtcTimestamp>("\"2026-08-07T00:00:00Z\"")?,
                protocol_version: ProtocolVersion::new(0, 3),
                timeout: Duration::from_secs(5),
                maximum_line_bytes: 128 * 1024,
            };
            let execution = RunExecutor.execute(&request, &inputs, &tool)?;
            expected_calls += 1;
            assert_eq!(execution.submission.malicious_event_ids.len(), 3);
            assert!(execution.artifacts.root.join("trajectory.jsonl").is_file());
            assert!(
                execution
                    .artifacts
                    .hashes
                    .contains_key("deployment_executable")
            );
            assert_eq!(execution.metrics.0["event_recall"].value, Some(1.0));
            assert!(execution.aggregate_score.value.is_some());
            let trajectory = fs::read(execution.artifacts.root.join("trajectory.jsonl"))?;
            let (event_count, digest) = inspect_trajectory(&trajectory)?;
            assert_eq!(event_count, 13);
            assert_eq!(execution.artifacts.hashes.get("trajectory"), Some(&digest));

            if provider == "aws" && topology == "single-agent" {
                let mut repeated_request = request.clone();
                repeated_request.output_root = temporary.path().join("repeated-runs");
                let repeated = RunExecutor.execute(&repeated_request, &inputs, &tool)?;
                expected_calls += 1;
                for artifact in ["trajectory", "submission"] {
                    assert_eq!(
                        execution.artifacts.hashes.get(artifact),
                        repeated.artifacts.hashes.get(artifact)
                    );
                }
                let mut first_metrics = execution.metrics.clone();
                let mut repeated_metrics = repeated.metrics.clone();
                first_metrics.0.remove("measured_duration_utilization");
                repeated_metrics.0.remove("measured_duration_utilization");
                assert_eq!(first_metrics, repeated_metrics);
            }
            completed += 1;
        }
    }
    assert_eq!(completed, 9);
    assert_eq!(tool.calls.load(Ordering::SeqCst), expected_calls);
    Ok(())
}

fn deployment_yaml(topology: &str, architecture: &str) -> String {
    format!(
        "schema_version: '0.4'\nid: reference-test\nkind: external_reference_process\narchitecture: {architecture}\nagents:\n  - id: investigator\n    role: investigator\nnetwork_access: false\nscored_tools: hunteval_managed_only\nprocess:\n  executable: bin/hunteval-reference-deployment\n  arguments: ['--topology', '{topology}']\n  environment_allowlist: []\n"
    )
}

fn failure_deployment_yaml() -> &'static str {
    "schema_version: '0.4'\nid: failure-test\nkind: external_reference_process\narchitecture: single_agent\nagents:\n  - id: investigator\n    role: investigator\nnetwork_access: false\nscored_tools: hunteval_managed_only\nprocess:\n  executable: bin/peer\n  arguments: []\n  environment_allowlist: []\n"
}

fn workspace_root() -> Result<PathBuf, std::io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| std::io::Error::other("workspace root is unavailable"))
}

fn copy_episode(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    for relative in [
        "package.yaml",
        "public/manifest.yaml",
        "public/telemetry/cloudtrail.parquet",
        "private/ground-truth.json",
    ] {
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source.join(relative), target)?;
    }
    Ok(())
}
