use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkManifest {
    pub schema_version: String,
    pub id: String,
    pub deployments: Vec<String>,
    pub episodes: Vec<String>,
    pub seeds: Vec<u64>,
    pub repetitions: Option<usize>,
    pub scoring_profile: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunCell {
    pub deployment: String,
    pub episode: String,
    pub seed: u64,
}

impl BenchmarkManifest {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_version != "0.3"
            || self.id.trim().is_empty()
            || self.deployments.is_empty()
            || self.episodes.is_empty()
            || self.seeds.is_empty()
        {
            return Err(BenchmarkError::InvalidManifest);
        }
        if self
            .repetitions
            .is_some_and(|count| count != self.seeds.len())
        {
            return Err(BenchmarkError::RepetitionMismatch);
        }
        for value in self
            .deployments
            .iter()
            .chain(&self.episodes)
            .chain(std::iter::once(&self.scoring_profile))
        {
            validate_path(value)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn cells(&self) -> Vec<RunCell> {
        let mut cells = Vec::new();
        for deployment in &self.deployments {
            for episode in &self.episodes {
                for seed in &self.seeds {
                    cells.push(RunCell {
                        deployment: deployment.clone(),
                        episode: episode.clone(),
                        seed: *seed,
                    });
                }
            }
        }
        cells
    }
}

pub fn load_benchmark(path: &Path) -> Result<BenchmarkManifest, BenchmarkError> {
    let bytes = fs::read(path).map_err(BenchmarkError::Io)?;
    let manifest: BenchmarkManifest =
        serde_yaml_ng::from_slice(&bytes).map_err(|_| BenchmarkError::InvalidManifest)?;
    manifest.validate()?;
    Ok(manifest)
}

fn validate_path(value: &str) -> Result<(), BenchmarkError> {
    let path = PathBuf::from(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        Err(BenchmarkError::UnsafePath)
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("benchmark manifest is invalid")]
    InvalidManifest,
    #[error("benchmark repetitions must equal the listed seed count")]
    RepetitionMismatch,
    #[error("benchmark contains an unsafe path")]
    UnsafePath,
    #[error("benchmark could not be read: {0}")]
    Io(std::io::Error),
}
