use std::{io, path::PathBuf};

use hunteval_runner::EpisodePackage;

#[test]
fn cloud_fixtures_cover_three_categories_per_provider_without_leakage()
-> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .ok_or_else(|| io::Error::other("workspace unavailable"))?
        .join("datasets");
    for provider in ["aws", "azure", "gcp"] {
        for number in 1..=3 {
            let id = format!("{provider}-iam-{number:03}");
            let package = EpisodePackage::load(root.join(provider).join(&id))?;
            assert_eq!(package.public().manifest.id.as_str(), id);
            let public =
                std::fs::read_to_string(package.public().public_root.join("manifest.yaml"))?;
            assert!(!public.contains("malicious_event_ids"));
            assert!(!public.contains("ground_truth"));
        }
    }
    Ok(())
}
