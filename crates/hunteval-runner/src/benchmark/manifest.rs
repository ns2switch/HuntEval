use std::{collections::BTreeSet, fs, path::Component, path::Path, path::PathBuf};

use hunteval_domain::{BenchmarkId, SchemaVersion};
use serde::{Deserialize, Serialize};

use super::BenchmarkError;

const V03: SchemaVersion = SchemaVersion::new(0, 3);
const V04: SchemaVersion = SchemaVersion::new(0, 4);

/// Human-authored benchmark manifest containing safe relative artifact references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredBenchmarkManifest {
    pub schema_version: SchemaVersion,
    pub id: BenchmarkId,
    pub deployments: Vec<String>,
    pub episodes: Vec<String>,
    pub seeds: Vec<u64>,
    pub repetitions: Option<usize>,
    pub scoring_profile: String,
    #[serde(default)]
    pub fault_profile: Option<String>,
}

/// Compatibility cell using authored references rather than resolved identities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredRunCell {
    pub deployment: String,
    pub episode: String,
    pub seed: u64,
}

/// Compatibility name retained for callers of the v0.3 runner API.
pub type BenchmarkManifest = AuthoredBenchmarkManifest;

/// Compatibility name retained for callers of the v0.3 runner API.
pub type RunCell = AuthoredRunCell;

impl AuthoredBenchmarkManifest {
    /// Validates authored fields without resolving filesystem artifacts.
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if !matches!(self.schema_version, V03 | V04) {
            return Err(BenchmarkError::UnsupportedSchema(self.schema_version));
        }
        if self.deployments.is_empty() || self.episodes.is_empty() || self.seeds.is_empty() {
            return Err(BenchmarkError::EmptyDimension);
        }
        if self
            .repetitions
            .is_some_and(|count| count != self.seeds.len())
        {
            return Err(BenchmarkError::RepetitionMismatch);
        }
        if self.schema_version == V04 && self.repetitions.is_some() {
            return Err(BenchmarkError::IncompatibleField("repetitions"));
        }
        if self.schema_version == V03 && self.fault_profile.is_some() {
            return Err(BenchmarkError::IncompatibleField("fault_profile"));
        }
        require_unique(&self.deployments, "deployment")?;
        require_unique(&self.episodes, "episode")?;
        require_unique(&self.seeds, "seed")?;
        for value in self
            .deployments
            .iter()
            .chain(&self.episodes)
            .chain(std::iter::once(&self.scoring_profile))
            .chain(self.fault_profile.iter())
        {
            validate_path(value)?;
        }
        Ok(())
    }

    /// Preserves the v0.3 authored Cartesian-product API for CLI validation.
    #[must_use]
    pub fn cells(&self) -> Vec<AuthoredRunCell> {
        let mut cells = Vec::new();
        for deployment in &self.deployments {
            for episode in &self.episodes {
                for seed in &self.seeds {
                    cells.push(AuthoredRunCell {
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

/// Loads and validates a v0.3 or v0.4 authored benchmark manifest.
pub fn load_benchmark(path: &Path) -> Result<AuthoredBenchmarkManifest, BenchmarkError> {
    let bytes = fs::read(path)?;
    let manifest: AuthoredBenchmarkManifest =
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
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        Err(BenchmarkError::UnsafePath)
    } else {
        Ok(())
    }
}

fn require_unique<T: Ord>(values: &[T], dimension: &'static str) -> Result<(), BenchmarkError> {
    let mut unique = BTreeSet::new();
    for value in values {
        if !unique.insert(value) {
            return Err(BenchmarkError::DuplicateDimension(dimension));
        }
    }
    Ok(())
}
