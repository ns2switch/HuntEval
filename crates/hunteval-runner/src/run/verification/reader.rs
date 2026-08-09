use std::{fs, io::Read, path::Path};

use super::{RunVerificationResult, VerificationCheck, VerificationStatus};

pub(super) struct Verifier {
    pub(super) checks: Vec<VerificationCheck>,
    checked_artifacts: usize,
}

impl Verifier {
    pub(super) fn new() -> Self {
        Self {
            checks: Vec::new(),
            checked_artifacts: 0,
        }
    }

    pub(super) fn read(&mut self, root: &Path, name: &str, maximum: u64) -> Option<Vec<u8>> {
        let root_metadata = match fs::symlink_metadata(root) {
            Ok(value) => value,
            Err(_) => {
                self.failed(name, "missing_root");
                return None;
            }
        };
        if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
            self.failed(name, "unsafe_root");
            return None;
        }
        let path = root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(value) => value,
            Err(_) => {
                self.failed(name, "missing_artifact");
                return None;
            }
        };
        if !safe_regular_file(&metadata) || metadata.len() > maximum {
            self.failed(name, "unsafe_artifact");
            return None;
        }
        let mut bytes = Vec::new();
        let file = match open_read_no_follow(&path) {
            Ok(value) => value,
            Err(_) => {
                self.failed(name, "unreadable_artifact");
                return None;
            }
        };
        let opened_metadata = match file.metadata() {
            Ok(value) if safe_regular_file(&value) && same_file(&metadata, &value) => value,
            _ => {
                self.failed(name, "unsafe_artifact");
                return None;
            }
        };
        if opened_metadata.len() > maximum {
            self.failed(name, "unsafe_artifact");
            return None;
        }
        let mut file = file.take(maximum + 1);
        if file.read_to_end(&mut bytes).is_err() || bytes.len() as u64 > maximum {
            self.failed(name, "unreadable_artifact");
            return None;
        }
        self.checked_artifacts += 1;
        Some(bytes)
    }

    pub(super) fn passed(&mut self, check: &str) {
        self.checks.push(VerificationCheck {
            check: check.to_owned(),
            passed: true,
            reason: None,
        });
    }

    pub(super) fn failed(&mut self, check: &str, reason: &str) {
        self.checks.push(VerificationCheck {
            check: check.to_owned(),
            passed: false,
            reason: Some(reason.to_owned()),
        });
    }

    pub(super) fn finish(self, status: VerificationStatus) -> RunVerificationResult {
        RunVerificationResult {
            schema_version: "0.5".to_owned(),
            status,
            private_evaluation: "not_checked".to_owned(),
            checked_artifacts: self.checked_artifacts,
            checks: self.checks,
        }
    }
}

fn open_read_no_follow(path: &Path) -> Result<fs::File, std::io::Error> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    options.open(path)
}

fn safe_regular_file(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return false;
        }
    }
    true
}

fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        left.dev() == right.dev() && left.ino() == right.ino()
    }
    #[cfg(not(unix))]
    {
        left.len() == right.len()
    }
}
