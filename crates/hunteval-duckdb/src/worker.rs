use std::{collections::BTreeSet, fs};

use duckdb::{Connection, params_from_iter, types::Value};
use serde::{Deserialize, Serialize};

use crate::{
    SqlParameter, SqlPolicy, SqlRequest, SqlValue, TableRegistration, ToolError, ToolErrorCode,
    ToolResult,
};

/// Internal command sent over the runner-owned worker pipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerCommand {
    pub tables: Vec<TableRegistration>,
    pub request: SqlRequest,
}

/// Internal response returned over the runner-owned worker pipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WorkerResponse {
    Success { result: ToolResult },
    Failure { error: ToolError },
}

/// Executes one validated command inside the short-lived worker process.
pub fn execute_command(command: WorkerCommand) -> Result<ToolResult, ToolError> {
    command.request.validate()?;
    validate_tables(&command.tables)?;
    let allowed_tables = logical_tables(&command.tables);
    SqlPolicy::new(allowed_tables)
        .validate(&command.request.query, command.request.parameters.len())
        .map_err(|_| {
            ToolError::new(
                ToolErrorCode::SqlRejected,
                "SQL query was rejected by the read-only policy",
            )
        })?;

    let connection = prepare_connection(&command)?;
    execute_query(&connection, &command.request)
}

fn validate_tables(tables: &[TableRegistration]) -> Result<(), ToolError> {
    if tables.is_empty() || tables.len() > 16 {
        return Err(ToolError::new(
            ToolErrorCode::InvalidRequest,
            "table registrations exceed a supported bound",
        ));
    }
    let mut names = BTreeSet::new();
    for table in tables {
        table.validate()?;
        if !names.insert(table.name.as_str()) {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "logical table names must be unique",
            ));
        }
        let metadata = fs::symlink_metadata(&table.parquet_path).map_err(|_| {
            ToolError::new(
                ToolErrorCode::InvalidRequest,
                "registered public artifact is unavailable",
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(ToolError::new(
                ToolErrorCode::InvalidRequest,
                "registered public artifact is invalid",
            ));
        }
    }
    Ok(())
}

fn logical_tables(tables: &[TableRegistration]) -> Vec<String> {
    let mut names: Vec<_> = tables.iter().map(|table| table.name.clone()).collect();
    if tables.iter().any(|table| table.name == "aws_cloudtrail") {
        names.push("normalized_events".to_owned());
    }
    names
}

fn prepare_connection(command: &WorkerCommand) -> Result<Connection, ToolError> {
    let connection = Connection::open_in_memory().map_err(query_failed)?;
    let memory = command.request.limits.memory_limit_mb;
    connection
        .execute_batch(&format!(
            "SET memory_limit='{memory}MB'; SET threads=1; SET allow_unsigned_extensions=false; SET autoinstall_known_extensions=false; SET autoload_known_extensions=false;"
        ))
        .map_err(query_failed)?;

    for table in &command.tables {
        let path = table.parquet_path.to_str().ok_or_else(|| {
            ToolError::new(
                ToolErrorCode::InvalidRequest,
                "registered public artifact path is unsupported",
            )
        })?;
        let statement = format!(
            "CREATE TEMP TABLE {} AS SELECT * FROM read_parquet(?)",
            table.name
        );
        connection
            .execute(&statement, [path])
            .map_err(query_failed)?;
    }
    if command
        .tables
        .iter()
        .any(|table| table.name == "aws_cloudtrail")
    {
        connection
            .execute_batch(
                "CREATE TEMP VIEW normalized_events AS SELECT event_id, event_time, provider, account_id, principal, event_name, resource, source_ip, user_agent FROM aws_cloudtrail",
            )
            .map_err(query_failed)?;
    }
    connection
        .execute_batch("SET enable_external_access=false; SET lock_configuration=true;")
        .map_err(query_failed)?;
    Ok(connection)
}

fn execute_query(connection: &Connection, request: &SqlRequest) -> Result<ToolResult, ToolError> {
    let mut statement = connection.prepare(&request.query).map_err(query_failed)?;
    let parameters: Vec<_> = request.parameters.iter().map(to_duckdb_value).collect();
    let mut result_rows = statement
        .query(params_from_iter(parameters.iter()))
        .map_err(query_failed)?;
    let columns = result_rows
        .as_ref()
        .ok_or_else(|| {
            ToolError::new(
                ToolErrorCode::QueryFailed,
                "managed SQL query returned no statement metadata",
            )
        })?
        .column_names();
    let column_count = columns.len();
    let mut result = ToolResult {
        columns,
        rows: Vec::new(),
        truncated: false,
    };

    while let Some(row) = result_rows.next().map_err(query_failed)? {
        if result.rows.len() == request.limits.max_rows {
            result.truncated = true;
            break;
        }
        let values = (0..column_count)
            .map(|index| {
                row.get::<_, Value>(index)
                    .map_err(query_failed)
                    .and_then(to_sql_value)
            })
            .collect::<Result<Vec<_>, _>>()?;
        result.rows.push(values);
        if serialized_size(&result)? > request.limits.max_output_bytes {
            result.rows.pop();
            result.truncated = true;
            break;
        }
    }
    if serialized_size(&result)? > request.limits.max_output_bytes {
        return Err(ToolError::new(
            ToolErrorCode::OutputLimit,
            "query metadata exceeds the output byte limit",
        ));
    }
    Ok(result)
}

fn to_duckdb_value(parameter: &SqlParameter) -> Value {
    match parameter {
        SqlParameter::Null => Value::Null,
        SqlParameter::Boolean(value) => Value::Boolean(*value),
        SqlParameter::Integer(value) => Value::BigInt(*value),
        SqlParameter::Float(value) => Value::Double(*value),
        SqlParameter::String(value) => Value::Text(value.clone()),
    }
}

fn to_sql_value(value: Value) -> Result<SqlValue, ToolError> {
    let converted = match value {
        Value::Null => SqlValue::Null,
        Value::Boolean(value) => SqlValue::Boolean(value),
        Value::TinyInt(value) => SqlValue::Integer(i64::from(value)),
        Value::SmallInt(value) => SqlValue::Integer(i64::from(value)),
        Value::Int(value) => SqlValue::Integer(i64::from(value)),
        Value::BigInt(value) => SqlValue::Integer(value),
        Value::HugeInt(value) => SqlValue::String(value.to_string()),
        Value::UTinyInt(value) => SqlValue::Integer(i64::from(value)),
        Value::USmallInt(value) => SqlValue::Integer(i64::from(value)),
        Value::UInt(value) => SqlValue::Integer(i64::from(value)),
        Value::UBigInt(value) => i64::try_from(value)
            .map(SqlValue::Integer)
            .unwrap_or_else(|_| SqlValue::String(value.to_string())),
        Value::Float(value) if value.is_finite() => SqlValue::Float(f64::from(value)),
        Value::Double(value) if value.is_finite() => SqlValue::Float(value),
        Value::Decimal(value) => SqlValue::String(value.to_string()),
        Value::Text(value) => SqlValue::String(value),
        _ => {
            return Err(ToolError::new(
                ToolErrorCode::QueryFailed,
                "query returned an unsupported value type",
            ));
        }
    };
    Ok(converted)
}

fn serialized_size(result: &ToolResult) -> Result<usize, ToolError> {
    serde_json::to_vec(result)
        .map(|bytes| bytes.len())
        .map_err(|_| {
            ToolError::new(
                ToolErrorCode::QueryFailed,
                "query result could not be serialized",
            )
        })
}

fn query_failed(_error: duckdb::Error) -> ToolError {
    ToolError::new(
        ToolErrorCode::QueryFailed,
        "managed SQL query execution failed",
    )
}
