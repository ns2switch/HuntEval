use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum TrajectoryCommand {
    Inspect { path: PathBuf },
}

#[derive(Debug, Subcommand)]
pub(crate) enum BenchmarkCommand {
    Validate {
        manifest: PathBuf,
        #[arg(long, default_value = ".")]
        artifact_root: PathBuf,
    },
    Run {
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long, default_value = ".")]
        artifact_root: PathBuf,
        #[arg(long)]
        deployment_executable: Option<PathBuf>,
        #[arg(long)]
        duckdb_worker: Option<PathBuf>,
        #[arg(long)]
        schema_contract: Option<PathBuf>,
    },
    Resume {
        benchmark_directory: PathBuf,
        #[arg(long, value_enum, default_value_t = RetryArgument::None)]
        retry: RetryArgument,
    },
    Status {
        benchmark_directory: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
    Compare {
        benchmark_directory: PathBuf,
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum RetryArgument {
    Failed,
    Interrupted,
    None,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum OutputFormatArgument {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ReportCommand {
    Generate {
        run: PathBuf,
        #[arg(long, value_enum)]
        format: ReportFormatArgument,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ReportFormatArgument {
    Json,
    Html,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{BenchmarkCommand, Cli, Command, OutputFormatArgument, RetryArgument};

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
}
