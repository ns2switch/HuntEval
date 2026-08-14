use std::{collections::BTreeMap, fs, path::Path};

use hunteval_domain::{EpisodeClassification, EpisodeManifest, GroundTruth};
use serde::Serialize;
use serde_json::Value;

use crate::FixtureGenerationError;

#[derive(Debug, Serialize)]
pub struct CorpusInventory {
    schema_version: &'static str,
    visibility: &'static str,
    baseline_only: bool,
    summary: InventorySummary,
    episodes: Vec<EpisodeInventory>,
}

#[derive(Debug, Default, Serialize)]
struct InventorySummary {
    episode_count: usize,
    providers: BTreeMap<String, usize>,
    difficulty: BTreeMap<String, usize>,
    event_volume_tiers: BTreeMap<String, usize>,
    multi_stage_episodes: usize,
    cross_boundary_episodes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    benign_episodes: Option<usize>,
}

#[derive(Debug, Serialize)]
struct EpisodeInventory {
    episode_id: String,
    provider: String,
    category: String,
    difficulty: String,
    required_capabilities: Vec<String>,
    investigation_shapes: Vec<String>,
    telemetry_tables: Vec<String>,
    cloud_services: Vec<String>,
    event_count: usize,
    event_volume_tier: String,
    investigation_duration_minutes: u64,
    boundary_type: &'static str,
    boundary_count: usize,
    classification_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attack_techniques: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    malicious_event_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    malicious_entity_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attack_path_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    benign_alternatives_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeline_complexity: Option<&'static str>,
}

pub fn write_corpus_inventory(
    dataset_root: &Path,
    output: &Path,
    baseline_only: bool,
    internal: bool,
) -> Result<(), FixtureGenerationError> {
    let inventory = build_inventory(dataset_root, baseline_only, internal)?;
    let parent = output
        .parent()
        .ok_or(FixtureGenerationError::OutputHasNoParent)?;
    fs::create_dir_all(parent)?;
    let mut bytes = serde_json::to_vec_pretty(&inventory)?;
    bytes.push(b'\n');
    fs::write(output, bytes)?;
    Ok(())
}

pub fn write_corpus_inventory_markdown(
    dataset_root: &Path,
    output: &Path,
    baseline_only: bool,
    internal: bool,
) -> Result<(), FixtureGenerationError> {
    let inventory = build_inventory(dataset_root, baseline_only, internal)?;
    let parent = output
        .parent()
        .ok_or(FixtureGenerationError::OutputHasNoParent)?;
    fs::create_dir_all(parent)?;
    let mut text = format!(
        "# HuntEval corpus inventory\n\nVisibility: `{}`. Episodes: `{}`.\n\n",
        inventory.visibility, inventory.summary.episode_count
    );
    text.push_str("| Episode | Provider | Category | Difficulty | Outcome | Events | Duration (min) | Boundaries | Stages | Services | ATT&CK |\n");
    text.push_str("|---|---|---|---|---|---:|---:|---:|---:|---|---|\n");
    for episode in inventory.episodes {
        text.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            episode.episode_id,
            episode.provider,
            episode.category,
            episode.difficulty,
            episode.outcome.unwrap_or("not published"),
            episode.event_count,
            episode.investigation_duration_minutes,
            episode.boundary_count,
            episode
                .stage_count
                .map_or_else(|| "-".to_owned(), |value| value.to_string()),
            episode.cloud_services.join(", "),
            episode.attack_techniques.unwrap_or_default().join(", ")
        ));
    }
    fs::write(output, text)?;
    Ok(())
}

fn build_inventory(
    dataset_root: &Path,
    baseline_only: bool,
    internal: bool,
) -> Result<CorpusInventory, FixtureGenerationError> {
    let mut episodes = Vec::new();
    for provider in ["aws", "azure", "gcp"] {
        let mut roots = fs::read_dir(dataset_root.join(provider))?
            .map(|entry| entry.map(|value| value.path()))
            .collect::<Result<Vec<_>, _>>()?;
        roots.sort();
        for root in roots {
            if !root.is_dir() {
                continue;
            }
            let id = root
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or(FixtureGenerationError::MalformedContributorPackage)?;
            if baseline_only && !id.contains("-iam-") {
                continue;
            }
            episodes.push(inspect_episode(&root, internal)?);
        }
    }
    let summary = summarize(&episodes, internal);
    Ok(CorpusInventory {
        schema_version: "1.0",
        visibility: if internal { "reviewer_only" } else { "public" },
        baseline_only,
        summary,
        episodes,
    })
}

fn inspect_episode(
    root: &Path,
    internal: bool,
) -> Result<EpisodeInventory, FixtureGenerationError> {
    let manifest: EpisodeManifest =
        serde_yaml_ng::from_slice(&fs::read(root.join("public/manifest.yaml"))?)?;
    let classification =
        read_optional::<EpisodeClassification>(&root.join("public/classification.json"))?;
    let ground_truth: GroundTruth =
        serde_json::from_slice(&fs::read(root.join("private/ground-truth.json"))?)?;
    let events: Vec<Value> = serde_json::from_slice(&fs::read(root.join("source/events.json"))?)?;
    let provenance = read_optional::<Value>(&root.join("public/provenance.json"))?;
    let boundaries = events
        .iter()
        .filter_map(|event| event["account_id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let shapes = classification
        .as_ref()
        .map(|value| strings(&value.investigation_shapes))
        .unwrap_or_default();
    let capabilities = classification
        .as_ref()
        .map(|value| strings(&value.capabilities))
        .unwrap_or_default();
    let difficulty = classification
        .as_ref()
        .map(|value| format!("{:?}", value.difficulty).to_ascii_lowercase())
        .unwrap_or_else(|| "unavailable".to_owned());
    let duration = duration_minutes(&events);
    let event_volume_tier = provenance
        .as_ref()
        .and_then(|value| value["event_volume_tier"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| volume_tier(events.len()).to_owned());
    let cloud_services = provenance
        .as_ref()
        .and_then(|value| value["services"].as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|| legacy_services(&manifest.provider));
    let benign = ground_truth.is_benign_scored_episode();
    let path_length = ground_truth.expected_attack_path.len();

    Ok(EpisodeInventory {
        episode_id: manifest.id.as_str().to_owned(),
        provider: format!("{:?}", manifest.provider).to_ascii_lowercase(),
        category: manifest.category,
        difficulty,
        required_capabilities: capabilities,
        investigation_shapes: shapes,
        telemetry_tables: manifest
            .telemetry
            .tables
            .into_iter()
            .map(|table| table.name)
            .collect(),
        cloud_services,
        event_count: events.len(),
        event_volume_tier,
        investigation_duration_minutes: duration,
        boundary_type: if boundaries.len() > 1 {
            "cross_scope"
        } else {
            "single_scope"
        },
        boundary_count: boundaries.len(),
        classification_available: classification.is_some(),
        outcome: internal.then_some(if benign { "benign" } else { "malicious" }),
        attack_techniques: internal.then(|| strings(&ground_truth.expected_attack_techniques)),
        malicious_event_count: internal.then_some(ground_truth.malicious_event_ids.len()),
        malicious_entity_count: internal.then_some(ground_truth.malicious_entity_ids.len()),
        attack_path_length: internal.then_some(path_length),
        stage_count: internal.then_some(path_length),
        benign_alternatives_present: internal.then_some(has_benign_alternatives(&events)),
        timeline_complexity: internal.then_some(timeline_complexity(duration, path_length)),
    })
}

fn summarize(episodes: &[EpisodeInventory], internal: bool) -> InventorySummary {
    let mut summary = InventorySummary {
        episode_count: episodes.len(),
        benign_episodes: internal.then_some(0),
        ..InventorySummary::default()
    };
    for episode in episodes {
        *summary
            .providers
            .entry(episode.provider.clone())
            .or_default() += 1;
        *summary
            .difficulty
            .entry(episode.difficulty.clone())
            .or_default() += 1;
        *summary
            .event_volume_tiers
            .entry(episode.event_volume_tier.clone())
            .or_default() += 1;
        if episode
            .investigation_shapes
            .iter()
            .any(|shape| shape == "multi_stage")
        {
            summary.multi_stage_episodes += 1;
        }
        if episode.boundary_type == "cross_scope" {
            summary.cross_boundary_episodes += 1;
        }
        if episode.outcome == Some("benign") {
            summary.benign_episodes = summary.benign_episodes.map(|value| value + 1);
        }
    }
    summary
}

fn read_optional<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, FixtureGenerationError> {
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

fn strings<T: Serialize + Ord>(values: &std::collections::BTreeSet<T>) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            serde_json::to_value(value)
                .ok()?
                .as_str()
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn duration_minutes(events: &[Value]) -> u64 {
    let first = events
        .first()
        .and_then(|event| event["event_time"].as_str());
    let last = events.last().and_then(|event| event["event_time"].as_str());
    match (first.and_then(minutes), last.and_then(minutes)) {
        (Some(first), Some(last)) => last.saturating_sub(first),
        _ => 0,
    }
}

fn minutes(timestamp: &str) -> Option<u64> {
    let time = timestamp.split('T').nth(1)?.strip_suffix('Z')?;
    let mut fields = time.split(':');
    let hour = fields.next()?.parse::<u64>().ok()?;
    let minute = fields.next()?.parse::<u64>().ok()?;
    Some(hour * 60 + minute)
}

fn volume_tier(events: usize) -> &'static str {
    match events {
        0..=16 => "small",
        17..=31 => "medium",
        _ => "large",
    }
}

fn has_benign_alternatives(events: &[Value]) -> bool {
    events.iter().any(|event| {
        event["user_agent"]
            .as_str()
            .is_some_and(|value| value.contains("approved") || value.contains("provider"))
            || event["principal"]
                .as_str()
                .is_some_and(|value| value.contains("benign") || value.contains("routine"))
    })
}

fn timeline_complexity(duration: u64, path_length: usize) -> &'static str {
    if duration >= 300 || path_length >= 6 {
        "long"
    } else if duration >= 90 || path_length >= 4 {
        "medium"
    } else {
        "short"
    }
}

fn legacy_services(provider: &hunteval_domain::Provider) -> Vec<String> {
    match provider {
        hunteval_domain::Provider::Aws => vec!["CloudTrail", "IAM", "STS"],
        hunteval_domain::Provider::Azure => vec!["EntraID", "AzureActivity", "RBAC"],
        hunteval_domain::Provider::Gcp => vec!["CloudAuditLogs", "IAM", "ServiceAccounts"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
}
