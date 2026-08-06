use std::{collections::BTreeSet, fs::File, io, path::Path};

use arrow_array::StringArray;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

fn fixture_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("datasets/aws/aws-iam-001/public/telemetry/cloudtrail.parquet")
}

#[test]
fn parquet_has_stable_schema_and_reference_ids() -> Result<(), Box<dyn std::error::Error>> {
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(fixture_path())?)?;
    let fields: Vec<_> = builder
        .schema()
        .fields()
        .iter()
        .map(|field| field.name().as_str())
        .collect();
    assert_eq!(
        fields,
        [
            "event_id",
            "event_time",
            "provider",
            "account_id",
            "principal",
            "event_name",
            "resource",
            "source_ip",
            "user_agent"
        ]
    );

    let mut ids = BTreeSet::new();
    for batch in builder.build()? {
        let batch = batch?;
        let event_ids = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| io::Error::other("event_id is not a string column"))?;
        ids.extend(event_ids.iter().flatten().map(str::to_owned));
    }
    assert_eq!(ids.len(), 10);
    for malicious_id in ["evt-0004", "evt-0005", "evt-0006"] {
        assert!(ids.contains(malicious_id));
    }
    Ok(())
}
