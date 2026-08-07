use std::{fs, path::Path};

use hunteval_duckdb::{DuckDbWorker, QueryLimits, SqlRequest, TableRegistration, ToolErrorCode};
use tempfile::TempDir;

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
fn executable_script(contents: &str) -> Result<(TempDir, std::path::PathBuf), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let directory = TempDir::new()?;
    let path = directory.path().join("worker-stub");
    fs::write(&path, contents)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions)?;
    Ok((directory, path))
}

#[cfg(unix)]
#[test]
fn timeout_kills_worker_without_terminating_runner() -> Result<(), Box<dyn std::error::Error>> {
    let (_directory, executable) = executable_script("#!/bin/sh\nexec sleep 2\n")?;
    let worker = DuckDbWorker::new(executable, vec![fixture_table()]);
    let mut sql = request();
    sql.limits.timeout_ms = 20;
    let error = worker.execute(sql).err();
    assert_eq!(error.map(|value| value.code), Some(ToolErrorCode::Timeout));
    Ok(())
}

#[cfg(unix)]
#[test]
fn crash_and_invalid_output_are_typed_failures() -> Result<(), Box<dyn std::error::Error>> {
    let (_crash_directory, crash) = executable_script("#!/bin/sh\nexit 7\n")?;
    let crashed = DuckDbWorker::new(crash, vec![fixture_table()])
        .execute(request())
        .err();
    assert_eq!(
        crashed.map(|value| value.code),
        Some(ToolErrorCode::WorkerCrashed)
    );

    let (_invalid_directory, invalid) =
        executable_script("#!/bin/sh\ncat >/dev/null\nprintf '{}'")?;
    let malformed = DuckDbWorker::new(invalid, vec![fixture_table()])
        .execute(request())
        .err();
    assert_eq!(
        malformed.map(|value| value.code),
        Some(ToolErrorCode::WorkerProtocol)
    );
    Ok(())
}
