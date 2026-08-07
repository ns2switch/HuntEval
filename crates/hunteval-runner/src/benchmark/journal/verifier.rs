use std::{
    fs,
    fs::File,
    io::{Read, Take},
    path::Path,
};

use hunteval_domain::{BenchmarkCellId, RunId, Sha256Digest};
use serde::Deserialize;

use super::storage::BenchmarkJournalError;

const MAX_RESULT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Deserialize)]
struct ResultIdentity {
    cell_id: BenchmarkCellId,
    run_id: RunId,
}

pub(super) fn verify_result(
    path: &Path,
    cell_id: BenchmarkCellId,
    run_id: &RunId,
) -> Result<Sha256Digest, BenchmarkJournalError> {
    let metadata = fs::symlink_metadata(path).map_err(BenchmarkJournalError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_RESULT_BYTES
    {
        return Err(BenchmarkJournalError::InvalidResult);
    }
    let mut bytes = Vec::new();
    let file = File::open(path).map_err(BenchmarkJournalError::Io)?;
    read_result(file.take(MAX_RESULT_BYTES + 1), &mut bytes)?;
    if bytes.len() as u64 > MAX_RESULT_BYTES {
        return Err(BenchmarkJournalError::InvalidResult);
    }
    let identity: ResultIdentity =
        serde_json::from_slice(&bytes).map_err(|_| BenchmarkJournalError::InvalidResult)?;
    if identity.cell_id != cell_id || &identity.run_id != run_id {
        return Err(BenchmarkJournalError::InvalidResult);
    }
    Ok(Sha256Digest::from_bytes(bytes))
}

fn read_result(mut reader: Take<File>, bytes: &mut Vec<u8>) -> Result<(), BenchmarkJournalError> {
    reader
        .read_to_end(bytes)
        .map(|_| ())
        .map_err(BenchmarkJournalError::Io)
}
