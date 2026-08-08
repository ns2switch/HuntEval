//! HuntEval command-line entry point.

mod args;
mod benchmark;

use std::process::ExitCode;

use args::{
    Cli, Command, OutputFormatArgument, ReportCommand, ReportFormatArgument, TrajectoryCommand,
};
use clap::Parser;

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: Cli) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match cli.command {
        None => Ok(ExitCode::SUCCESS),
        Some(Command::Run {
            episode,
            deployment,
            output,
        }) => {
            let path = hunteval_runner::run_vertical_slice(&episode, &deployment, &output)?;
            println!("run artifacts: {}", path.display());
            Ok(ExitCode::SUCCESS)
        }
        Some(Command::Trajectory {
            command: TrajectoryCommand::Inspect { path },
        }) => {
            let bytes = std::fs::read(path)?;
            let (event_count, digest) = hunteval_runner::inspect_trajectory(&bytes)?;
            println!("events: {event_count}");
            println!("sha256: {digest}");
            Ok(ExitCode::SUCCESS)
        }
        Some(Command::Benchmark { command }) => benchmark::execute(command),
        Some(Command::Report { command }) => execute_report(command),
    }
}

fn execute_report(command: ReportCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        ReportCommand::Generate { run, format } => {
            let format = match format {
                ReportFormatArgument::Json => hunteval_runner::ReportFormat::Json,
                ReportFormatArgument::Html => hunteval_runner::ReportFormat::Html,
            };
            hunteval_runner::generate_report(&run, format)?;
            println!("report generated: {}", run.display());
            Ok(ExitCode::SUCCESS)
        }
        ReportCommand::Verify { report, format } => {
            let verification = hunteval_runner::verify_report(&report)?;
            match format {
                OutputFormatArgument::Json => {
                    println!("{}", serde_json::to_string_pretty(&verification)?);
                }
                OutputFormatArgument::Text => {
                    println!("report kind: {}", verification.report_kind);
                    println!("checked artifacts: {}", verification.checked_artifacts);
                    println!("valid: {}", verification.valid);
                    for error in &verification.errors {
                        println!("error: {error}");
                    }
                }
            }
            Ok(if verification.valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
    }
}
