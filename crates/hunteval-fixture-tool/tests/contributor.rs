use std::fs;

use hunteval_fixture_tool::{
    ScaffoldRequest, build_review_bundle_manifest, render_public_documentation, scaffold_episode,
    validate_episode,
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
