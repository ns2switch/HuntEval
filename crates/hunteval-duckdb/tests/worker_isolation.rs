use std::path::Path;

use hunteval_duckdb::{DuckDbWorker, QueryLimits, SqlRequest, TableRegistration, ToolErrorCode};

#[cfg(unix)]
#[test]
fn worker_cannot_observe_workspace_or_host_network_routes() -> Result<(), Box<dyn std::error::Error>>
{
    let executable = std::fs::canonicalize("/bin/sh")?;
    let command = "cat >/dev/null; if test -e /root/hunteval/Cargo.toml || test \"$(wc -l </proc/net/route)\" -gt 1; then exit 9; else printf '{}'; fi";
    let worker = DuckDbWorker::new(executable, vec![fixture_table()])
        .with_arguments(vec!["-c".to_owned(), command.to_owned()]);
    let error = worker.execute(request()).err();
    assert_eq!(
        error.map(|value| value.code),
        Some(ToolErrorCode::WorkerProtocol)
    );
    Ok(())
}

fn fixture_table() -> TableRegistration {
    TableRegistration {
        name: "aws_cloudtrail".to_owned(),
        parquet_path: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("datasets/aws/aws-iam-001/public/telemetry/cloudtrail.parquet"),
    }
}

fn request() -> SqlRequest {
    SqlRequest {
        query: "SELECT event_id FROM normalized_events LIMIT 1".to_owned(),
        parameters: Vec::new(),
        limits: QueryLimits::default(),
    }
}
