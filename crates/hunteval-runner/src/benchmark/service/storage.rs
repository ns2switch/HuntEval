use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use hunteval_domain::BenchmarkDefinition;

use super::BenchmarkServiceError;

const DEFINITION_FILE: &str = "benchmark-definition.json";
const MAX_DEFINITION_BYTES: u64 = 16 * 1024 * 1024;

pub(super) fn store_definition(
    output_root: &Path,
    definition: &BenchmarkDefinition,
) -> Result<(), BenchmarkServiceError> {
    let path = output_root.join(DEFINITION_FILE);
    let mut bytes =
        serde_json::to_vec_pretty(definition).map_err(BenchmarkServiceError::Serialize)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(BenchmarkServiceError::Io)?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(BenchmarkServiceError::Io)
}

pub(super) fn verify_definition(
    output_root: &Path,
    expected: &BenchmarkDefinition,
) -> Result<(), BenchmarkServiceError> {
    let stored = load_stored_definition(output_root)?;
    if &stored == expected {
        Ok(())
    } else {
        Err(BenchmarkServiceError::ConfigurationDrift)
    }
}

pub fn load_stored_definition(
    output_root: &Path,
) -> Result<BenchmarkDefinition, BenchmarkServiceError> {
    let path = output_root.join(DEFINITION_FILE);
    let metadata = fs::symlink_metadata(&path).map_err(BenchmarkServiceError::Io)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_DEFINITION_BYTES
    {
        return Err(BenchmarkServiceError::ConfigurationDrift);
    }
    let bytes = fs::read(path).map_err(BenchmarkServiceError::Io)?;
    serde_json::from_slice(&bytes).map_err(BenchmarkServiceError::Serialize)
}
