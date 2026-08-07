use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hunteval_fixture_tool::{generate_all, generate_fixture};

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
    /// Regenerate all nine cloud episode packages.
    GenerateAll {
        #[arg(default_value = "datasets")]
        dataset_root: PathBuf,
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
    }
    Ok(())
}
