use std::{fs, fs::File};

use hunteval_fixture_tool::{
    ContributorCheckStatus, ContributorValidationStatus, ScaffoldRequest,
    build_review_bundle_manifest, generate_expanded_catalog, render_public_documentation,
    scaffold_episode, validate_episode,
};

#[test]
fn scaffold_is_non_overwriting_bounded_and_intentionally_incomplete()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let target = temporary.path().join("aws-iam-900");
    scaffold_episode(&ScaffoldRequest {
        provider: "aws",
        episode_id: "aws-iam-900",
        target: &target,
    })?;
    assert!(target.join("package.yaml").is_file());
    assert!(target.join("public/classification.json").is_file());
    let public = fs::read_to_string(target.join("public/classification.json"))?;
    assert!(!public.contains("ground_truth"));
    assert!(
        scaffold_episode(&ScaffoldRequest {
            provider: "aws",
            episode_id: "aws-iam-900",
            target: &target,
        })
        .is_err()
    );
    let validation = validate_episode(&target)?;
    assert_eq!(
        serde_json::to_value(&validation)?["status"],
        serde_json::json!("incomplete")
    );
    let documentation = String::from_utf8(render_public_documentation(&validation)?)?;
    assert!(!documentation.contains("ground-truth.json"));
    let bundle: serde_json::Value =
        serde_json::from_slice(&build_review_bundle_manifest(&target, &validation)?)?;
    assert_eq!(bundle["episode_id"], "aws-iam-900");
    Ok(())
}

#[test]
fn every_expanded_episode_is_valid_except_for_pending_independent_review()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    generate_expanded_catalog(temporary.path())?;
    for provider in ["aws", "azure", "gcp"] {
        for number in 7..=18 {
            let episode = temporary
                .path()
                .join(provider)
                .join(format!("{provider}-cloud-{number:03}"));
            let result = validate_episode(&episode)?;
            assert_eq!(result.status, ContributorValidationStatus::Incomplete);
            assert_eq!(
                check_status(&result, "independent_review"),
                Some(ContributorCheckStatus::Unavailable)
            );
            assert!(result.checks.iter().all(|check| {
                check.name == "independent_review" || check.status == ContributorCheckStatus::Passed
            }));
        }
    }
    Ok(())
}

#[test]
fn expanded_episode_validation_fails_closed_on_answer_leakage()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    generate_expanded_catalog(temporary.path())?;
    let episode = temporary.path().join("aws/aws-cloud-009");

    let clean = validate_episode(&episode)?;
    assert_eq!(
        check_status(&clean, "answer_leakage"),
        Some(ContributorCheckStatus::Passed)
    );

    let manifest = episode.join("public/manifest.yaml");
    let mut bytes = fs::read(&manifest)?;
    bytes.extend_from_slice(b"\n# evt-0006\n");
    fs::write(&manifest, bytes)?;
    let leaked = validate_episode(&episode)?;
    assert_eq!(
        check_status(&leaked, "answer_leakage"),
        Some(ContributorCheckStatus::Failed)
    );
    Ok(())
}

fn check_status(
    result: &hunteval_fixture_tool::ContributorValidationResult,
    name: &str,
) -> Option<ContributorCheckStatus> {
    result
        .checks
        .iter()
        .find(|check| check.name == name)
        .map(|check| check.status)
}

#[test]
fn scaffold_rejects_provider_identifier_and_symlink_parent()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    assert!(
        scaffold_episode(&ScaffoldRequest {
            provider: "unknown",
            episode_id: "episode-001",
            target: &temporary.path().join("unknown/episode-001"),
        })
        .is_err()
    );
    assert!(
        scaffold_episode(&ScaffoldRequest {
            provider: "aws",
            episode_id: "../escape",
            target: &temporary.path().join("aws/escape"),
        })
        .is_err()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(temporary.path(), temporary.path().join("linked"))?;
        assert!(
            scaffold_episode(&ScaffoldRequest {
                provider: "aws",
                episode_id: "aws-iam-901",
                target: &temporary.path().join("linked/aws-iam-901"),
            })
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn validation_rejects_symlinks_without_mutating_authored_files()
-> Result<(), Box<dyn std::error::Error>> {
    let temporary = tempfile::tempdir()?;
    let target = temporary.path().join("aws-iam-902");
    scaffold_episode(&ScaffoldRequest {
        provider: "aws",
        episode_id: "aws-iam-902",
        target: &target,
    })?;
    let before = fs::read(target.join("package.yaml"))?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            target.join("package.yaml"),
            target.join("public/unsafe-link"),
        )?;
        assert!(validate_episode(&target).is_err());
    }
    assert_eq!(before, fs::read(target.join("package.yaml"))?);
    Ok(())
}

#[test]
fn validation_rejects_oversized_and_malformed_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let oversized_root = tempfile::tempdir()?;
    generate_expanded_catalog(oversized_root.path())?;
    let oversized = oversized_root
        .path()
        .join("aws/aws-cloud-007/source/oversized.bin");
    File::create(oversized)?.set_len(64 * 1024 * 1024 + 1)?;
    assert!(validate_episode(&oversized_root.path().join("aws/aws-cloud-007")).is_err());

    let malformed_root = tempfile::tempdir()?;
    generate_expanded_catalog(malformed_root.path())?;
    let episode = malformed_root.path().join("azure/azure-cloud-009");
    let classification = episode.join("public/classification.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&classification)?)?;
    value["unexpected_field"] = serde_json::json!(true);
    fs::write(classification, serde_json::to_vec_pretty(&value)?)?;
    let validation = validate_episode(&episode)?;
    assert_eq!(
        check_status(&validation, "classification"),
        Some(ContributorCheckStatus::Failed)
    );
    Ok(())
}
