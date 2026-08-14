use std::{fs, path::Path};

use hunteval_domain::Sha256Digest;
use hunteval_runner::{
    DatasetReviewValidationStatus, EpisodePackage, create_approved_dataset_review,
    hash_public_package, validate_dataset_review,
};

fn copy_tree(source: &Path, target: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}

#[test]
fn approved_record_generation_requires_exact_bounded_inputs()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let episode = workspace.join("datasets/aws/aws-iam-004");
    let policy = fs::read(workspace.join("policies/dataset-review-v1.json"))?;
    let record = create_approved_dataset_review(
        &episode,
        &policy,
        "review-aws-iam-004-r4",
        "anibal.canada",
        "2026-08-09T00:00:00Z",
    )?;
    assert_eq!(record.episode_id.as_str(), "aws-iam-004");
    assert_eq!(record.reviewer_id.as_str(), "anibal.canada");
    assert!(
        create_approved_dataset_review(
            &episode,
            &[],
            "review-aws-iam-004-r4",
            "anibal.canada",
            "2026-08-09T00:00:00Z",
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn review_binds_exact_public_private_query_and_policy_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temporary = tempfile::tempdir()?;
    let episode = temporary.path().join("aws-iam-004");
    copy_tree(&workspace.join("datasets/aws/aws-iam-004"), &episode)?;
    let policy = b"independent-review-policy-v1\n";
    let package = EpisodePackage::load(&episode)?;
    let query = fs::read(episode.join("private/reference-query.sql"))?;
    let review = serde_json::json!({
        "schema_version": "0.6",
        "review_id": "review-aws-iam-004",
        "episode_id": "aws-iam-004",
        "reviewer_id": "reviewer-independent-001",
        "reviewed_at": "2026-08-09T00:00:00Z",
        "status": "approved",
        "public_package_sha256": hash_public_package(&episode.join("public"))?,
        "private_ground_truth_sha256": package.digests().private_ground_truth,
        "reference_query_sha256": Sha256Digest::from_bytes(&query),
        "review_policy_sha256": Sha256Digest::from_bytes(policy),
        "reason_codes": []
    });
    fs::write(
        episode.join("private/review.json"),
        serde_json::to_vec_pretty(&review)?,
    )?;
    let validation = validate_dataset_review(&episode, policy)?;
    assert_eq!(validation.status, DatasetReviewValidationStatus::Approved);

    let telemetry = episode.join("public/telemetry/cloudtrail.parquet");
    let mut changed = fs::read(&telemetry)?;
    changed.push(0);
    fs::write(telemetry, changed)?;
    let validation = validate_dataset_review(&episode, policy)?;
    assert_eq!(validation.status, DatasetReviewValidationStatus::Stale);
    Ok(())
}

#[test]
fn missing_and_stale_reviews_are_not_approved() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temporary = tempfile::tempdir()?;
    let episode = temporary.path().join("gcp-iam-004");
    copy_tree(&workspace.join("datasets/gcp/gcp-iam-004"), &episode)?;
    fs::remove_file(episode.join("private/review.json"))?;
    let missing = validate_dataset_review(&episode, b"policy")?;
    assert_eq!(missing.status, DatasetReviewValidationStatus::Missing);

    let review = fs::read(workspace.join("examples/contracts/v0.6/dataset-review-record.json"))?;
    fs::write(episode.join("private/review.json"), review)?;
    let stale = validate_dataset_review(&episode, b"policy")?;
    assert_eq!(stale.status, DatasetReviewValidationStatus::Stale);
    Ok(())
}

#[test]
fn canonical_r4_episode_reviews_bind_every_approved_artifact()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy = fs::read(workspace.join("policies/dataset-review-v1.json"))?;
    for provider in ["aws", "azure", "gcp"] {
        for suffix in ["004", "005", "006"] {
            let episode = workspace
                .join("datasets")
                .join(provider)
                .join(format!("{provider}-iam-{suffix}"));
            let validation = validate_dataset_review(&episode, &policy)?;
            assert_eq!(
                validation.status,
                DatasetReviewValidationStatus::Approved,
                "{}",
                episode.display()
            );
        }
    }
    Ok(())
}

#[test]
fn approved_review_becomes_stale_after_any_reviewed_private_change()
-> Result<(), Box<dyn std::error::Error>> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = workspace.join("datasets/aws/aws-iam-004");
    let policy = fs::read(workspace.join("policies/dataset-review-v1.json"))?;
    for relative in ["private/ground-truth.json", "private/reference-query.sql"] {
        let temporary = tempfile::tempdir()?;
        let episode = temporary.path().join("aws-iam-004");
        copy_tree(&source, &episode)?;
        assert_eq!(
            validate_dataset_review(&episode, &policy)?.status,
            DatasetReviewValidationStatus::Approved
        );
        let path = episode.join(relative);
        let mut changed = fs::read(&path)?;
        changed.extend_from_slice(b" ");
        fs::write(path, changed)?;
        assert_eq!(
            validate_dataset_review(&episode, &policy)?.status,
            DatasetReviewValidationStatus::Stale,
            "{relative}"
        );
    }
    Ok(())
}
