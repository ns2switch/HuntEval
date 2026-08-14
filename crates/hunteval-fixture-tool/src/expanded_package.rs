use std::{collections::BTreeSet, fs, path::Path};

use hunteval_domain::Sha256Digest;
use serde_json::{Value, json};

use crate::{
    FixtureGenerationError, build_review_bundle_manifest,
    expanded_catalog::{EXPANDED_TEMPLATES, PROVIDERS, ScenarioTemplate, table, telemetry_file},
    expanded_provider::{noise_actions, services, stage_actions},
    generate_fixture, validate_episode,
};

pub fn generate_expanded(dataset_root: &Path) -> Result<(), FixtureGenerationError> {
    for provider in PROVIDERS {
        for template in EXPANDED_TEMPLATES {
            generate_episode(dataset_root, provider, template)?;
        }
    }
    Ok(())
}

fn generate_episode(
    dataset_root: &Path,
    provider: &str,
    template: ScenarioTemplate,
) -> Result<(), FixtureGenerationError> {
    let id = format!("{provider}-cloud-{:03}", template.number);
    let root = dataset_root.join(provider).join(&id);
    fs::create_dir_all(root.join("source"))?;
    fs::create_dir_all(root.join("public/telemetry"))?;
    fs::create_dir_all(root.join("private"))?;

    let events = source_events(provider, template);
    let source = json_bytes(&events)?;
    let manifest = public_manifest(provider, &id, template);
    let classification = classification(&id, template)?;
    let truth = ground_truth(provider, &id, template, &events)?;
    let query = reference_query(provider, template);

    fs::write(root.join("package.yaml"), package_index(&id))?;
    fs::write(root.join("public/manifest.yaml"), &manifest)?;
    fs::write(root.join("public/classification.json"), &classification)?;
    fs::write(root.join("source/events.json"), &source)?;
    fs::write(root.join("private/ground-truth.json"), &truth)?;
    fs::write(root.join("private/reference-query.sql"), &query)?;

    let telemetry = root.join("public/telemetry").join(telemetry_file(provider));
    generate_fixture(&root.join("source/events.json"), &telemetry)?;

    let public_provenance = public_provenance(
        provider,
        &id,
        template,
        &source,
        &manifest,
        &classification,
        &fs::read(&telemetry)?,
    )?;
    fs::write(root.join("public/provenance.json"), public_provenance)?;
    fs::write(
        root.join("private/provenance.json"),
        private_provenance(&id, &truth, query.as_bytes())?,
    )?;

    let validation = validate_episode(&root)?;
    let bundle = build_review_bundle_manifest(&root, &validation)?;
    fs::write(root.join("private/review-bundle.json"), bundle)?;
    Ok(())
}

fn package_index(id: &str) -> String {
    format!(
        "schema_version: \"0.4\"\nepisode_id: {id}\npublic_root: public\nprivate_ground_truth: private/ground-truth.json\n"
    )
}

fn public_manifest(provider: &str, id: &str, template: ScenarioTemplate) -> Vec<u8> {
    format!(
        r#"schema_version: "0.4"
id: {id}
title: "Synthetic {provider} cloud activity investigation {number:03}"
provider: {provider}
category: {category}
objective:
  primary: "Investigate unusual cloud control-plane activity and determine the evidence-supported outcome."
  secondary:
    - "Correlate identities, resources, services, and administrative boundaries."
    - "Preserve event and entity provenance and document plausible benign alternatives."
telemetry:
  tables:
    - name: {table}
      path: telemetry/{file}
knowledge:
  documents: []
limits:
  max_agents: 3
  max_parallel_agents: 2
  max_parallel_tool_calls: 1
  max_outstanding_tasks: 12
  max_delegation_depth: 2
  max_tool_calls: 14
  max_sql_queries: 12
  max_retrieved_documents: 0
  max_messages: 60
  max_duration_seconds: 180
  max_tokens: 30000
  max_estimated_cost: null
fault_profile: null
benign_evaluation: false
"#,
        number = template.number,
        category = template.category,
        table = table(provider),
        file = telemetry_file(provider),
    )
    .into_bytes()
}

fn classification(id: &str, template: ScenarioTemplate) -> Result<Vec<u8>, serde_json::Error> {
    let mut capabilities = vec!["evidence_correlation"];
    if template.number <= 15 || template.cross_boundary {
        capabilities.push("identity_analysis");
    }
    if template.multi_stage {
        capabilities.extend(["timeline_reconstruction", "attack_path_analysis"]);
    }
    if !template.malicious {
        capabilities.push("benign_disambiguation");
    }
    if template.cross_boundary {
        capabilities.push("cross_boundary_correlation");
    }
    capabilities.sort_unstable();
    capabilities.dedup();

    let mut shapes = vec![if template.multi_stage {
        "multi_stage"
    } else {
        "single_stage"
    }];
    shapes.push("ambiguous_alternative");
    if template.cross_boundary {
        shapes.push("cross_boundary");
    }
    shapes.sort_unstable();

    json_bytes(&json!({
        "schema_version": "0.6",
        "episode_id": id,
        "difficulty": template.difficulty.as_str(),
        "capabilities": capabilities,
        "investigation_shapes": shapes,
    }))
}

fn source_events(provider: &str, template: ScenarioTemplate) -> Value {
    let count = template.volume.event_count();
    let malicious_positions = malicious_positions(count, template.path_length);
    let stages = stage_actions(provider, template.family);
    let noise = noise_actions(provider);
    let day = match provider {
        "aws" => 10,
        "azure" => 11,
        _ => 12,
    };
    let suspicious_ip = format!("203.0.113.{}", 70 + template.number);

    Value::Array(
        (0..count)
            .map(|index| {
                let event_number = index + 1;
                let malicious_stage = malicious_positions.iter().position(|item| *item == index);
                let confounder = malicious_stage.is_none() && index % 6 == 4;
                let stage_index = malicious_stage.unwrap_or(index / 6) % stages.len();
                let action = if malicious_stage.is_some() || confounder {
                    stages[stage_index]
                } else {
                    noise[index % noise.len()]
                };
                let target_boundary = template.cross_boundary
                    && (malicious_stage.is_some_and(|stage| stage >= template.path_length / 2)
                        || (!template.malicious && index >= count / 2));
                let boundary = if target_boundary { "002" } else { "001" };
                let minute = index * duration_step_minutes(template);
                let hour = minute / 60;
                let minute_of_hour = minute % 60;
                let malicious = malicious_stage.is_some();
                json!({
                    "event_id": format!("evt-{event_number:04}"),
                    "event_time": format!("2026-02-{day:02}T{hour:02}:{minute_of_hour:02}:00Z"),
                    "provider": provider,
                    "account_id": format!("{provider}-scope-{boundary}"),
                    "principal": if malicious { format!("{provider}-principal-investigation") } else if confounder { format!("{provider}-authorized-automation") } else { format!("{provider}-routine-operator") },
                    "event_name": action,
                    "resource": format!("{provider}:{}:resource-{boundary}", service_for_stage(provider, template, stage_index)),
                    "source_ip": if malicious { suspicious_ip.clone() } else if confounder { "198.51.100.44".to_owned() } else { format!("198.51.100.{}", 10 + index % 20) },
                    "user_agent": if malicious { "unrecognized-client/2.0" } else if confounder { "approved-automation/3.2" } else { "provider-console/1.0" }
                })
            })
            .collect(),
    )
}

fn malicious_positions(event_count: usize, path_length: usize) -> Vec<usize> {
    if path_length == 0 {
        return Vec::new();
    }
    let start = event_count / 3;
    (0..path_length).map(|stage| start + stage * 2).collect()
}

fn duration_step_minutes(template: ScenarioTemplate) -> usize {
    match template.volume {
        crate::expanded_catalog::VolumeTier::Small => 2,
        crate::expanded_catalog::VolumeTier::Medium => 5,
        crate::expanded_catalog::VolumeTier::Large => 10,
    }
}

fn service_for_stage(provider: &str, template: ScenarioTemplate, stage: usize) -> &'static str {
    let represented = services(provider, template.family);
    represented[stage % represented.len()]
}

fn reference_query(provider: &str, template: ScenarioTemplate) -> String {
    let suspicious_ip = format!("203.0.113.{}", 70 + template.number);
    format!(
        "SELECT event_id, principal, event_time, event_name FROM {} WHERE source_ip = '{}' AND user_agent = 'unrecognized-client/2.0' ORDER BY event_time, event_id;\n",
        table(provider),
        suspicious_ip
    )
}

fn ground_truth(
    provider: &str,
    id: &str,
    template: ScenarioTemplate,
    events: &Value,
) -> Result<Vec<u8>, serde_json::Error> {
    if !template.malicious {
        return json_bytes(&json!({
            "schema_version": "0.4",
            "episode_id": id,
            "malicious_event_ids": [],
            "malicious_entity_ids": [],
            "expected_attack_path": [],
            "expected_attack_techniques": [],
            "acceptable_conclusions": ["The unusual activity is consistent with authorized administration or automation."],
            "acceptable_submission_statuses": ["no_malicious_activity"],
            "expected_timeline_windows": [],
            "minimum_evidence_items": 1
        }));
    }

    let event_array = events.as_array().cloned().unwrap_or_default();
    let malicious: Vec<_> = event_array
        .iter()
        .filter(|event| event["user_agent"] == "unrecognized-client/2.0")
        .collect();
    let event_ids: Vec<_> = malicious
        .iter()
        .map(|event| event["event_id"].clone())
        .collect();
    let mut entities = BTreeSet::from([format!("{provider}-principal-investigation")]);
    for event in &malicious {
        if let Some(resource) = event["resource"].as_str() {
            entities.insert(resource.to_owned());
        }
    }
    let windows: Vec<_> = malicious
        .iter()
        .map(|event| {
            json!({
                "event_id": event["event_id"],
                "earliest": event["event_time"],
                "latest": event["event_time"],
            })
        })
        .collect();
    json_bytes(&json!({
        "schema_version": "0.4",
        "episode_id": id,
        "malicious_event_ids": event_ids,
        "malicious_entity_ids": entities,
        "expected_attack_path": event_ids,
        "expected_attack_techniques": template.techniques,
        "acceptable_conclusions": ["The correlated control-plane sequence supports a malicious cloud activity finding."],
        "acceptable_submission_statuses": ["confirmed_malicious_activity", "suspicious_activity"],
        "expected_timeline_windows": windows,
        "minimum_evidence_items": template.path_length
    }))
}

fn public_provenance(
    provider: &str,
    id: &str,
    template: ScenarioTemplate,
    source: &[u8],
    manifest: &[u8],
    classification: &[u8],
    telemetry: &[u8],
) -> Result<Vec<u8>, serde_json::Error> {
    let template_identity = json_bytes(&json!({
        "number": template.number,
        "category": template.category,
        "difficulty": template.difficulty.as_str(),
        "volume_tier": template.volume.as_str(),
        "path_length": template.path_length,
        "multi_stage": template.multi_stage,
        "cross_boundary": template.cross_boundary,
        "services": services(provider, template.family),
    }))?;
    json_bytes(&json!({
        "schema_version": "1.0",
        "episode_id": id,
        "provider": provider,
        "generator": "hunteval-fixture-tool",
        "generator_version": env!("CARGO_PKG_VERSION"),
        "generation_seed": 7000 + u64::from(template.number),
        "toolchain": "rust-1.93.1",
        "source_template_sha256": Sha256Digest::from_bytes(template_identity),
        "content_hashes": {
            "source_events": Sha256Digest::from_bytes(source),
            "public_manifest": Sha256Digest::from_bytes(manifest),
            "public_classification": Sha256Digest::from_bytes(classification),
            "public_telemetry": Sha256Digest::from_bytes(telemetry),
        },
        "services": services(provider, template.family),
        "event_volume_tier": template.volume.as_str(),
    }))
}

fn private_provenance(id: &str, truth: &[u8], query: &[u8]) -> Result<Vec<u8>, serde_json::Error> {
    json_bytes(&json!({
        "schema_version": "1.0",
        "episode_id": id,
        "private_ground_truth_sha256": Sha256Digest::from_bytes(truth),
        "private_reference_query_sha256": Sha256Digest::from_bytes(query),
        "review_status": "pending_independent_review"
    }))
}

fn json_bytes(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}
