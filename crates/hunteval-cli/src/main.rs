//! HuntEval command-line entry point.

use std::{path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        #[arg(long)]
        episode: PathBuf,
        #[arg(long)]
        deployment: PathBuf,
        #[arg(long, default_value = "runs")]
        output: PathBuf,
    },
    Trajectory {
        #[command(subcommand)]
        command: TrajectoryCommand,
    },
    Benchmark {
        #[command(subcommand)]
        command: BenchmarkCommand,
    },
}

#[derive(Debug, Subcommand)]
enum TrajectoryCommand {
    Inspect { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum BenchmarkCommand {
    Validate { manifest: PathBuf },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        None => Ok(()),
        Some(Command::Run {
            episode,
            deployment,
            output,
        }) => {
            let path = hunteval_runner::run_vertical_slice(&episode, &deployment, &output)?;
            println!("run artifacts: {}", path.display());
            Ok(())
        }
        Some(Command::Trajectory {
            command: TrajectoryCommand::Inspect { path },
        }) => {
            let bytes = std::fs::read(path)?;
            let (event_count, digest) = hunteval_runner::inspect_trajectory(&bytes)?;
            println!("events: {event_count}");
            println!("sha256: {digest}");
            Ok(())
        }
        Some(Command::Benchmark {
            command: BenchmarkCommand::Validate { manifest },
        }) => {
            let benchmark = hunteval_runner::load_benchmark(&manifest)?;
            println!("run cells: {}", benchmark.cells().len());
            Ok(())
        }
    }
}
