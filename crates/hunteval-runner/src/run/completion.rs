use std::path::Path;

use hunteval_domain::{RunId, SchemaVersion};
use serde::Serialize;

use crate::{ArtifactWriter, RunManifest};

use super::{
    ResolvedRunInputs,
    engine::PendingSuccess,
    types::{RunArtifacts, RunExecution, RunFailure, RunFailureKind, RunRequest},
};

pub(super) fn finalize_success(
    success: PendingSuccess,
    writer: ArtifactWriter,
    inputs: &ResolvedRunInputs,
    run_id: &RunId,
) -> Result<RunExecution, RunFailure> {
    let mut hashes = inputs.hashes.clone();
    let evaluated_hashes = success.evaluated_hashes;
    let trajectory = writer.partial_root().join("trajectory.jsonl");
    let submission = writer.partial_root().join("submission.json");
    for (name, path) in [
        ("trajectory", trajectory),
        ("submission", submission),
        ("metrics", writer.partial_root().join("metrics.json")),
        (
            "aggregate_score",
            writer.partial_root().join("aggregate-score.json"),
        ),
    ] {
        let digest = crate::hash_file(&path).map_err(|_| artifact_failure(&writer))?;
        if (name == "trajectory" && digest != evaluated_hashes.trajectory)
            || (name == "submission" && digest != evaluated_hashes.submission)
        {
            return Err(artifact_failure(&writer));
        }
        hashes.insert(name.to_owned(), digest);
    }
    let manifest = RunManifest {
        schema_version: SchemaVersion::new(0, 4),
        run_id: run_id.clone(),
        hashes: hashes.clone(),
        partial: false,
    };
    if writer
        .write_json(Path::new("manifest.json"), &manifest)
        .is_err()
    {
        return Err(artifact_failure(&writer));
    }
    let partial_root = writer.partial_root().to_path_buf();
    let root = writer.finalize().map_err(|_| RunFailure {
        kind: RunFailureKind::Artifact,
        partial_artifacts: partial_root,
    })?;
    Ok(RunExecution {
        submission: success.submission,
        metrics: success.metrics,
        aggregate_score: success.aggregate_score,
        usage: success.usage,
        artifacts: RunArtifacts { root, hashes },
    })
}

pub(super) fn preserve_failure(
    kind: RunFailureKind,
    writer: ArtifactWriter,
    request: &RunRequest,
    inputs: &ResolvedRunInputs,
) -> Result<RunExecution, RunFailure> {
    let partial_artifacts = writer.partial_root().to_path_buf();
    let _ = writer.write_json(Path::new("failure.json"), &FailureArtifact { kind });
    let _ = writer.write_json(
        Path::new("manifest.json"),
        &RunManifest {
            schema_version: SchemaVersion::new(0, 4),
            run_id: request.run_id.clone(),
            hashes: inputs.hashes.clone(),
            partial: true,
        },
    );
    Err(RunFailure {
        kind,
        partial_artifacts,
    })
}

fn artifact_failure(writer: &ArtifactWriter) -> RunFailure {
    RunFailure {
        kind: RunFailureKind::Artifact,
        partial_artifacts: writer.partial_root().to_path_buf(),
    }
}

#[derive(Serialize)]
struct FailureArtifact {
    kind: RunFailureKind,
}
