use std::{collections::BTreeMap, path::Path};

use hunteval_domain::{RunId, SchemaVersion};
use hunteval_runner::{ArtifactWriter, RunManifest};

#[test]
fn finalizes_atomically_and_preserves_partial_runs() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let run_id = RunId::new("run-artifacts")?;
    let writer = ArtifactWriter::create(root.path(), &run_id)?;
    writer.append(Path::new("trajectory.jsonl"), b"one\n")?;
    assert!(writer.partial_root().ends_with("run-artifacts.partial"));
    let final_root = writer.finalize()?;
    assert!(final_root.join("trajectory.jsonl").is_file());

    let partial_id = RunId::new("run-interrupted")?;
    let partial = ArtifactWriter::create(root.path(), &partial_id)?;
    partial.write_json(
        Path::new("manifest.json"),
        &RunManifest {
            schema_version: SchemaVersion::new(0, 3),
            run_id: partial_id,
            hashes: BTreeMap::new(),
            partial: true,
        },
    )?;
    assert!(partial.partial_root().join("manifest.json").is_file());
    Ok(())
}
