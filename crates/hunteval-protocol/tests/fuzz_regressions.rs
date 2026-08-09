use std::{fs, io, path::PathBuf};

use hunteval_protocol::{JsonlDecoder, replay_trajectory};

#[test]
fn retained_public_fuzz_corpus_has_stable_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let malformed = fs::read(root.join("fuzz/corpus/jsonl_decoder/malformed-json"))?;
    let partial = fs::read(root.join("fuzz/corpus/trajectory_replay/partial-event"))?;
    assert!(JsonlDecoder::new(128 * 1024)?.decode(&malformed).is_err());
    assert!(replay_trajectory(&partial, 128 * 1024).is_err());
    Ok(())
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))
}
