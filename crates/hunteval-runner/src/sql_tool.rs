use std::{collections::BTreeSet, path::Path};

use hunteval_domain::EventId;
use hunteval_duckdb::{DuckDbWorker, SqlRequest, SqlValue, TableRegistration};

use crate::{ManagedTool, ManagedToolError, ManagedToolOutput, PublicEpisodePackage};

/// Production adapter exposing only public episode telemetry to the managed SQL worker.
#[derive(Debug, Clone)]
pub struct DuckDbManagedTool {
    worker: DuckDbWorker,
}

impl DuckDbManagedTool {
    /// Creates a worker adapter from the deployment-safe episode view.
    pub fn new(worker_executable: &Path, episode: &PublicEpisodePackage) -> Self {
        let tables = episode
            .manifest
            .telemetry
            .tables
            .iter()
            .map(|table| TableRegistration {
                name: table.name.clone(),
                parquet_path: episode.public_root.join(&table.path),
            })
            .collect();
        Self {
            worker: DuckDbWorker::new(worker_executable, tables),
        }
    }
}

impl ManagedTool for DuckDbManagedTool {
    fn execute(
        &self,
        tool: &str,
        arguments: &serde_json::Value,
    ) -> Result<ManagedToolOutput, ManagedToolError> {
        if tool != "duckdb_sql" {
            return Err(ManagedToolError::UnknownTool);
        }
        let request: SqlRequest = serde_json::from_value(arguments.clone())
            .map_err(|_| ManagedToolError::InvalidRequest("invalid SQL request".to_owned()))?;
        let result = self
            .worker
            .execute(request)
            .map_err(|error| ManagedToolError::Execution(error.to_string()))?;
        let event_ids = extract_event_ids(&result.columns, &result.rows)?;
        let result = serde_json::to_value(result)
            .map_err(|_| ManagedToolError::Execution("invalid worker result".to_owned()))?;
        Ok(ManagedToolOutput { event_ids, result })
    }
}

fn extract_event_ids(
    columns: &[String],
    rows: &[Vec<SqlValue>],
) -> Result<BTreeSet<EventId>, ManagedToolError> {
    let Some(index) = columns.iter().position(|column| column == "event_id") else {
        return Ok(BTreeSet::new());
    };
    rows.iter()
        .filter_map(|row| row.get(index))
        .map(|value| match value {
            SqlValue::String(value) => EventId::new(value.clone()).map_err(|_| {
                ManagedToolError::Execution("worker returned an invalid event id".to_owned())
            }),
            _ => Err(ManagedToolError::Execution(
                "worker returned a non-string event id".to_owned(),
            )),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use hunteval_duckdb::SqlValue;

    use super::extract_event_ids;

    #[test]
    fn derives_unique_event_provenance_from_worker_rows() -> Result<(), Box<dyn std::error::Error>>
    {
        let ids = extract_event_ids(
            &["event_id".to_owned(), "actor".to_owned()],
            &[
                vec![SqlValue::String("evt-1".to_owned()), SqlValue::Null],
                vec![SqlValue::String("evt-1".to_owned()), SqlValue::Null],
            ],
        )?;
        assert_eq!(ids.len(), 1);
        assert!(ids.iter().any(|id| id.as_str() == "evt-1"));
        Ok(())
    }
}
