//! HuntEval command-line entry point.

mod args;
mod benchmark;

use std::process::ExitCode;

use args::{
    Cli, Command, DeploymentCommand, OutputFormatArgument, ReportCommand, ReportFormatArgument,
    RunArguments, RunCommand, SystemCommand, TrajectoryCommand,
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
        Some(Command::Run(arguments)) => execute_run(arguments),
        Some(Command::System { command }) => execute_system(command),
        Some(Command::Deployment { command }) => execute_deployment(command),
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

fn execute_deployment(command: DeploymentCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        DeploymentCommand::Conformance {
            deployment,
            arguments,
            format,
        } => {
            let result = hunteval_runner::run_conformance(&deployment, &arguments);
            match format {
                OutputFormatArgument::Json => {
                    println!("{}", serde_json::to_string(&result)?);
                }
                OutputFormatArgument::Text => {
                    println!("status: {:?}", result.status);
                    println!("protocol version: {}", result.protocol_version);
                    println!("transcript sha256: {}", result.transcript_sha256);
                    for check in &result.checks {
                        println!("check: {check}");
                    }
                }
            }
            Ok(
                if result.status == hunteval_runner::ConformanceStatus::Conformant {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                },
            )
        }
    }
}

fn execute_run(arguments: RunArguments) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if let Some(RunCommand::Verify { directory, format }) = arguments.command {
        let result = hunteval_runner::verify_run(&directory);
        match format {
            OutputFormatArgument::Json => println!("{}", serde_json::to_string(&result)?),
            OutputFormatArgument::Text => {
                println!("status: {:?}", result.status);
                println!("private evaluation: {}", result.private_evaluation);
                println!("checked artifacts: {}", result.checked_artifacts);
                for check in &result.checks {
                    println!(
                        "{}: {}{}",
                        check.check,
                        if check.passed { "passed" } else { "failed" },
                        check
                            .reason
                            .as_deref()
                            .map(|reason| format!(" ({reason})"))
                            .unwrap_or_default()
                    );
                }
            }
        }
        return Ok(
            if result.status == hunteval_runner::VerificationStatus::Verified {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            },
        );
    }
    let episode = arguments
        .episode
        .ok_or_else(|| std::io::Error::other("run requires --episode and --deployment"))?;
    let deployment = arguments
        .deployment
        .ok_or_else(|| std::io::Error::other("run requires --episode and --deployment"))?;
    let path = hunteval_runner::run_vertical_slice(&episode, &deployment, &arguments.output)?;
    println!("run artifacts: {}", path.display());
    Ok(ExitCode::SUCCESS)
}

fn execute_system(command: SystemCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        SystemCommand::Check { format } => {
            let report = hunteval_runner::probe_linux_sandbox();
            match format {
                OutputFormatArgument::Json => {
                    println!("{}", serde_json::to_string(&report)?);
                }
                OutputFormatArgument::Text => {
                    println!("backend: {}", report.backend);
                    println!("supported: {}", report.supported);
                    for capability in &report.capabilities {
                        println!(
                            "{:?}: {}{}",
                            capability.requirement,
                            capability.available,
                            capability
                                .reason_code
                                .as_deref()
                                .map(|reason| format!(" ({reason})"))
                                .unwrap_or_default()
                        );
                    }
                }
            }
            Ok(if report.supported {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }
        SystemCommand::SecretScan {
            root,
            paths,
            format,
        } => {
            let result = hunteval_runner::scan_paths(
                &root,
                &paths,
                &hunteval_runner::SecretScanPolicy::default(),
            );
            match format {
                OutputFormatArgument::Json => {
                    println!("{}", serde_json::to_string(&result)?);
                }
                OutputFormatArgument::Text => {
                    println!("status: {:?}", result.status);
                    println!("scanned artifacts: {}", result.scanned_artifacts);
                    for finding in &result.findings {
                        println!(
                            "finding: {} {}:{} {}",
                            finding.rule_id, finding.artifact, finding.line, finding.fingerprint
                        );
                    }
                    for reason in &result.incomplete_reasons {
                        println!("incomplete: {reason}");
                    }
                }
            }
            Ok(
                if result.status == hunteval_runner::SecretScanStatus::Clean {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                },
            )
        }
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
