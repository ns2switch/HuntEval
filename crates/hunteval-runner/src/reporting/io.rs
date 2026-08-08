use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use hunteval_domain::Sha256Digest;

use super::ReportGenerationError;

pub(super) const MAX_REPORT_INPUT_BYTES: u64 = 16 * 1024 * 1024;

pub(super) fn read_regular_bounded(path: &Path) -> Result<Vec<u8>, ReportGenerationError> {
    let metadata = fs::symlink_metadata(path).map_err(ReportGenerationError::Io)?;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() > MAX_REPORT_INPUT_BYTES
    {
        return Err(ReportGenerationError::InvalidInput);
    }
    let file = File::open(path).map_err(ReportGenerationError::Io)?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_REPORT_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(ReportGenerationError::Io)?;
    if bytes.len() as u64 > MAX_REPORT_INPUT_BYTES {
        return Err(ReportGenerationError::InvalidInput);
    }
    Ok(bytes)
}

pub(super) fn read_verified(
    path: &Path,
    expected: Sha256Digest,
) -> Result<Vec<u8>, ReportGenerationError> {
    let bytes = read_regular_bounded(path)?;
    if Sha256Digest::from_bytes(&bytes) != expected {
        return Err(ReportGenerationError::DigestMismatch(path.to_path_buf()));
    }
    Ok(bytes)
}

pub(super) fn write_atomic(
    root: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<(), ReportGenerationError> {
    let temporary = root.join(format!(".{name}.partial"));
    let destination = root.join(name);
    reject_symlink(&temporary)?;
    reject_symlink(&destination)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(ReportGenerationError::Io)?;
    let write_result = file.write_all(bytes).and_then(|()| file.sync_all());
    if let Err(error) = write_result {
        let _ignored = fs::remove_file(&temporary);
        return Err(ReportGenerationError::Io(error));
    }
    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ignored = fs::remove_file(&temporary);
        return Err(ReportGenerationError::Io(error));
    }
    File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(ReportGenerationError::Io)
}

fn reject_symlink(path: &Path) -> Result<(), ReportGenerationError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(ReportGenerationError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "report path is a symbolic link",
            )))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ReportGenerationError::Io(error)),
    }
}
