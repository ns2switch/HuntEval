use std::{fs, io, path::PathBuf};

use hunteval_domain::{ProtocolVersion, Sha256Digest};
use hunteval_protocol::{ProtocolEnvelope, TrajectoryRecorder, replay_trajectory};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityManifest {
    schema_version: String,
    supported: Vec<SupportedFixture>,
    required_rejections: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SupportedFixture {
    protocol_version: ProtocolVersion,
    positive_fixture: String,
    sha256: Sha256Digest,
}

#[test]
fn supported_protocol_inventory_is_exact_and_content_addressed()
-> Result<(), Box<dyn std::error::Error>> {
    let root = workspace_root()?;
    let manifest_path = root.join("examples/contracts/protocol/compatibility-manifest.json");
    let manifest: CompatibilityManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    assert_eq!(manifest.schema_version, "0.5");
    assert_eq!(manifest.supported.len(), 1);
    assert_eq!(manifest.required_rejections.len(), 7);

    let fixture = &manifest.supported[0];
    assert_eq!(fixture.protocol_version, ProtocolVersion::new(0, 3));
    let path = manifest_path
        .parent()
        .ok_or_else(|| io::Error::other("compatibility manifest has no parent"))?
        .join(&fixture.positive_fixture);
    let bytes = fs::read(path)?;
    assert_eq!(Sha256Digest::from_bytes(&bytes), fixture.sha256);
    let messages: Vec<ProtocolEnvelope> = serde_json::from_slice(&bytes)?;
    let mut trajectory = TrajectoryRecorder::new();
    for message in messages {
        trajectory.append(message)?;
    }
    replay_trajectory(trajectory.as_bytes(), 128 * 1024)?;
    Ok(())
}

fn workspace_root() -> Result<PathBuf, io::Error> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("workspace root is unavailable"))
}
