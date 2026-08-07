use std::{collections::BTreeSet, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use thiserror::Error;

use crate::{
    BenchmarkId, DeploymentId, DigestParseError, EpisodeId, FaultProfileId, SchemaVersion,
    ScoringProfileId, Sha256Digest,
};

const BENCHMARK_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(0, 4);
const MAX_BENCHMARK_CELLS: usize = 1_000_000;
const CELL_ID_PREFIX: &str = "cell:";

/// A deployment identity and the digest of its complete resolved configuration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedDeployment {
    pub configuration_sha256: Sha256Digest,
    pub id: DeploymentId,
}

/// An episode identity and the digest of its trusted package.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedEpisode {
    pub id: EpisodeId,
    pub package_sha256: Sha256Digest,
}

/// A named versioned artifact resolved to exact bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedArtifact<I> {
    pub id: I,
    pub sha256: Sha256Digest,
}

/// Infrastructure-independent definition of one benchmark matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDefinition {
    pub schema_version: SchemaVersion,
    pub id: BenchmarkId,
    pub deployments: Vec<ResolvedDeployment>,
    pub episodes: Vec<ResolvedEpisode>,
    pub seeds: Vec<u64>,
    pub scoring_profile: ResolvedArtifact<ScoringProfileId>,
    pub fault_profile: Option<ResolvedArtifact<FaultProfileId>>,
}

impl BenchmarkDefinition {
    /// Validates and canonicalizes resolved benchmark inputs.
    pub fn new(
        id: BenchmarkId,
        mut deployments: Vec<ResolvedDeployment>,
        mut episodes: Vec<ResolvedEpisode>,
        mut seeds: Vec<u64>,
        scoring_profile: ResolvedArtifact<ScoringProfileId>,
        fault_profile: Option<ResolvedArtifact<FaultProfileId>>,
    ) -> Result<Self, BenchmarkDefinitionError> {
        require_unique(deployments.iter().map(|item| &item.id), "deployment")?;
        require_unique(episodes.iter().map(|item| &item.id), "episode")?;
        require_unique(seeds.iter(), "seed")?;
        deployments.sort();
        episodes.sort();
        seeds.sort_unstable();
        let definition = Self {
            schema_version: BENCHMARK_SCHEMA_VERSION,
            id,
            deployments,
            episodes,
            seeds,
            scoring_profile,
            fault_profile,
        };
        definition.validate()?;
        Ok(definition)
    }

    /// Validates a deserialized definition before it enters application services.
    pub fn validate(&self) -> Result<(), BenchmarkDefinitionError> {
        if self.schema_version != BENCHMARK_SCHEMA_VERSION {
            return Err(BenchmarkDefinitionError::UnsupportedSchema(
                self.schema_version,
            ));
        }
        if self.deployments.is_empty() || self.episodes.is_empty() || self.seeds.is_empty() {
            return Err(BenchmarkDefinitionError::EmptyDimension);
        }
        require_unique(self.deployments.iter().map(|item| &item.id), "deployment")?;
        require_unique(self.episodes.iter().map(|item| &item.id), "episode")?;
        require_unique(self.seeds.iter(), "seed")?;
        self.cell_count()?;
        Ok(())
    }

    /// Returns the bounded Cartesian product size without allocating it.
    pub fn cell_count(&self) -> Result<usize, BenchmarkDefinitionError> {
        let count = self
            .deployments
            .len()
            .checked_mul(self.episodes.len())
            .and_then(|value| value.checked_mul(self.seeds.len()))
            .ok_or(BenchmarkDefinitionError::MatrixTooLarge)?;
        if count > MAX_BENCHMARK_CELLS {
            return Err(BenchmarkDefinitionError::MatrixTooLarge);
        }
        Ok(count)
    }

    /// Expands the canonical matrix and derives stable cell identifiers.
    pub fn cells(&self) -> Result<Vec<BenchmarkCell>, BenchmarkDefinitionError> {
        self.validate()?;
        let mut cells = Vec::with_capacity(self.cell_count()?);
        for deployment in &self.deployments {
            for episode in &self.episodes {
                for seed in &self.seeds {
                    let key = BenchmarkCellKey {
                        benchmark_id: self.id.clone(),
                        deployment: deployment.clone(),
                        episode: episode.clone(),
                        fault_profile: self.fault_profile.clone(),
                        scoring_profile: self.scoring_profile.clone(),
                        seed: *seed,
                    };
                    cells.push(BenchmarkCell::from_key(key)?);
                }
            }
        }
        Ok(cells)
    }
}

/// Canonically serialized identity inputs for one benchmark cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCellKey {
    pub benchmark_id: BenchmarkId,
    pub deployment: ResolvedDeployment,
    pub episode: ResolvedEpisode,
    pub fault_profile: Option<ResolvedArtifact<FaultProfileId>>,
    pub scoring_profile: ResolvedArtifact<ScoringProfileId>,
    pub seed: u64,
}

/// One resolved benchmark cell and its stable content-derived identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCell {
    pub schema_version: SchemaVersion,
    pub cell_id: BenchmarkCellId,
    pub key: BenchmarkCellKey,
}

impl BenchmarkCell {
    /// Derives a cell identifier from canonical compact JSON bytes.
    pub fn from_key(key: BenchmarkCellKey) -> Result<Self, BenchmarkDefinitionError> {
        let canonical =
            serde_json::to_vec(&key).map_err(BenchmarkDefinitionError::CanonicalJson)?;
        Ok(Self {
            schema_version: BENCHMARK_SCHEMA_VERSION,
            cell_id: BenchmarkCellId(Sha256Digest::from_bytes(canonical)),
            key,
        })
    }
}

/// A stable benchmark cell identifier derived from its canonical key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BenchmarkCellId(Sha256Digest);

impl fmt::Display for BenchmarkCellId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{CELL_ID_PREFIX}{}", self.0)
    }
}

impl FromStr for BenchmarkCellId {
    type Err = BenchmarkCellIdParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let digest = value
            .strip_prefix(CELL_ID_PREFIX)
            .ok_or(BenchmarkCellIdParseError::InvalidPrefix)?;
        if !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(BenchmarkCellIdParseError::NonCanonicalDigest);
        }
        Ok(Self(digest.parse()?))
    }
}

impl Serialize for BenchmarkCellId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BenchmarkCellId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

fn require_unique<T, I>(values: I, dimension: &'static str) -> Result<(), BenchmarkDefinitionError>
where
    T: Ord + Clone,
    I: IntoIterator<Item = T>,
{
    let mut unique = BTreeSet::new();
    for value in values {
        if !unique.insert(value) {
            return Err(BenchmarkDefinitionError::Duplicate(dimension));
        }
    }
    Ok(())
}

/// Failures while validating or expanding a resolved benchmark definition.
#[derive(Debug, Error)]
pub enum BenchmarkDefinitionError {
    #[error("benchmark schema version {0} is unsupported")]
    UnsupportedSchema(SchemaVersion),
    #[error("benchmark dimensions must not be empty")]
    EmptyDimension,
    #[error("benchmark contains a duplicate {0}")]
    Duplicate(&'static str),
    #[error("benchmark matrix exceeds the supported cell bound")]
    MatrixTooLarge,
    #[error("benchmark cell key could not be serialized canonically: {0}")]
    CanonicalJson(serde_json::Error),
}

/// Failures while parsing a benchmark cell identifier.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BenchmarkCellIdParseError {
    #[error("benchmark cell identifier must start with cell:")]
    InvalidPrefix,
    #[error("benchmark cell digest must use lowercase hexadecimal")]
    NonCanonicalDigest,
    #[error("benchmark cell digest is invalid: {0}")]
    Digest(#[from] DigestParseError),
}
