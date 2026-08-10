use std::{collections::BTreeSet, fs, io::Read, path::Path};

use hunteval_domain::{
    ExtensionCapabilityPolicy, ExtensionConformanceResult, ExtensionConformanceStatus,
    ExtensionKind, ExtensionManifest, ExtensionResolution, ExtensionResolutionStatus,
    SchemaVersion, Sha256Digest,
};
use thiserror::Error;

const MAX_EXECUTABLE_BYTES: u64 = 256 * 1024 * 1024;

pub fn validate_extension_manifest(
    manifest_bytes: &[u8],
) -> Result<ExtensionManifest, ExtensionServiceError> {
    let manifest: ExtensionManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|_| ExtensionServiceError::InvalidManifest)?;
    manifest
        .validate()
        .map_err(|_| ExtensionServiceError::InvalidManifest)?;
    Ok(manifest)
}

pub fn resolve_extension(
    manifest_bytes: &[u8],
    policy_bytes: &[u8],
) -> Result<ExtensionResolution, ExtensionServiceError> {
    let manifest = validate_extension_manifest(manifest_bytes)?;
    let policy: ExtensionCapabilityPolicy =
        serde_json::from_slice(policy_bytes).map_err(|_| ExtensionServiceError::InvalidPolicy)?;
    policy
        .validate()
        .map_err(|_| ExtensionServiceError::InvalidPolicy)?;
    Ok(ExtensionResolution::resolve(
        Sha256Digest::from_bytes(manifest_bytes),
        &manifest,
        &policy,
    ))
}

pub fn check_extension(
    manifest_bytes: &[u8],
    policy: &ExtensionCapabilityPolicy,
    executable: &Path,
) -> ExtensionConformanceResult {
    let manifest_sha256 = Sha256Digest::from_bytes(manifest_bytes);
    let mut checks = BTreeSet::new();
    let mut reasons = Vec::new();
    let parsed = validate_extension_manifest(manifest_bytes);
    let executable_sha256 = hash_bounded_executable(executable);
    if parsed.is_ok() {
        checks.insert("manifest_validation".to_owned());
    } else {
        reasons.push("invalid_manifest".to_owned());
    }
    let fallback = Sha256Digest::from_bytes([]);
    let actual_executable = executable_sha256.unwrap_or(fallback);
    if executable_sha256.is_some() {
        checks.insert("bounded_regular_executable".to_owned());
    } else {
        reasons.push("invalid_executable".to_owned());
    }
    if policy.validate().is_err() {
        reasons.push("invalid_policy".to_owned());
    }
    if let Ok(manifest) = parsed {
        let resolution = ExtensionResolution::resolve(manifest_sha256, &manifest, policy);
        if resolution.status == ExtensionResolutionStatus::Eligible {
            checks.insert("capability_policy".to_owned());
        } else {
            reasons.extend(resolution.reasons);
        }
        if actual_executable == manifest.executable_sha256 {
            checks.insert("executable_identity".to_owned());
        } else {
            reasons.push("executable_digest_mismatch".to_owned());
        }
    }
    reasons.sort();
    reasons.dedup();
    ExtensionConformanceResult {
        schema_version: SchemaVersion::new(0, 9),
        manifest_sha256,
        executable_sha256: actual_executable,
        policy_sha256: policy.policy_sha256,
        protocol_transcript_sha256: None,
        status: if reasons.is_empty() {
            ExtensionConformanceStatus::Conformant
        } else {
            ExtensionConformanceStatus::Rejected
        },
        checks,
        reasons,
    }
}

pub fn conform_extension(
    manifest_bytes: &[u8],
    policy: &ExtensionCapabilityPolicy,
    executable: &Path,
    arguments: &[String],
) -> ExtensionConformanceResult {
    let mut result = check_extension(manifest_bytes, policy, executable);
    if result.status != ExtensionConformanceStatus::Conformant {
        return result;
    }
    let Ok(manifest) = validate_extension_manifest(manifest_bytes) else {
        return result;
    };
    match manifest.kind {
        ExtensionKind::DeploymentAdapter => {
            if !manifest
                .supported_versions
                .contains(&SchemaVersion::new(0, 3))
            {
                result.reasons.push("unsupported_protocol".to_owned());
            } else {
                let protocol = crate::run_conformance(executable, arguments);
                if protocol.status == crate::ConformanceStatus::Conformant {
                    result.checks.insert("deployment_protocol".to_owned());
                    result.protocol_transcript_sha256 = protocol.transcript_sha256.parse().ok();
                } else {
                    result
                        .reasons
                        .push("deployment_protocol_failure".to_owned());
                }
            }
        }
        ExtensionKind::ManagedTool => {
            match crate::managed_tool_adapter::conform_managed_tool(executable, &manifest) {
                Ok(transcript_sha256) => {
                    result.checks.insert("managed_tool_protocol".to_owned());
                    result.protocol_transcript_sha256 = Some(transcript_sha256);
                }
                Err(reason) => result.reasons.push(reason.to_owned()),
            }
        }
    }
    result.reasons.sort();
    result.reasons.dedup();
    if !result.reasons.is_empty() {
        result.status = ExtensionConformanceStatus::Rejected;
        result.protocol_transcript_sha256 = None;
    }
    result
}

fn hash_bounded_executable(path: &Path) -> Option<Sha256Digest> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_EXECUTABLE_BYTES
    {
        return None;
    }
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path).ok()?;
    let opened = file.metadata().ok()?;
    if !opened.is_file() || opened.len() != metadata.len() {
        return None;
    }
    #[cfg(unix)]
    if opened.dev() != metadata.dev() || opened.ino() != metadata.ino() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_EXECUTABLE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_EXECUTABLE_BYTES {
        return None;
    }
    Some(Sha256Digest::from_bytes(bytes))
}

#[derive(Debug, Error)]
pub enum ExtensionServiceError {
    #[error("extension manifest is malformed or invalid")]
    InvalidManifest,
    #[error("extension capability policy is malformed or invalid")]
    InvalidPolicy,
}
