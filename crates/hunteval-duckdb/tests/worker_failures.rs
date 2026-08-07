use std::path::Path;

use hunteval_duckdb::{DuckDbWorker, QueryLimits, SqlRequest, TableRegistration, ToolErrorCode};

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
        query: "SELECT event_id FROM normalized_events ORDER BY event_id LIMIT 1".to_owned(),
        parameters: Vec::new(),
        limits: QueryLimits::default(),
    }
}

#[test]
fn separate_worker_returns_a_result() -> Result<(), Box<dyn std::error::Error>> {
    let worker = DuckDbWorker::new(
        env!("CARGO_BIN_EXE_hunteval-duckdb-worker"),
        vec![fixture_table()],
    );
    let result = worker.execute(request())?;
    assert_eq!(result.rows.len(), 1);
    Ok(())
}

#[cfg(unix)]
fn shell_worker(command: &str) -> Result<DuckDbWorker, std::io::Error> {
    let executable = std::fs::canonicalize("/bin/sh")?;
    Ok(DuckDbWorker::new(executable, vec![fixture_table()])
        .with_arguments(vec!["-c".into(), command.into()]))
}

#[cfg(unix)]
#[test]
fn timeout_kills_worker_without_terminating_runner() -> Result<(), Box<dyn std::error::Error>> {
    let worker = shell_worker("exec sleep 2")?;
    let mut sql = request();
    sql.limits.timeout_ms = 20;
    let error = worker.execute(sql).err();
    assert_eq!(error.map(|value| value.code), Some(ToolErrorCode::Timeout));
    Ok(())
}

#[cfg(unix)]
#[test]
fn crash_and_invalid_output_are_typed_failures() -> Result<(), Box<dyn std::error::Error>> {
    let crashed = shell_worker("cat >/dev/null; exit 7")?
        .execute(request())
        .err();
    assert_eq!(
        crashed.map(|value| value.code),
        Some(ToolErrorCode::WorkerCrashed)
    );

    let malformed = shell_worker("cat >/dev/null; printf '{}'")?
        .execute(request())
        .err();
    assert_eq!(
        malformed.map(|value| value.code),
        Some(ToolErrorCode::WorkerProtocol)
    );
    Ok(())
}
