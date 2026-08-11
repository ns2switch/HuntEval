use std::{collections::BTreeSet, fs, io::Read, path::Path, sync::LazyLock};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DEFAULT_MAXIMUM_FILE_BYTES: u64 = 128 * 1024 * 1024;
const TOKEN_PREFIX_MASK: u8 = 0xa5;

struct TokenRule {
    encoded_prefix: &'static [u8],
    rule_id: &'static str,
    length: usize,
}

const TOKEN_RULES: [TokenRule; 3] = [
    TokenRule {
        encoded_prefix: &[0xe4, 0xee, 0xec, 0xe4],
        rule_id: "aws_access_key",
        length: 20,
    },
    TokenRule {
        encoded_prefix: &[0xc2, 0xcd, 0xd5, 0xfa],
        rule_id: "github_token",
        length: 40,
    },
    TokenRule {
        encoded_prefix: &[
            0xc2, 0xcc, 0xd1, 0xcd, 0xd0, 0xc7, 0xfa, 0xd5, 0xc4, 0xd1, 0xfa,
        ],
        rule_id: "github_fine_grained_token",
        length: 82,
    },
];

static TOKEN_PREFIXES: LazyLock<[Vec<u8>; 3]> = LazyLock::new(|| {
    TOKEN_RULES.map(|rule| {
        // Keeping scanner signatures out of the release binary prevents the
        // scanner from treating its own immutable rule data as a credential.
        let mask = std::hint::black_box(TOKEN_PREFIX_MASK);
        rule.encoded_prefix.iter().map(|byte| byte ^ mask).collect()
    })
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretScanPolicy {
    pub maximum_files: usize,
    pub maximum_file_bytes: u64,
    pub allowlisted_fingerprints: BTreeSet<String>,
}

impl Default for SecretScanPolicy {
    fn default() -> Self {
        Self {
            maximum_files: 100_000,
            maximum_file_bytes: DEFAULT_MAXIMUM_FILE_BYTES,
            allowlisted_fingerprints: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretScanStatus {
    Clean,
    Findings,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretScanFinding {
    pub rule_id: String,
    pub artifact: String,
    pub line: usize,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretScanResult {
    pub schema_version: String,
    pub status: SecretScanStatus,
    pub scanned_artifacts: usize,
    pub findings: Vec<SecretScanFinding>,
    pub incomplete_reasons: Vec<String>,
}

#[must_use]
pub fn scan_paths(
    root: &Path,
    relative_paths: &[std::path::PathBuf],
    policy: &SecretScanPolicy,
) -> SecretScanResult {
    let mut result = SecretScanResult {
        schema_version: "0.5".to_owned(),
        status: SecretScanStatus::Clean,
        scanned_artifacts: 0,
        findings: Vec::new(),
        incomplete_reasons: Vec::new(),
    };
    if fs::symlink_metadata(root)
        .map(|metadata| metadata.file_type().is_symlink() || !metadata.is_dir())
        .unwrap_or(true)
    {
        result.incomplete_reasons.push("unsafe_root".to_owned());
        result.status = SecretScanStatus::Incomplete;
        return result;
    }
    if policy.maximum_files == 0 || policy.maximum_file_bytes == 0 {
        result.incomplete_reasons.push("invalid_policy".to_owned());
        result.status = SecretScanStatus::Incomplete;
        return result;
    }
    for relative in relative_paths.iter().take(policy.maximum_files) {
        let Some(label) = safe_label(relative) else {
            result.incomplete_reasons.push("unsafe_path".to_owned());
            continue;
        };
        let path = root.join(relative);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            result
                .incomplete_reasons
                .push("unreadable_artifact".to_owned());
            continue;
        };
        if !safe_regular_file(&metadata) || metadata.len() > policy.maximum_file_bytes {
            result.incomplete_reasons.push("unsafe_artifact".to_owned());
            continue;
        }
        let Ok(mut file) = open_read_no_follow(&path) else {
            result
                .incomplete_reasons
                .push("unreadable_artifact".to_owned());
            continue;
        };
        let opened_metadata = match file.metadata() {
            Ok(value) if safe_regular_file(&value) && same_file(&metadata, &value) => value,
            _ => {
                result.incomplete_reasons.push("unsafe_artifact".to_owned());
                continue;
            }
        };
        if opened_metadata.len() > policy.maximum_file_bytes {
            result.incomplete_reasons.push("unsafe_artifact".to_owned());
            continue;
        }
        let mut bytes = Vec::new();
        if file
            .by_ref()
            .take(policy.maximum_file_bytes + 1)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() as u64 > policy.maximum_file_bytes
        {
            result
                .incomplete_reasons
                .push("unreadable_artifact".to_owned());
            continue;
        }
        result.scanned_artifacts += 1;
        scan_bytes(&label, &bytes, policy, &mut result.findings);
    }
    if relative_paths.len() > policy.maximum_files {
        result.incomplete_reasons.push("file_limit".to_owned());
    }
    result.incomplete_reasons.sort();
    result.incomplete_reasons.dedup();
    result.findings.sort_by(|left, right| {
        (&left.artifact, left.line, &left.rule_id, &left.fingerprint).cmp(&(
            &right.artifact,
            right.line,
            &right.rule_id,
            &right.fingerprint,
        ))
    });
    result.status = if !result.incomplete_reasons.is_empty() {
        SecretScanStatus::Incomplete
    } else if result.findings.is_empty() {
        SecretScanStatus::Clean
    } else {
        SecretScanStatus::Findings
    };
    result
}

fn scan_bytes(
    artifact: &str,
    bytes: &[u8],
    policy: &SecretScanPolicy,
    findings: &mut Vec<SecretScanFinding>,
) {
    for (line_index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        for (rule_id, candidate) in candidates(line) {
            let fingerprint = hex::encode(Sha256::digest(candidate));
            if !policy.allowlisted_fingerprints.contains(&fingerprint) {
                findings.push(SecretScanFinding {
                    rule_id: rule_id.to_owned(),
                    artifact: artifact.to_owned(),
                    line: line_index + 1,
                    fingerprint,
                });
            }
        }
    }
}

fn candidates(line: &[u8]) -> Vec<(&'static str, &[u8])> {
    let mut output = Vec::new();
    if let Some(index) = find(line, b"-----BEGIN ")
        && line[index..]
            .windows(12)
            .any(|window| window == b"PRIVATE KEY-")
    {
        output.push(("private_key", &line[index..]));
    }
    for (rule, prefix) in TOKEN_RULES.iter().zip(TOKEN_PREFIXES.iter()) {
        let mut offset = 0;
        while let Some(index) = find(&line[offset..], prefix) {
            let start = offset + index;
            let end = start.saturating_add(rule.length);
            if end <= line.len()
                && line[start..end]
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                output.push((rule.rule_id, &line[start..end]));
            }
            offset = start + prefix.len();
        }
    }
    output
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn safe_label(path: &Path) -> Option<String> {
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return None;
    }
    path.to_str().map(str::to_owned)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_never_contain_the_matched_value() -> Result<(), Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let token = format!("{}{}", "ghp_", "123456789012345678901234567890123456");
        fs::write(root.path().join("fixture.txt"), format!("token={token}\n"))?;
        let result = scan_paths(
            root.path(),
            &["fixture.txt".into()],
            &SecretScanPolicy::default(),
        );
        let json = serde_json::to_string(&result)?;
        assert_eq!(result.status, SecretScanStatus::Findings);
        assert!(!json.contains(&token));
        assert_eq!(result.findings[0].rule_id, "github_token");
        Ok(())
    }

    #[test]
    fn unsafe_paths_make_the_scan_incomplete() {
        let result = scan_paths(
            Path::new("."),
            &["../private".into()],
            &SecretScanPolicy::default(),
        );
        assert_eq!(result.status, SecretScanStatus::Incomplete);
        assert_eq!(result.scanned_artifacts, 0);
    }

    #[test]
    fn release_binary_bound_remains_explicit_and_finite() {
        let policy = SecretScanPolicy::default();
        assert_eq!(policy.maximum_file_bytes, 128 * 1024 * 1024);
    }

    #[test]
    fn encoded_token_prefixes_preserve_the_detection_contract() {
        assert_eq!(TOKEN_PREFIXES[0], b"AKIA");
        assert_eq!(TOKEN_PREFIXES[1], b"ghp_");
        assert_eq!(TOKEN_PREFIXES[2], b"github_pat_");
    }
}
