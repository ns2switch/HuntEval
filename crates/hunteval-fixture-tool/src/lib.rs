//! Deterministic Parquet generation for synthetic HuntEval fixtures.

use std::{
    fs::{self, File},
    path::Path,
    sync::Arc,
};

use arrow_array::{ArrayRef, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::{
    arrow::ArrowWriter,
    basic::Compression,
    file::properties::{WriterProperties, WriterVersion},
};
use serde::Deserialize;
use thiserror::Error;

/// One provider-native synthetic CloudTrail row without ground-truth labels.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceEvent {
    event_id: String,
    event_time: String,
    account_id: String,
    principal: String,
    event_name: String,
    resource: String,
    source_ip: String,
    user_agent: String,
}

/// Generates byte-stable uncompressed Parquet from a versioned JSON source.
pub fn generate_fixture(source: &Path, output: &Path) -> Result<(), FixtureGenerationError> {
    let source_json = fs::read_to_string(source)?;
    let mut events: Vec<SourceEvent> = serde_json::from_str(&source_json)?;
    events.sort_by(|left, right| left.event_id.cmp(&right.event_id));
    if events.is_empty() {
        return Err(FixtureGenerationError::EmptySource);
    }
    for pair in events.windows(2) {
        if pair[0].event_id == pair[1].event_id {
            return Err(FixtureGenerationError::DuplicateEventId);
        }
    }

    let schema = Arc::new(Schema::new(vec![
        Field::new("event_id", DataType::Utf8, false),
        Field::new("event_time", DataType::Utf8, false),
        Field::new("provider", DataType::Utf8, false),
        Field::new("account_id", DataType::Utf8, false),
        Field::new("principal", DataType::Utf8, false),
        Field::new("event_name", DataType::Utf8, false),
        Field::new("resource", DataType::Utf8, false),
        Field::new("source_ip", DataType::Utf8, false),
        Field::new("user_agent", DataType::Utf8, false),
    ]));
    let batch = RecordBatch::try_new(schema.clone(), columns(&events))?;

    let parent = output
        .parent()
        .ok_or(FixtureGenerationError::OutputHasNoParent)?;
    fs::create_dir_all(parent)?;
    let properties = WriterProperties::builder()
        .set_created_by("HuntEval fixture generator 0.1.0".to_owned())
        .set_writer_version(WriterVersion::PARQUET_1_0)
        .set_compression(Compression::UNCOMPRESSED)
        .set_max_row_group_row_count(Some(events.len()))
        .build();
    let mut writer = ArrowWriter::try_new(File::create(output)?, schema, Some(properties))?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn columns(events: &[SourceEvent]) -> Vec<ArrayRef> {
    vec![
        strings(events, |event| &event.event_id),
        strings(events, |event| &event.event_time),
        Arc::new(StringArray::from(vec!["aws"; events.len()])),
        strings(events, |event| &event.account_id),
        strings(events, |event| &event.principal),
        strings(events, |event| &event.event_name),
        strings(events, |event| &event.resource),
        strings(events, |event| &event.source_ip),
        strings(events, |event| &event.user_agent),
    ]
}

fn strings(events: &[SourceEvent], field: fn(&SourceEvent) -> &str) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(events.iter().map(field)))
}

/// Typed fixture generation failures.
#[derive(Debug, Error)]
pub enum FixtureGenerationError {
    #[error("fixture source could not be read or output could not be written")]
    Io(#[from] std::io::Error),
    #[error("fixture source is not valid JSON")]
    Json(#[from] serde_json::Error),
    #[error("fixture Arrow data is invalid")]
    Arrow(#[from] arrow_schema::ArrowError),
    #[error("fixture Parquet output failed")]
    Parquet(#[from] parquet::errors::ParquetError),
    #[error("fixture source must contain at least one event")]
    EmptySource,
    #[error("fixture event identifiers must be unique")]
    DuplicateEventId,
    #[error("fixture output path must have a parent directory")]
    OutputHasNoParent,
}
