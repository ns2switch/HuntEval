use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::storage::BenchmarkJournalError;

const LOCK_NAME: &str = ".benchmark.lock";

#[derive(Debug)]
pub(super) struct JournalLock {
    path: PathBuf,
}

impl JournalLock {
    pub(super) fn acquire(root: &Path) -> Result<Self, BenchmarkJournalError> {
        let path = root.join(LOCK_NAME);
        reject_symlink_if_present(&path)?;
        let mut file = match create_lock(&path) {
            Ok(file) => file,
            Err(BenchmarkJournalError::Locked) => {
                reclaim_stale_lock(&path)?;
                create_lock(&path)?
            }
            Err(error) => return Err(error),
        };
        writeln!(file, "{}", std::process::id()).map_err(BenchmarkJournalError::Io)?;
        file.sync_all().map_err(BenchmarkJournalError::Io)?;
        Ok(Self { path })
    }
}

impl Drop for JournalLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub(super) fn prepare_root(path: &Path) -> Result<PathBuf, BenchmarkJournalError> {
    reject_symlink_if_present(path)?;
    fs::create_dir_all(path).map_err(BenchmarkJournalError::Io)?;
    let root = fs::canonicalize(path).map_err(BenchmarkJournalError::Io)?;
    if !root.is_dir() {
        return Err(BenchmarkJournalError::UnsafePath);
    }
    Ok(root)
}

pub(super) fn reject_symlink_if_present(path: &Path) -> Result<(), BenchmarkJournalError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(BenchmarkJournalError::UnsafePath),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(BenchmarkJournalError::Io(error)),
    }
}

pub(super) fn recover_temporary_state(
    root: &Path,
    name: &str,
) -> Result<(), BenchmarkJournalError> {
    let temporary = root.join(name);
    reject_symlink_if_present(&temporary)?;
    if temporary.exists() {
        fs::remove_file(temporary).map_err(BenchmarkJournalError::Io)?;
    }
    Ok(())
}

fn create_lock(path: &Path) -> Result<File, BenchmarkJournalError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                BenchmarkJournalError::Locked
            } else {
                BenchmarkJournalError::Io(error)
            }
        })
}

fn reclaim_stale_lock(path: &Path) -> Result<(), BenchmarkJournalError> {
    reject_symlink_if_present(path)?;
    let metadata = fs::metadata(path).map_err(BenchmarkJournalError::Io)?;
    if !metadata.is_file() || metadata.len() > 64 {
        return Err(BenchmarkJournalError::Locked);
    }
    let text = fs::read_to_string(path).map_err(BenchmarkJournalError::Io)?;
    let process_id = text
        .trim()
        .parse::<u32>()
        .map_err(|_| BenchmarkJournalError::Locked)?;
    if process_is_live(process_id) {
        return Err(BenchmarkJournalError::Locked);
    }
    fs::remove_file(path).map_err(BenchmarkJournalError::Io)
}

#[cfg(target_os = "linux")]
fn process_is_live(process_id: u32) -> bool {
    Path::new("/proc").join(process_id.to_string()).exists()
}

#[cfg(not(target_os = "linux"))]
const fn process_is_live(_: u32) -> bool {
    true
}
