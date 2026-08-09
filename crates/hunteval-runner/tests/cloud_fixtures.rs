use std::{io, path::PathBuf};

use hunteval_domain::InvestigationShape;
use hunteval_runner::EpisodePackage;

#[test]
fn cloud_fixtures_cover_malicious_and_benign_cases_without_leakage()
-> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace unavailable"))?
        .join("datasets");
    for provider in ["aws", "azure", "gcp"] {
        for number in 1..=6 {
            let id = format!("{provider}-iam-{number:03}");
            let package = EpisodePackage::load(root.join(provider).join(&id))?;
            assert_eq!(package.public().manifest.id.as_str(), id);
            let public =
                std::fs::read_to_string(package.public().public_root.join("manifest.yaml"))?;
            assert!(!public.contains("malicious_event_ids"));
            assert!(!public.contains("ground_truth"));
            if number == 4 {
                assert!(package.ground_truth().is_benign_scored_episode());
                assert!(!public.contains("benign_evaluation: true"));
                assert!(!package.public().manifest.category.contains("benign"));
            }
            if number >= 4 {
                let classification = package
                    .public()
                    .classification
                    .as_ref()
                    .ok_or("new R4 episode must have classification")?;
                assert_eq!(classification.episode_id.as_str(), id);
                assert!(package.digests().public_classification.is_some());
                assert!(!public.contains("classification"));
            }
            if number == 5 {
                let classification = package
                    .public()
                    .classification
                    .as_ref()
                    .ok_or("multi-stage classification unavailable")?;
                assert!(
                    classification
                        .investigation_shapes
                        .contains(&InvestigationShape::MultiStage)
                );
                assert_eq!(package.ground_truth().expected_attack_path.len(), 5);
            }
            if number == 6 {
                let classification = package
                    .public()
                    .classification
                    .as_ref()
                    .ok_or("cross-boundary classification unavailable")?;
                assert!(
                    classification
                        .investigation_shapes
                        .contains(&InvestigationShape::CrossBoundary)
                );
                let events = std::fs::read_to_string(
                    root.join(provider).join(&id).join("source/events.json"),
                )?;
                assert!(events.contains(&format!("{provider}-tenant-001")));
                assert!(events.contains(&format!("{provider}-tenant-002")));
            }
        }
    }
    Ok(())
}
