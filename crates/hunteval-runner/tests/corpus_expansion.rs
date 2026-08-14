use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use hunteval_domain::InvestigationShape;
use hunteval_duckdb::{
    QueryLimits, SqlRequest, SqlValue, TableRegistration, WorkerCommand, execute_command,
};
use hunteval_runner::{DatasetReviewValidationStatus, EpisodePackage, validate_dataset_review};

const PROVIDERS: [&str; 3] = ["aws", "azure", "gcp"];

#[test]
fn expanded_corpus_contracts_and_private_separation_hold() -> Result<(), Box<dyn std::error::Error>>
{
    let datasets = workspace()?.join("datasets");
    let review_policy = fs::read(workspace()?.join("policies/dataset-review-v1.json"))?;
    let mut benign = 0;
    let mut multi_stage = 0;
    let mut cross_boundary = 0;

    for provider in PROVIDERS {
        for number in 7..=18 {
            let id = format!("{provider}-cloud-{number:03}");
            let root = datasets.join(provider).join(&id);
            let package = EpisodePackage::load(&root)?;
            assert_eq!(package.public().manifest.id.as_str(), id);
            assert!(package.public().classification.is_some());
            assert!(root.join("private/review-bundle.json").is_file());
            assert!(!root.join("private/review.json").exists());
            assert_eq!(
                validate_dataset_review(&root, &review_policy)?.status,
                DatasetReviewValidationStatus::Missing
            );

            let truth = package.ground_truth();
            if matches!(number, 7 | 8) {
                benign += 1;
                assert!(truth.is_benign_scored_episode());
                assert!(truth.malicious_event_ids.is_empty());
                assert!(truth.malicious_entity_ids.is_empty());
            }

            let classification = package
                .public()
                .classification
                .as_ref()
                .ok_or("classification unavailable")?;
            if classification
                .investigation_shapes
                .contains(&InvestigationShape::MultiStage)
            {
                multi_stage += 1;
                assert!(truth.expected_attack_path.len() >= 3);
            }
            if classification
                .investigation_shapes
                .contains(&InvestigationShape::CrossBoundary)
            {
                cross_boundary += 1;
                let events: serde_json::Value =
                    serde_json::from_slice(&fs::read(root.join("source/events.json"))?)?;
                let boundaries = events
                    .as_array()
                    .ok_or("source event array unavailable")?
                    .iter()
                    .filter_map(|event| event["account_id"].as_str())
                    .collect::<BTreeSet<_>>();
                assert!(boundaries.len() >= 2);
            }

            let public_metadata = [
                "public/manifest.yaml",
                "public/classification.json",
                "public/provenance.json",
            ]
            .into_iter()
            .map(|relative| fs::read_to_string(root.join(relative)))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
            .to_ascii_lowercase();
            for private_token in [
                "ground_truth",
                "ground-truth",
                "reference_query",
                "reference-query",
                "private/",
            ] {
                assert!(!public_metadata.contains(private_token));
            }
        }
    }

    assert_eq!(benign, 6);
    assert_eq!(multi_stage, 24);
    assert_eq!(cross_boundary, 12);
    Ok(())
}

#[test]
fn every_expanded_reference_query_recovers_exact_private_event_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let datasets = workspace()?.join("datasets");
    for provider in PROVIDERS {
        for number in 7..=18 {
            let id = format!("{provider}-cloud-{number:03}");
            let root = datasets.join(provider).join(&id);
            let package = EpisodePackage::load(&root)?;
            let result = execute_command(WorkerCommand {
                tables: vec![TableRegistration {
                    name: table(provider).to_owned(),
                    parquet_path: root.join("public/telemetry").join(telemetry(provider)),
                }],
                request: SqlRequest {
                    query: fs::read_to_string(root.join("private/reference-query.sql"))?,
                    parameters: Vec::new(),
                    limits: QueryLimits::default(),
                },
            })?;
            let recovered = result
                .rows
                .into_iter()
                .filter_map(|row| match row.first() {
                    Some(SqlValue::String(value)) => Some(value.clone()),
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            let expected = package
                .ground_truth()
                .malicious_event_ids
                .iter()
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>();
            assert_eq!(recovered, expected, "{id}");
            assert!(!result.truncated);
        }
    }
    Ok(())
}

fn workspace() -> Result<PathBuf, io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("workspace unavailable"))
}

fn table(provider: &str) -> &str {
    match provider {
        "aws" => "aws_cloudtrail",
        "azure" => "azure_activity",
        _ => "gcp_audit",
    }
}

fn telemetry(provider: &str) -> &str {
    match provider {
        "aws" => "cloudtrail.parquet",
        "azure" => "activity.parquet",
        _ => "audit.parquet",
    }
}
