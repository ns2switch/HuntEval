use clap::Parser;

use super::{
    BenchmarkCommand, Cli, Command, DatasetCommand, DeploymentCommand, OutputFormatArgument,
    ReportCommand, ReportFormatArgument, RetryArgument, RunCommand, SystemCommand,
};

#[test]
fn parses_resume_retry_policy() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from([
        "hunteval",
        "benchmark",
        "resume",
        "runs/cloud-mvp",
        "--retry",
        "interrupted",
    ])?;
    assert!(matches!(
        cli.command,
        Some(Command::Benchmark {
            command: BenchmarkCommand::Resume {
                retry: RetryArgument::Interrupted,
                ..
            }
        })
    ));
    Ok(())
}

#[test]
fn review_record_requires_explicit_approval_confirmation() -> Result<(), clap::Error> {
    let base = [
        "hunteval",
        "dataset",
        "review-record",
        "episode",
        "--review-policy",
        "policy.json",
        "--review-id",
        "review-001",
        "--reviewer-id",
        "reviewer-001",
        "--reviewed-at",
        "2026-08-09T00:00:00Z",
    ];
    assert!(Cli::try_parse_from(base).is_err());
    let cli = Cli::try_parse_from(base.into_iter().chain(["--confirm-independent-approval"]))?;
    assert!(matches!(
        cli.command,
        Some(Command::Dataset {
            command: DatasetCommand::ReviewRecord {
                confirm_independent_approval: true,
                ..
            }
        })
    ));
    Ok(())
}

#[test]
fn parses_controlled_topology_report() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from([
        "hunteval",
        "benchmark",
        "topology-report",
        "--experiment",
        "experiment.json",
        "--baseline-topology",
        "baseline.json",
        "--candidate-topology",
        "candidate.json",
        "--statistical-policy",
        "statistics.json",
        "--scoring-profile",
        "scoring.yaml",
        "--observations",
        "observations.json",
        "--format",
        "html",
    ])?;
    assert!(matches!(
        cli.command,
        Some(Command::Benchmark {
            command: BenchmarkCommand::TopologyReport {
                format: ReportFormatArgument::Html,
                ..
            }
        })
    ));
    Ok(())
}

#[test]
fn parses_json_status_output() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from([
        "hunteval",
        "benchmark",
        "status",
        "runs/cloud-mvp",
        "--format",
        "json",
    ])?;
    assert!(matches!(
        cli.command,
        Some(Command::Benchmark {
            command: BenchmarkCommand::Status {
                format: OutputFormatArgument::Json,
                ..
            }
        })
    ));
    Ok(())
}

#[test]
fn requires_output_for_new_benchmark() {
    assert!(Cli::try_parse_from(["hunteval", "benchmark", "run", "benchmark.yaml"]).is_err());
}

#[test]
fn parses_json_system_check() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from(["hunteval", "system", "check", "--format", "json"])?;
    assert!(matches!(
        cli.command,
        Some(Command::System {
            command: SystemCommand::Check {
                format: OutputFormatArgument::Json
            }
        })
    ));
    Ok(())
}

#[test]
fn preserves_legacy_run_and_parses_run_verification() -> Result<(), clap::Error> {
    let legacy = Cli::try_parse_from([
        "hunteval",
        "run",
        "--episode",
        "episode",
        "--deployment",
        "deployment",
    ])?;
    assert!(matches!(legacy.command, Some(Command::Run(arguments)) if arguments.command.is_none()));

    let verify = Cli::try_parse_from([
        "hunteval",
        "run",
        "verify",
        "run-directory",
        "--format",
        "json",
    ])?;
    assert!(matches!(
        verify.command,
        Some(Command::Run(arguments))
            if matches!(arguments.command, Some(RunCommand::Verify { format: OutputFormatArgument::Json, .. }))
    ));
    Ok(())
}

#[test]
fn parses_deployment_conformance_arguments() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from([
        "hunteval",
        "deployment",
        "conformance",
        "deployment-bin",
        "--format",
        "json",
        "--",
        "--topology",
        "single-agent",
    ])?;
    assert!(matches!(
        cli.command,
        Some(Command::Deployment {
            command: DeploymentCommand::Conformance {
                format: OutputFormatArgument::Json,
                arguments,
                ..
            }
        }) if arguments == ["--topology", "single-agent"]
    ));
    Ok(())
}

#[test]
fn parses_json_report_verification() -> Result<(), clap::Error> {
    let cli = Cli::try_parse_from([
        "hunteval",
        "report",
        "verify",
        "runs/cloud-mvp",
        "--format",
        "json",
    ])?;
    assert!(matches!(
        cli.command,
        Some(Command::Report {
            command: ReportCommand::Verify {
                format: OutputFormatArgument::Json,
                ..
            }
        })
    ));
    Ok(())
}
