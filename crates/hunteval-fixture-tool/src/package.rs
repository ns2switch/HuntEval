use std::{fs, path::Path};

use serde_json::json;

use crate::{
    FixtureGenerationError,
    catalog::{EPISODES, EpisodeSpec},
    generate_fixture,
};

/// Regenerates the complete nine-episode deterministic cloud catalog.
pub fn generate_all(dataset_root: &Path) -> Result<(), FixtureGenerationError> {
    for spec in EPISODES {
        let root = dataset_root.join(spec.provider).join(spec.id);
        if spec.id != "aws-iam-001" {
            write_package(&root, spec)?;
        }
        generate_fixture(
            &root.join("source/events.json"),
            &root.join("public/telemetry").join(spec.telemetry_file),
        )?;
    }
    Ok(())
}

fn write_package(root: &Path, spec: EpisodeSpec) -> Result<(), FixtureGenerationError> {
    fs::create_dir_all(root.join("source"))?;
    fs::create_dir_all(root.join("public/telemetry"))?;
    fs::create_dir_all(root.join("private"))?;
    fs::write(root.join("package.yaml"), package_index(spec))?;
    fs::write(root.join("public/manifest.yaml"), public_manifest(spec))?;
    fs::write(root.join("private/ground-truth.json"), ground_truth(spec)?)?;
    fs::write(root.join("source/events.json"), source_events(spec)?)?;
    Ok(())
}

fn package_index(spec: EpisodeSpec) -> String {
    format!(
        "schema_version: \"0.3\"\nepisode_id: {}\npublic_root: public\nprivate_ground_truth: private/ground-truth.json\n",
        spec.id
    )
}

fn public_manifest(spec: EpisodeSpec) -> String {
    format!(
        r#"schema_version: "0.3"
id: {id}
title: "Synthetic {provider} {category} hunt"
provider: {provider}
category: {category}
objective:
  primary: "Identify suspicious identity activity and distinguish the benign alternative."
  secondary: ["Preserve event and entity provenance."]
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
  max_outstanding_tasks: 8
  max_delegation_depth: 2
  max_tool_calls: 10
  max_sql_queries: 8
  max_retrieved_documents: 0
  max_messages: 40
  max_duration_seconds: 120
  max_tokens: 20000
  max_estimated_cost: null
fault_profile: null
benign_evaluation: false
"#,
        id = spec.id,
        provider = spec.provider,
        category = spec.category,
        table = spec.table,
        file = spec.telemetry_file
    )
}

fn source_events(spec: EpisodeSpec) -> Result<Vec<u8>, serde_json::Error> {
    let actions = [
        "Login",
        "ReadPolicy",
        "ListRoles",
        "AssumeAdmin",
        "GrantPrivilege",
        "CreateCredential",
    ];
    let events: Vec<_> = actions
        .iter()
        .enumerate()
        .map(|(index, action)| {
            json!({
                "event_id": format!("evt-{:04}", index + 1),
                "event_time": format!("2026-01-01T00:0{}:00Z", index),
                "provider": spec.provider,
                "account_id": format!("{}-tenant-001", spec.provider),
                "principal": if index < 3 { "benign-operator" } else { "suspected-identity" },
                "event_name": action,
                "resource": format!("{}:resource:admin", spec.provider),
                "source_ip": if index < 3 { "198.51.100.10" } else { "203.0.113.77" },
                "user_agent": if index < 3 { "provider-cli/1.0" } else { "unknown-client/1.0" }
            })
        })
        .collect();
    let mut bytes = serde_json::to_vec_pretty(&events)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn ground_truth(spec: EpisodeSpec) -> Result<Vec<u8>, serde_json::Error> {
    let truth = json!({
        "schema_version": "0.3", "episode_id": spec.id,
        "malicious_event_ids": ["evt-0004", "evt-0005", "evt-0006"],
        "malicious_entity_ids": ["suspected-identity", format!("{}:resource:admin", spec.provider)],
        "expected_attack_path": ["evt-0004", "evt-0005", "evt-0006"],
        "expected_attack_techniques": ["T1078", "T1098"],
        "acceptable_conclusions": [format!("The synthetic {} identity escalated privileges and created persistence.", spec.provider)],
        "minimum_evidence_items": 1
    });
    let mut bytes = serde_json::to_vec_pretty(&truth)?;
    bytes.push(b'\n');
    Ok(bytes)
}
