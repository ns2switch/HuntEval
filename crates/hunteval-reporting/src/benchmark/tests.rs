use std::collections::BTreeMap;

use hunteval_domain::{
    BenchmarkDefinition, BenchmarkId, DeploymentId, EpisodeId, ResolvedArtifact,
    ResolvedDeployment, ResolvedEpisode, SchemaVersion, ScoringProfileId, Sha256Digest,
};
use hunteval_statistics::StatisticalSummary;

use super::{
    BenchmarkArtifact, BenchmarkCellSummary, BenchmarkClaim, BenchmarkClaimSource,
    BenchmarkDeploymentSummary, BenchmarkJsonRenderer, BenchmarkRankingGroup, BenchmarkResult,
    BenchmarkStaticHtmlRenderer,
};

#[test]
fn normalized_json_is_deterministic_and_ends_with_newline() -> Result<(), Box<dyn std::error::Error>>
{
    let report = fixture()?;
    let first = BenchmarkJsonRenderer.render(&report)?;
    let second = BenchmarkJsonRenderer.render(&report)?;
    assert_eq!(first, second);
    assert!(first.ends_with(b"\n"));
    let decoded: BenchmarkResult = serde_json::from_slice(&first)?;
    assert_eq!(decoded, report);
    Ok(())
}

#[test]
fn html_is_static_and_escapes_untrusted_content() -> Result<(), Box<dyn std::error::Error>> {
    let mut report = fixture()?;
    report.limitations = vec!["<script>alert(1)</script>".to_owned()];
    let html = String::from_utf8(BenchmarkStaticHtmlRenderer.render(&report)?)?;
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("Attribution is observational"));
    Ok(())
}

#[test]
fn claims_require_observable_sources() -> Result<(), Box<dyn std::error::Error>> {
    let mut report = fixture()?;
    report.claims[0].sources.clear();
    assert!(report.validate().is_err());
    Ok(())
}

fn fixture() -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let definition = BenchmarkDefinition::new(
        BenchmarkId::new("report-test")?,
        vec![ResolvedDeployment {
            id: DeploymentId::new("deployment-a")?,
            configuration_sha256: digest(b"deployment"),
        }],
        vec![ResolvedEpisode {
            id: EpisodeId::new("episode-a")?,
            package_sha256: digest(b"episode"),
        }],
        vec![11],
        ResolvedArtifact {
            id: ScoringProfileId::new("profile")?,
            sha256: digest(b"profile"),
        },
        None,
    )?;
    let cell = definition.cells()?.remove(0);
    let summary = StatisticalSummary {
        count: 1,
        mean: Some(0.75),
        interval: None,
    };
    Ok(BenchmarkResult {
        schema_version: SchemaVersion::new(0, 4),
        benchmark_id: definition.id.clone(),
        benchmark_definition_sha256: digest(b"definition"),
        benchmark_state_sha256: digest(b"state"),
        scoring_profile_sha256: definition.scoring_profile.sha256,
        cells: vec![BenchmarkCellSummary {
            cell_id: cell.cell_id,
            deployment_id: cell.key.deployment.id.clone(),
            episode_id: cell.key.episode.id,
            seed: cell.key.seed,
            status: "completed".to_owned(),
            reason_code: None,
            run_id: None,
            result_sha256: None,
            aggregate_score: Some(0.75),
            aggregate_score_omissions: BTreeMap::new(),
            metrics: BTreeMap::new(),
            constraints: Vec::new(),
            resource_usage: None,
            submitted_timeline: Vec::new(),
            artifacts: Vec::new(),
        }],
        deployments: vec![BenchmarkDeploymentSummary {
            deployment_id: cell.key.deployment.id.clone(),
            completed_cells: 1,
            failed_cells: 0,
            pending_cells: 0,
            non_comparable_cells: 0,
            disqualifying_constraints: 0,
            aggregate_score: summary,
            metrics: BTreeMap::new(),
        }],
        comparisons: Vec::new(),
        rankings: vec![BenchmarkRankingGroup {
            rank: 1,
            deployments: vec![cell.key.deployment.id],
            disqualifying_constraints: 0,
            aggregate_score: Some(0.75),
        }],
        claims: vec![BenchmarkClaim {
            claim_id: "deployment:deployment-a".to_owned(),
            text: "Deployment result.".to_owned(),
            sources: vec![BenchmarkClaimSource::BenchmarkCell {
                benchmark_id: definition.id,
                cell_id: cell.cell_id,
            }],
        }],
        artifacts: vec![BenchmarkArtifact {
            path: "benchmark-state.json".to_owned(),
            sha256: digest(b"state"),
        }],
        limitations: vec!["Attribution is observational.".to_owned()],
    })
}

fn digest(bytes: &[u8]) -> Sha256Digest {
    Sha256Digest::from_bytes(bytes)
}
