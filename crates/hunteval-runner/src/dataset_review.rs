use std::{fs, path::Path};

use hunteval_domain::{
    DatasetReviewId, DatasetReviewRecord, DatasetReviewStatus, ReviewerId, Sha256Digest,
    UtcTimestamp,
};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{EpisodeLoadError, EpisodePackage};

const MAX_PUBLIC_FILES: usize = 256;
const MAX_PUBLIC_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetReviewValidationStatus {
    Approved,
    Missing,
    Stale,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetReviewValidation {
    pub status: DatasetReviewValidationStatus,
    pub reason_code: &'static str,
    pub review_sha256: Option<Sha256Digest>,
}

#[derive(Debug, Error)]
pub enum DatasetReviewError {
    #[error("episode package is invalid")]
    Episode(#[from] EpisodeLoadError),
    #[error("dataset review I/O failed")]
    Io(#[from] std::io::Error),
    #[error("dataset review artifact is malformed")]
    Malformed,
    #[error("public package exceeds review bounds")]
    ArtifactLimit,
}

/// Builds an approved review record from exact current bytes after human approval.
///
/// This function does not perform or infer the review. Its caller must obtain an
/// independent human decision before invoking it.
pub fn create_approved_dataset_review(
    root: &Path,
    review_policy_bytes: &[u8],
    review_id: &str,
    reviewer_id: &str,
    reviewed_at: &str,
) -> Result<DatasetReviewRecord, DatasetReviewError> {
    if review_policy_bytes.is_empty() || review_policy_bytes.len() > 1024 * 1024 {
        return Err(DatasetReviewError::ArtifactLimit);
    }
    let package = EpisodePackage::load(root)?;
    let query_bytes = read_regular(&root.join("private/reference-query.sql"))?;
    let record = DatasetReviewRecord {
        schema_version: hunteval_domain::SchemaVersion::new(0, 6),
        review_id: DatasetReviewId::new(review_id).map_err(|_| DatasetReviewError::Malformed)?,
        episode_id: package.public().manifest.id.clone(),
        reviewer_id: ReviewerId::new(reviewer_id).map_err(|_| DatasetReviewError::Malformed)?,
        reviewed_at: reviewed_at
            .parse::<UtcTimestamp>()
            .map_err(|_| DatasetReviewError::Malformed)?,
        status: DatasetReviewStatus::Approved,
        public_package_sha256: hash_public_package(&package.public().public_root)?,
        private_ground_truth_sha256: package.digests().private_ground_truth,
        reference_query_sha256: Sha256Digest::from_bytes(query_bytes),
        review_policy_sha256: Sha256Digest::from_bytes(review_policy_bytes),
        reason_codes: Default::default(),
    };
    record
        .validate()
        .map_err(|_| DatasetReviewError::Malformed)?;
    Ok(record)
}

/// Validates that an approval binds the exact current public, private, query and policy bytes.
pub fn validate_dataset_review(
    root: &Path,
    review_policy_bytes: &[u8],
) -> Result<DatasetReviewValidation, DatasetReviewError> {
    let package = EpisodePackage::load(root)?;
    let review_path = root.join("private/review.json");
    if !review_path.try_exists()? {
        return Ok(result(
            DatasetReviewValidationStatus::Missing,
            "review_missing",
            None,
        ));
    }
    let query_path = root.join("private/reference-query.sql");
    let review_bytes = read_regular(&review_path)?;
    let query_bytes = read_regular(&query_path)?;
    let review: DatasetReviewRecord =
        serde_json::from_slice(&review_bytes).map_err(|_| DatasetReviewError::Malformed)?;
    review
        .validate()
        .map_err(|_| DatasetReviewError::Malformed)?;
    let review_sha256 = Some(Sha256Digest::from_bytes(&review_bytes));
    if review.episode_id != package.public().manifest.id {
        return Ok(result(
            DatasetReviewValidationStatus::Stale,
            "review_episode_mismatch",
            review_sha256,
        ));
    }
    if review.status == DatasetReviewStatus::Rejected {
        return Ok(result(
            DatasetReviewValidationStatus::Rejected,
            "review_rejected",
            review_sha256,
        ));
    }
    let public_hash = hash_public_package(&package.public().public_root)?;
    let matches = review.public_package_sha256 == public_hash
        && review.private_ground_truth_sha256 == package.digests().private_ground_truth
        && review.reference_query_sha256 == Sha256Digest::from_bytes(query_bytes)
        && review.review_policy_sha256 == Sha256Digest::from_bytes(review_policy_bytes);
    Ok(if matches {
        result(
            DatasetReviewValidationStatus::Approved,
            "review_approved",
            review_sha256,
        )
    } else {
        result(
            DatasetReviewValidationStatus::Stale,
            "review_hash_mismatch",
            review_sha256,
        )
    })
}

fn result(
    status: DatasetReviewValidationStatus,
    reason_code: &'static str,
    review_sha256: Option<Sha256Digest>,
) -> DatasetReviewValidation {
    DatasetReviewValidation {
        status,
        reason_code,
        review_sha256,
    }
}

fn read_regular(path: &Path) -> Result<Vec<u8>, DatasetReviewError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > MAX_PUBLIC_BYTES
    {
        return Err(DatasetReviewError::ArtifactLimit);
    }
    Ok(fs::read(path)?)
}

/// Hashes a validated public package tree using stable relative paths and exact bytes.
pub fn hash_public_package(root: &Path) -> Result<Sha256Digest, DatasetReviewError> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(DatasetReviewError::ArtifactLimit);
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path)? {
                pending.push(entry?.path());
            }
        } else if metadata.is_file() {
            files.push(path);
        } else {
            return Err(DatasetReviewError::ArtifactLimit);
        }
    }
    files.sort();
    if files.is_empty() || files.len() > MAX_PUBLIC_FILES {
        return Err(DatasetReviewError::ArtifactLimit);
    }
    let mut total = 0_u64;
    let mut hasher = Sha256::new();
    for path in files {
        let relative = path
            .strip_prefix(root)
            .map_err(|_| DatasetReviewError::ArtifactLimit)?
            .to_str()
            .ok_or(DatasetReviewError::ArtifactLimit)?;
        let bytes = read_regular(&path)?;
        total = total
            .checked_add(bytes.len() as u64)
            .ok_or(DatasetReviewError::ArtifactLimit)?;
        if total > MAX_PUBLIC_BYTES {
            return Err(DatasetReviewError::ArtifactLimit);
        }
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update(bytes);
    }
    hex::encode(hasher.finalize())
        .parse()
        .map_err(|_| DatasetReviewError::Malformed)
}
