use std::path::Path;

use hunteval_duckdb::{
    QueryLimits, SqlParameter, SqlRequest, SqlValue, TableRegistration, WorkerCommand,
    execute_command,
};

fn fixture_table() -> TableRegistration {
    TableRegistration {
        name: "aws_cloudtrail".to_owned(),
        parquet_path: Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("datasets/aws/aws-iam-001/public/telemetry/cloudtrail.parquet"),
    }
}

fn request(query: &str) -> SqlRequest {
    SqlRequest {
        query: query.to_owned(),
        parameters: Vec::new(),
        limits: QueryLimits::default(),
    }
}

#[test]
fn reference_query_recovers_expected_public_rows() -> Result<(), Box<dyn std::error::Error>> {
    let mut sql =
        request("SELECT event_id FROM normalized_events WHERE source_ip = ? ORDER BY event_id");
    sql.parameters = vec![SqlParameter::String("203.0.113.77".to_owned())];
    let result = execute_command(WorkerCommand {
        tables: vec![fixture_table()],
        request: sql,
    })?;

    assert_eq!(result.columns, ["event_id"]);
    assert_eq!(
        result.rows,
        [
            vec![SqlValue::String("evt-0004".to_owned())],
            vec![SqlValue::String("evt-0005".to_owned())],
            vec![SqlValue::String("evt-0006".to_owned())]
        ]
    );
    assert!(!result.truncated);
    Ok(())
}

#[test]
fn row_and_byte_limits_truncate_results() -> Result<(), Box<dyn std::error::Error>> {
    let mut row_limited = request("SELECT event_id FROM normalized_events ORDER BY event_id");
    row_limited.limits.max_rows = 2;
    let rows = execute_command(WorkerCommand {
        tables: vec![fixture_table()],
        request: row_limited,
    })?;
    assert_eq!(rows.rows.len(), 2);
    assert!(rows.truncated);

    let mut byte_limited = request(
        "SELECT event_id, principal, event_name, resource, source_ip, user_agent FROM normalized_events ORDER BY event_id",
    );
    byte_limited.limits.max_output_bytes = 256;
    let bytes = execute_command(WorkerCommand {
        tables: vec![fixture_table()],
        request: byte_limited,
    })?;
    assert!(bytes.rows.len() < 10);
    assert!(bytes.truncated);
    Ok(())
}

#[test]
fn result_serialization_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    let command = || WorkerCommand {
        tables: vec![fixture_table()],
        request: request(
            "SELECT event_id, event_name FROM normalized_events ORDER BY event_id LIMIT 3",
        ),
    };
    let first = serde_json::to_vec(&execute_command(command())?)?;
    let second = serde_json::to_vec(&execute_command(command())?)?;
    assert_eq!(first, second);
    Ok(())
}
