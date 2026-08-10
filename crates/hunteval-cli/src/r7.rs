use std::{path::Path, process::ExitCode};

use crate::r7_args::{ExtensionCommand, KnowledgeCommand, KnowledgeFormat};

const MAX_CONTRACT_BYTES: u64 = 16 * 1024 * 1024;

pub(crate) fn execute_knowledge(
    command: KnowledgeCommand,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        KnowledgeCommand::Validate { manifest } => {
            let manifest = hunteval_runner::validate_analytical_manifest(&read_bytes(&manifest)?)?;
            println!("analytical corpus is valid: {}", manifest.id);
            Ok(ExitCode::SUCCESS)
        }
        KnowledgeCommand::Build { manifest, root } => {
            let index = hunteval_runner::build_analytical_index(&root, &read_bytes(&manifest)?)?;
            println!("{}", serde_json::to_string_pretty(&index)?);
            Ok(ExitCode::SUCCESS)
        }
        KnowledgeCommand::Verify {
            manifest,
            root,
            audit,
        } => {
            let index = hunteval_runner::build_analytical_index(&root, &read_bytes(&manifest)?)?;
            let audit_events = audit
                .as_deref()
                .map(hunteval_runner::verify_retrieval_audit)
                .transpose()?
                .unwrap_or(0);
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status":"verified",
                    "index":index,
                    "audit_events":audit_events
                }))?
            );
            Ok(ExitCode::SUCCESS)
        }
        KnowledgeCommand::Query {
            manifest,
            query,
            root,
            audit,
            format,
        } => {
            let result = hunteval_runner::query_analytical_index_audited(
                &root,
                &read_bytes(&manifest)?,
                &read_bytes(&query)?,
                &audit,
            )?;
            let format = match format {
                KnowledgeFormat::Json => hunteval_runner::ReportFormat::Json,
                KnowledgeFormat::Html => hunteval_runner::ReportFormat::Html,
            };
            print!(
                "{}",
                String::from_utf8(hunteval_runner::render_analytical_result(&result, format)?)?
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

pub(crate) fn execute_extension(
    command: ExtensionCommand,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        ExtensionCommand::Validate { manifest, policy } => {
            let bytes = read_bytes(&manifest)?;
            if let Some(policy) = policy {
                let resolution = hunteval_runner::resolve_extension(&bytes, &read_bytes(&policy)?)?;
                println!("{}", serde_json::to_string_pretty(&resolution)?);
                return Ok(if resolution.reasons.is_empty() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                });
            }
            hunteval_runner::validate_extension_manifest(&bytes)?;
            println!("extension manifest is valid");
            Ok(ExitCode::SUCCESS)
        }
        ExtensionCommand::Conformance {
            manifest,
            policy,
            executable,
            arguments,
        } => {
            let manifest = read_bytes(&manifest)?;
            let policy_bytes = read_bytes(&policy)?;
            let _ = hunteval_runner::resolve_extension(&manifest, &policy_bytes)?;
            let policy = serde_json::from_slice(&policy_bytes)?;
            let result =
                hunteval_runner::conform_extension(&manifest, &policy, &executable, &arguments);
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(if result.reasons.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
    }
}

fn read_bytes(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTRACT_BYTES
    {
        return Err(std::io::Error::other("contract is not a bounded regular file").into());
    }
    Ok(std::fs::read(path)?)
}
