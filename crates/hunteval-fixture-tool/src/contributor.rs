use std::{fs, path::Path};

use hunteval_domain::{EpisodeClassification, EpisodeId, GroundTruth, Sha256Digest};
use serde::{Deserialize, Serialize};

use crate::FixtureGenerationError;

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// Bounded input for creating a new, incomplete episode package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldRequest<'a> {
    pub provider: &'a str,
    pub episode_id: &'a str,
    pub target: &'a Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributorCheckStatus {
    Passed,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContributorCheck {
    pub name: String,
    pub status: ContributorCheckStatus,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributorValidationStatus {
    Valid,
    Invalid,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContributorValidationResult {
    pub schema_version: String,
    pub episode_id: EpisodeId,
    pub status: ContributorValidationStatus,
    pub package_sha256: Option<Sha256Digest>,
    pub checks: Vec<ContributorCheck>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageIndex {
    schema_version: String,
    episode_id: EpisodeId,
    public_root: String,
    private_ground_truth: String,
}

/// Creates a non-overwriting skeleton under an exact new target directory.
pub fn scaffold_episode(request: &ScaffoldRequest<'_>) -> Result<(), FixtureGenerationError> {
    validate_provider(request.provider)?;
    let episode_id = EpisodeId::new(request.episode_id)
        .map_err(|_| FixtureGenerationError::UnsafeContributorTarget)?;
    validate_new_target(request.target)?;

    let parent = request
        .target
        .parent()
        .ok_or(FixtureGenerationError::UnsafeContributorTarget)?;
    let temporary = tempfile::Builder::new()
        .prefix(".hunteval-scaffold-")
        .tempdir_in(parent)?;
    let staged = ScaffoldRequest {
        provider: request.provider,
        episode_id: request.episode_id,
        target: temporary.path(),
    };
    write_skeleton(&staged, &episode_id)?;
    fs::rename(temporary.path(), request.target)?;
    Ok(())
}

fn write_skeleton(
    request: &ScaffoldRequest<'_>,
    episode_id: &EpisodeId,
) -> Result<(), FixtureGenerationError> {
    fs::create_dir(request.target.join("public"))?;
    fs::create_dir(request.target.join("private"))?;
    fs::create_dir(request.target.join("source"))?;
    fs::write(
        request.target.join("package.yaml"),
        format!(
            "schema_version: \"0.3\"\nepisode_id: {episode_id}\npublic_root: public\nprivate_ground_truth: private/ground-truth.json\n"
        ),
    )?;
    fs::write(
        request.target.join("public/classification.json"),
        format!(
            "{{\n  \"schema_version\": \"0.6\",\n  \"episode_id\": \"{episode_id}\",\n  \"difficulty\": \"introductory\",\n  \"capabilities\": [\"identity_analysis\"],\n  \"investigation_shapes\": [\"single_stage\"]\n}}\n"
        ),
    )?;
    fs::write(
        request.target.join("AUTHORING.md"),
        format!(
            "# {episode_id}\n\nProvider: `{}`\n\nThis scaffold is intentionally incomplete. Add a public manifest and telemetry, private ground truth, deterministic source events, a private reference query, and an independent review record before validation.\n",
            request.provider
        ),
    )?;
    Ok(())
}

/// Performs bounded, read-only structural validation and emits only safe reason codes.
pub fn validate_episode(
    root: &Path,
) -> Result<ContributorValidationResult, FixtureGenerationError> {
    reject_unsafe_tree(root)?;
    let package_bytes = bounded_read(&root.join("package.yaml"))?;
    let package: PackageIndex = serde_yaml_ng::from_slice(&package_bytes)
        .map_err(|_| FixtureGenerationError::MalformedContributorPackage)?;
    if package.schema_version != "0.3" || package.public_root != "public" {
        return Err(FixtureGenerationError::MalformedContributorPackage);
    }
    let mut checks = Vec::new();
    check_classification(root, &package.episode_id, &mut checks);
    check_ground_truth(
        root,
        &package.episode_id,
        &package.private_ground_truth,
        &mut checks,
    );
    check_required(root, "public_manifest", "public/manifest.yaml", &mut checks);
    check_required(root, "source_events", "source/events.json", &mut checks);
    check_required(
        root,
        "reference_query",
        "private/reference-query.sql",
        &mut checks,
    );
    check_required(
        root,
        "independent_review",
        "private/review.json",
        &mut checks,
    );
    let status = if checks
        .iter()
        .any(|check| check.status == ContributorCheckStatus::Failed)
    {
        ContributorValidationStatus::Invalid
    } else if checks
        .iter()
        .any(|check| check.status == ContributorCheckStatus::Unavailable)
    {
        ContributorValidationStatus::Incomplete
    } else {
        ContributorValidationStatus::Valid
    };
    Ok(ContributorValidationResult {
        schema_version: "0.6".to_owned(),
        episode_id: package.episode_id,
        status,
        package_sha256: Some(Sha256Digest::from_bytes(package_bytes)),
        checks,
    })
}

fn check_classification(root: &Path, id: &EpisodeId, checks: &mut Vec<ContributorCheck>) {
    let status = bounded_read(&root.join("public/classification.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<EpisodeClassification>(&bytes).ok())
        .filter(|value| value.episode_id == *id && value.validate().is_ok())
        .map_or(ContributorCheckStatus::Failed, |_| {
            ContributorCheckStatus::Passed
        });
    checks.push(check("classification", status, "classification_invalid"));
}

fn check_ground_truth(
    root: &Path,
    id: &EpisodeId,
    relative: &str,
    checks: &mut Vec<ContributorCheck>,
) {
    let path = safe_child(root, relative);
    let status = match path {
        Ok(path) if !path.exists() => ContributorCheckStatus::Unavailable,
        Ok(path) => bounded_read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<GroundTruth>(&bytes).ok())
            .filter(|value| value.episode_id == *id && value.validate().is_ok())
            .map_or(ContributorCheckStatus::Failed, |_| {
                ContributorCheckStatus::Passed
            }),
        Err(_) => ContributorCheckStatus::Failed,
    };
    checks.push(check("ground_truth", status, "ground_truth_invalid"));
}

fn check_required(root: &Path, name: &str, relative: &str, checks: &mut Vec<ContributorCheck>) {
    let status = if root.join(relative).is_file() {
        ContributorCheckStatus::Passed
    } else {
        ContributorCheckStatus::Unavailable
    };
    checks.push(check(name, status, "artifact_unavailable"));
}

fn check(name: &str, status: ContributorCheckStatus, reason: &str) -> ContributorCheck {
    ContributorCheck {
        name: name.to_owned(),
        status,
        reason_code: (status != ContributorCheckStatus::Passed).then(|| reason.to_owned()),
    }
}

fn validate_provider(value: &str) -> Result<(), FixtureGenerationError> {
    if matches!(value, "aws" | "azure" | "gcp") {
        Ok(())
    } else {
        Err(FixtureGenerationError::UnsafeContributorTarget)
    }
}

fn validate_new_target(target: &Path) -> Result<(), FixtureGenerationError> {
    if target.exists()
        || target.components().count() < 2
        || target.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(FixtureGenerationError::UnsafeContributorTarget);
    }
    let parent = target
        .parent()
        .ok_or(FixtureGenerationError::UnsafeContributorTarget)?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(FixtureGenerationError::UnsafeContributorTarget);
    }
    Ok(())
}

fn reject_unsafe_tree(root: &Path) -> Result<(), FixtureGenerationError> {
    let root = root.canonicalize()?;
    let mut pending = vec![root];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || metadata.len() > MAX_FILE_BYTES {
            return Err(FixtureGenerationError::UnsafeContributorTarget);
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? {
                pending.push(entry?.path());
            }
        }
    }
    Ok(())
}

fn safe_child(root: &Path, relative: &str) -> Result<std::path::PathBuf, FixtureGenerationError> {
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(FixtureGenerationError::UnsafeContributorTarget);
    }
    Ok(root.join(relative))
}

pub(super) fn bounded_read(path: &Path) -> Result<Vec<u8>, FixtureGenerationError> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_FILE_BYTES {
        return Err(FixtureGenerationError::UnsafeContributorTarget);
    }
    Ok(fs::read(path)?)
}
