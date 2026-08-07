use std::{fs, io, path::Path, path::PathBuf};

use hunteval_domain::{
    BenchmarkCell, BenchmarkDefinition, BenchmarkDefinitionError, BenchmarkId, DeploymentId,
    EpisodeId, ResolvedArtifact, ResolvedDeployment, ResolvedEpisode, SchemaVersion,
    ScoringProfileId, Sha256Digest,
};

fn digest(byte: u8) -> Sha256Digest {
    Sha256Digest::from_bytes([byte])
}

fn definition(
    deployments: Vec<ResolvedDeployment>,
    episodes: Vec<ResolvedEpisode>,
    seeds: Vec<u64>,
) -> Result<BenchmarkDefinition, Box<dyn std::error::Error>> {
    Ok(BenchmarkDefinition::new(
        BenchmarkId::new("benchmark-001")?,
        deployments,
        episodes,
        seeds,
        ResolvedArtifact {
            id: ScoringProfileId::new("balanced:1.0.0")?,
            sha256: digest(9),
        },
        None,
    )?)
}

fn deployment(id: &str, byte: u8) -> Result<ResolvedDeployment, Box<dyn std::error::Error>> {
    Ok(ResolvedDeployment {
        configuration_sha256: digest(byte),
        id: DeploymentId::new(id)?,
    })
}

fn episode(id: &str, byte: u8) -> Result<ResolvedEpisode, Box<dyn std::error::Error>> {
    Ok(ResolvedEpisode {
        id: EpisodeId::new(id)?,
        package_sha256: digest(byte),
    })
}

#[test]
fn cell_identity_is_stable_and_order_independent() -> Result<(), Box<dyn std::error::Error>> {
    let first = definition(
        vec![
            deployment("deployment-b", 2)?,
            deployment("deployment-a", 1)?,
        ],
        vec![episode("episode-b", 4)?, episode("episode-a", 3)?],
        vec![29, 11],
    )?;
    let second = definition(
        vec![
            deployment("deployment-a", 1)?,
            deployment("deployment-b", 2)?,
        ],
        vec![episode("episode-a", 3)?, episode("episode-b", 4)?],
        vec![11, 29],
    )?;

    assert_eq!(first, second);
    assert_eq!(first.cells()?, second.cells()?);
    assert_eq!(first.cell_count()?, 8);
    Ok(())
}

#[test]
fn identity_changes_when_a_resolved_digest_changes() -> Result<(), Box<dyn std::error::Error>> {
    let first = definition(
        vec![deployment("deployment-a", 1)?],
        vec![episode("episode-a", 2)?],
        vec![11],
    )?;
    let second = definition(
        vec![deployment("deployment-a", 8)?],
        vec![episode("episode-a", 2)?],
        vec![11],
    )?;
    assert_ne!(first.cells()?[0].cell_id, second.cells()?[0].cell_id);
    Ok(())
}

#[test]
fn duplicate_dimensions_and_unknown_versions_are_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    let duplicated = definition(
        vec![
            deployment("deployment-a", 1)?,
            deployment("deployment-a", 2)?,
        ],
        vec![episode("episode-a", 3)?],
        vec![11],
    );
    assert!(
        matches!(duplicated, Err(error) if error.downcast_ref::<BenchmarkDefinitionError>().is_some())
    );

    let mut valid = definition(
        vec![deployment("deployment-a", 1)?],
        vec![episode("episode-a", 3)?],
        vec![11],
    )?;
    valid.schema_version = SchemaVersion::new(0, 5);
    assert!(matches!(
        valid.validate(),
        Err(BenchmarkDefinitionError::UnsupportedSchema(_))
    ));
    Ok(())
}

#[test]
fn v04_round_trip_preserves_the_derived_cell_identifier() -> Result<(), Box<dyn std::error::Error>>
{
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("domain crate is not inside the workspace"))?;
    let bytes = fs::read(root.join("examples/contracts/v0.4/benchmark-cell.json"))?;
    let stored: BenchmarkCell = serde_json::from_slice(&bytes)?;
    let derived = BenchmarkCell::from_key(stored.key.clone())?;
    assert_eq!(derived, stored);
    assert_eq!(
        serde_json::from_slice::<BenchmarkCell>(&serde_json::to_vec(&stored)?)?,
        stored
    );
    Ok(())
}
