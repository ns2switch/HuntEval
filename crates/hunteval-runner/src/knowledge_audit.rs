use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use hunteval_domain::UtcTimestamp;
use hunteval_knowledge::{
    AnalyticalResult, CorpusScope, RetrievalAuditEvent, RetrievalAuditJournal,
};
use thiserror::Error;
use time::OffsetDateTime;

use crate::{AnalyticalCorpusLoadError, query_analytical_index};

const MAX_EVENT_BYTES: u64 = 64 * 1024;
const MAX_EVENTS: usize = 4096;

pub fn query_analytical_index_audited(
    root: &Path,
    manifest_bytes: &[u8],
    query_bytes: &[u8],
    audit_path: &Path,
) -> Result<AnalyticalResult, AnalyticalAuditError> {
    let started = Instant::now();
    let result = query_analytical_index(root, manifest_bytes, query_bytes)?;
    let latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let scope = serde_json::from_slice::<serde_json::Value>(query_bytes)
        .ok()
        .and_then(|value| value.get("scope").cloned())
        .and_then(|value| serde_json::from_value(value).ok())
        .ok_or(AnalyticalAuditError::InvalidQueryScope)?;
    let timestamp =
        UtcTimestamp::new(OffsetDateTime::now_utc()).map_err(|_| AnalyticalAuditError::Clock)?;
    let mut audit = RetrievalAuditFile::open(audit_path)?;
    audit.append(timestamp, scope, &result, latency_ms, None)?;
    Ok(result)
}

pub fn verify_retrieval_audit(path: &Path) -> Result<usize, AnalyticalAuditError> {
    if !path.exists() {
        return Err(AnalyticalAuditError::InvalidHistory);
    }
    let events = read_events(path)?;
    RetrievalAuditJournal::replay(events.clone())?;
    Ok(events.len())
}

#[derive(Debug)]
struct RetrievalAuditFile {
    path: PathBuf,
    lock_path: PathBuf,
    journal: RetrievalAuditJournal,
}

impl RetrievalAuditFile {
    fn open(path: &Path) -> Result<Self, AnalyticalAuditError> {
        let parent = path.parent().ok_or(AnalyticalAuditError::UnsafePath)?;
        let metadata = fs::symlink_metadata(parent).map_err(|_| AnalyticalAuditError::Io)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(AnalyticalAuditError::UnsafePath);
        }
        reject_existing_symlink(path)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(AnalyticalAuditError::UnsafePath)?;
        let lock_path = parent.join(format!(".{file_name}.lock"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    AnalyticalAuditError::ConcurrentWriter
                } else {
                    AnalyticalAuditError::Io
                }
            })?;
        let journal = match read_events(path).and_then(|events| {
            RetrievalAuditJournal::replay(events).map_err(AnalyticalAuditError::Knowledge)
        }) {
            Ok(journal) => journal,
            Err(error) => {
                let _ = fs::remove_file(&lock_path);
                return Err(error);
            }
        };
        Ok(Self {
            path: path.to_path_buf(),
            lock_path,
            journal,
        })
    }

    fn append(
        &mut self,
        timestamp: UtcTimestamp,
        scope: CorpusScope,
        result: &AnalyticalResult,
        latency_ms: u64,
        cost_microunits: Option<u64>,
    ) -> Result<(), AnalyticalAuditError> {
        let event = self
            .journal
            .append(timestamp, scope, result, latency_ms, cost_microunits)?
            .clone();
        let mut bytes = serde_json::to_vec(&event).map_err(|_| AnalyticalAuditError::Serialize)?;
        if bytes.len() as u64 > MAX_EVENT_BYTES {
            return Err(AnalyticalAuditError::EventLimit);
        }
        bytes.push(b'\n');
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        let mut file = options
            .open(&self.path)
            .map_err(|_| AnalyticalAuditError::Io)?;
        file.write_all(&bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| AnalyticalAuditError::Io)
    }
}

impl Drop for RetrievalAuditFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn read_events(path: &Path) -> Result<Vec<RetrievalAuditEvent>, AnalyticalAuditError> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.file_type().is_symlink()
                || !metadata.is_file()
                || metadata.len() > MAX_EVENT_BYTES.saturating_mul(MAX_EVENTS as u64) =>
        {
            return Err(AnalyticalAuditError::UnsafePath);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(AnalyticalAuditError::Io),
    }
    let mut events = Vec::new();
    for line in BufReader::new(File::open(path).map_err(|_| AnalyticalAuditError::Io)?).lines() {
        let line = line.map_err(|_| AnalyticalAuditError::Io)?;
        if line.len() as u64 > MAX_EVENT_BYTES || events.len() >= MAX_EVENTS {
            return Err(AnalyticalAuditError::EventLimit);
        }
        events.push(serde_json::from_str(&line).map_err(|_| AnalyticalAuditError::InvalidHistory)?);
    }
    Ok(events)
}

fn reject_existing_symlink(path: &Path) -> Result<(), AnalyticalAuditError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(AnalyticalAuditError::UnsafePath),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(AnalyticalAuditError::Io),
    }
}

#[derive(Debug, Error)]
pub enum AnalyticalAuditError {
    #[error("analytical query failed: {0}")]
    Query(#[from] AnalyticalCorpusLoadError),
    #[error("analytical query scope is malformed")]
    InvalidQueryScope,
    #[error("UTC clock could not be represented")]
    Clock,
    #[error("retrieval audit path is unsafe")]
    UnsafePath,
    #[error("another retrieval-audit writer owns the journal")]
    ConcurrentWriter,
    #[error("retrieval audit history is malformed")]
    InvalidHistory,
    #[error("retrieval audit event or journal exceeds its bound")]
    EventLimit,
    #[error("retrieval audit serialization failed")]
    Serialize,
    #[error("retrieval audit I/O failed")]
    Io,
    #[error("retrieval audit contract is invalid: {0}")]
    Knowledge(#[from] hunteval_knowledge::KnowledgeError),
}
