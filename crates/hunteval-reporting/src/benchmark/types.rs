use std::collections::BTreeMap;

use hunteval_domain::{
    BenchmarkCellId, BenchmarkId, DeploymentId, EpisodeId, MetricValue, ResourceUsage, RunId,
    SchemaVersion, Sha256Digest, TimelineEntry,
};
use hunteval_statistics::{PairedDifference, StatisticalSummary};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDigestReference {
    pub artifact: String,
    pub sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkArtifact {
    pub path: String,
    pub sha256: Sha256Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkConstraintSummary {
    pub code: String,
    pub status: String,
    pub disqualifying: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCellSummary {
    pub cell_id: BenchmarkCellId,
    pub deployment_id: DeploymentId,
    pub episode_id: EpisodeId,
    pub seed: u64,
    pub status: String,
    pub reason_code: Option<String>,
    pub run_id: Option<RunId>,
    pub result_sha256: Option<Sha256Digest>,
    pub aggregate_score: Option<f64>,
    pub aggregate_score_omissions: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, MetricValue>,
    pub constraints: Vec<BenchmarkConstraintSummary>,
    pub resource_usage: Option<ResourceUsage>,
    pub submitted_timeline: Vec<TimelineEntry>,
    pub artifacts: Vec<ArtifactDigestReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMetricSummary {
    pub metric: String,
    pub values: BTreeMap<BenchmarkCellId, Option<f64>>,
    pub statistics: StatisticalSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDeploymentSummary {
    pub deployment_id: DeploymentId,
    pub completed_cells: usize,
    pub failed_cells: usize,
    pub pending_cells: usize,
    pub non_comparable_cells: usize,
    pub disqualifying_constraints: usize,
    pub aggregate_score: StatisticalSummary,
    pub metrics: BTreeMap<String, BenchmarkMetricSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPairwiseComparison {
    pub comparison_id: String,
    pub left: DeploymentId,
    pub right: DeploymentId,
    pub eligible: bool,
    pub reasons: Vec<String>,
    pub cell_ids: Vec<BenchmarkCellId>,
    pub aggregate_difference: PairedDifference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRankingGroup {
    pub rank: usize,
    pub deployments: Vec<DeploymentId>,
    pub disqualifying_constraints: usize,
    pub aggregate_score: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BenchmarkClaimSource {
    BenchmarkCell {
        benchmark_id: BenchmarkId,
        cell_id: BenchmarkCellId,
    },
    MetricPointer {
        run_id: RunId,
        pointer: String,
    },
    Constraint {
        scope_id: String,
        constraint_id: String,
    },
    StatisticalComparison {
        comparison_id: String,
    },
    ArtifactDigest {
        artifact: String,
        sha256: Sha256Digest,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkClaim {
    pub claim_id: String,
    pub text: String,
    pub sources: Vec<BenchmarkClaimSource>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkResult {
    pub schema_version: SchemaVersion,
    pub benchmark_id: BenchmarkId,
    pub benchmark_definition_sha256: Sha256Digest,
    pub benchmark_state_sha256: Sha256Digest,
    pub scoring_profile_sha256: Sha256Digest,
    pub cells: Vec<BenchmarkCellSummary>,
    pub deployments: Vec<BenchmarkDeploymentSummary>,
    pub comparisons: Vec<BenchmarkPairwiseComparison>,
    pub rankings: Vec<BenchmarkRankingGroup>,
    pub claims: Vec<BenchmarkClaim>,
    pub artifacts: Vec<BenchmarkArtifact>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Error)]
pub enum BenchmarkResultError {
    #[error("benchmark result contract is invalid")]
    InvalidContract,
    #[error("benchmark result contains an invalid claim source")]
    InvalidSource,
    #[error("benchmark report serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
}
