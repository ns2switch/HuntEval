use hunteval_duckdb::{SqlPolicy, SqlPolicyError};

fn policy() -> SqlPolicy {
    SqlPolicy::new(["aws_cloudtrail".to_owned(), "normalized_events".to_owned()])
}

#[test]
fn accepts_supported_parameterized_select() {
    let result = policy().validate(
        "SELECT event_id, count(*) FROM normalized_events WHERE source_ip = ? GROUP BY event_id ORDER BY event_id LIMIT 10",
        1,
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_mutation_and_multiple_statements() {
    for sql in [
        "DELETE FROM aws_cloudtrail",
        "UPDATE aws_cloudtrail SET event_name = 'x'",
        "SELECT * FROM aws_cloudtrail; SELECT 1",
        "COPY aws_cloudtrail TO '/tmp/leak.csv'",
        "ATTACH '/tmp/other.db' AS other",
        "PRAGMA version",
        "INSTALL httpfs",
        "LOAD httpfs",
    ] {
        assert!(policy().validate(sql, 0).is_err(), "accepted: {sql}");
    }
}

#[test]
fn rejects_file_network_and_table_function_bypasses() {
    for sql in [
        "SELECT * FROM read_parquet('/tmp/private.parquet')",
        "SELECT * FROM range(10)",
        "SELECT read_blob('/etc/passwd')",
        "SELECT http_get('https://example.invalid')",
        "SELECT * FROM information_schema.tables",
        "WITH hidden AS (SELECT * FROM aws_cloudtrail) SELECT * FROM hidden",
        "SELECT * INTO leaked FROM aws_cloudtrail",
    ] {
        assert!(policy().validate(sql, 0).is_err(), "accepted: {sql}");
    }
}

#[test]
fn rejects_unknown_tables_functions_and_parameter_mismatch() {
    assert_eq!(
        policy().validate("SELECT * FROM secrets", 0),
        Err(SqlPolicyError::UnknownTable)
    );
    assert_eq!(
        policy().validate("SELECT current_setting('home_directory')", 0),
        Err(SqlPolicyError::UnknownFunction)
    );
    assert_eq!(
        policy().validate("SELECT * FROM aws_cloudtrail WHERE event_id = ?", 0),
        Err(SqlPolicyError::ParameterCountMismatch)
    );
}
