use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hunteval_fixture_tool::generate_fixture;

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { episode_path } => generate_fixture(
            &episode_path.join("source/events.json"),
            &episode_path.join("public/telemetry/cloudtrail.parquet"),
        )?,
    }
    Ok(())
}
