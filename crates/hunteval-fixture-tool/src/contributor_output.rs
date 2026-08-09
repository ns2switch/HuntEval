use std::path::Path;

use hunteval_domain::{EpisodeId, Sha256Digest};
use serde::Serialize;

use crate::{ContributorValidationResult, FixtureGenerationError, contributor::bounded_read};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReviewVisibility {
    Public,
    ReviewerOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReviewFile {
    label: String,
    sha256: Sha256Digest,
    visibility: ReviewVisibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReviewBundleManifest {
    schema_version: String,
    episode_id: EpisodeId,
    validation_result_sha256: Sha256Digest,
    files: Vec<ReviewFile>,
}

/// Renders deterministic public documentation containing only safe validation metadata.
pub fn render_public_documentation(
    result: &ContributorValidationResult,
) -> Result<Vec<u8>, FixtureGenerationError> {
    let mut output = format!(
        "# Episode {}\n\nValidation status: `{:?}`\n\n## Checks\n\n",
        result.episode_id, result.status
    );
    for check in &result.checks {
        output.push_str(&format!(
            "- `{}`: `{:?}`{}\n",
            check.name,
            check.status,
            check
                .reason_code
                .as_deref()
                .map(|reason| format!(" (`{reason}`)"))
                .unwrap_or_default()
        ));
    }
    Ok(output.to_ascii_lowercase().into_bytes())
}

/// Builds a content-addressed reviewer inventory without exposing private bytes.
pub fn build_review_bundle_manifest(
    root: &Path,
    result: &ContributorValidationResult,
) -> Result<Vec<u8>, FixtureGenerationError> {
    let validation_bytes = serde_json::to_vec(result)?;
    let files = [
        (
            "public_manifest",
            "public/manifest.yaml",
            ReviewVisibility::Public,
        ),
        (
            "public_classification",
            "public/classification.json",
            ReviewVisibility::Public,
        ),
        (
            "private_ground_truth",
            "private/ground-truth.json",
            ReviewVisibility::ReviewerOnly,
        ),
        (
            "private_reference_query",
            "private/reference-query.sql",
            ReviewVisibility::ReviewerOnly,
        ),
        (
            "source_events",
            "source/events.json",
            ReviewVisibility::ReviewerOnly,
        ),
    ]
    .into_iter()
    .filter_map(|(label, relative, visibility)| {
        let path = root.join(relative);
        path.is_file().then_some((label, path, visibility))
    })
    .map(|(label, path, visibility)| {
        Ok(ReviewFile {
            label: label.to_owned(),
            sha256: Sha256Digest::from_bytes(bounded_read(&path)?),
            visibility,
        })
    })
    .collect::<Result<Vec<_>, FixtureGenerationError>>()?;
    if files.is_empty() {
        return Err(FixtureGenerationError::MalformedContributorPackage);
    }
    let manifest = ReviewBundleManifest {
        schema_version: "0.6".to_owned(),
        episode_id: result.episode_id.clone(),
        validation_result_sha256: Sha256Digest::from_bytes(validation_bytes),
        files,
    };
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    Ok(bytes)
}
