use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hunteval_fixture_tool::{
    ContributorValidationStatus, ScaffoldRequest, build_review_bundle_manifest, generate_all,
    generate_fixture, render_public_documentation, scaffold_episode, validate_episode,
    write_corpus_inventory, write_corpus_inventory_markdown,
};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate the public telemetry for one episode package.
    Generate { episode_path: PathBuf },
    /// Regenerate the complete cloud episode catalog.
    GenerateAll {
        #[arg(default_value = "datasets")]
        dataset_root: PathBuf,
    },
    /// Create a new, non-overwriting episode authoring skeleton.
    Scaffold {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        episode_id: String,
        target: PathBuf,
    },
    /// Validate one authored episode without modifying it.
    Validate { episode_path: PathBuf },
    /// Render ground-truth-free public documentation.
    Document { episode_path: PathBuf },
    /// Render a content-addressed private review inventory.
    ReviewBundle { episode_path: PathBuf },
    /// Write a deterministic corpus inventory.
    CorpusInventory {
        #[arg(default_value = "datasets")]
        dataset_root: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        baseline_only: bool,
        #[arg(long)]
        internal: bool,
        #[arg(long)]
        markdown: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { episode_path } => generate_fixture(
            &episode_path.join("source/events.json"),
            &episode_path.join("public/telemetry/cloudtrail.parquet"),
        )?,
        Command::GenerateAll { dataset_root } => generate_all(&dataset_root)?,
        Command::Scaffold {
            provider,
            episode_id,
            target,
        } => scaffold_episode(&ScaffoldRequest {
            provider: &provider,
            episode_id: &episode_id,
            target: &target,
        })?,
        Command::Validate { episode_path } => {
            let result = validate_episode(&episode_path)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            if result.status != ContributorValidationStatus::Valid {
                return Err("episode validation did not pass".into());
            }
        }
        Command::Document { episode_path } => {
            let result = validate_episode(&episode_path)?;
            print!(
                "{}",
                String::from_utf8(render_public_documentation(&result)?)?
            );
        }
        Command::ReviewBundle { episode_path } => {
            let result = validate_episode(&episode_path)?;
            print!(
                "{}",
                String::from_utf8(build_review_bundle_manifest(&episode_path, &result)?)?
            );
        }
        Command::CorpusInventory {
            dataset_root,
            output,
            baseline_only,
            internal,
            markdown,
        } => {
            if markdown {
                write_corpus_inventory_markdown(&dataset_root, &output, baseline_only, internal)?;
            } else {
                write_corpus_inventory(&dataset_root, &output, baseline_only, internal)?;
            }
        }
    }
    Ok(())
}
