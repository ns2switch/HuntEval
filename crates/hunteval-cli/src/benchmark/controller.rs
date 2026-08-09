use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const CONTROLLER_FILE: &str = "benchmark-controller.json";
const MAX_CONTROLLER_BYTES: u64 = 64 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ControllerConfig {
    pub manifest: PathBuf,
    pub artifact_root: PathBuf,
    pub deployment_executable: PathBuf,
    pub duckdb_worker: PathBuf,
    pub schema_contract: PathBuf,
    pub jobs: usize,
    pub fail_fast: bool,
}

pub(super) fn sibling_binary_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .ok_or_else(|| std::io::Error::other("CLI executable has no parent directory"))?
        .to_path_buf())
}

pub(super) fn canonical_regular(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(std::io::Error::other("expected a regular non-symlink file").into());
    }
    Ok(path.canonicalize()?)
}

pub(super) fn canonical_directory(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(std::io::Error::other("expected a non-symlink directory").into());
    }
    Ok(path.canonicalize()?)
}

pub(super) fn create_output_directory(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "benchmark output already exists",
        )
        .into());
    }
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("benchmark output has no directory name"))?;
    if !matches!(path.components().next_back(), Some(Component::Normal(_))) {
        return Err(std::io::Error::other("benchmark output path is unsafe").into());
    }
    let parent = canonical_directory(path.parent().unwrap_or_else(|| Path::new(".")))?;
    let output = parent.join(name);
    fs::create_dir(&output)?;
    Ok(output.canonicalize()?)
}

pub(super) fn write_controller(
    root: &Path,
    config: &ControllerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = serde_json::to_vec_pretty(config)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(CONTROLLER_FILE))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn read_controller(root: &Path) -> Result<ControllerConfig, Box<dyn std::error::Error>> {
    let path = root.join(CONTROLLER_FILE);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTROLLER_BYTES
    {
        return Err(std::io::Error::other("benchmark controller file is unsafe").into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_CONTROLLER_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONTROLLER_BYTES {
        return Err(std::io::Error::other("benchmark controller file is too large").into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
