use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(author, version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    System {
        #[command(subcommand)]
        command: SystemCommand,
    },
    Deployment {
        #[command(subcommand)]
        command: DeploymentCommand,
    },
    Run(RunArguments),
    Trajectory {
        #[command(subcommand)]
        command: TrajectoryCommand,
    },
    Benchmark {
        #[command(subcommand)]
        command: BenchmarkCommand,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommand,
    },
    Diagnose {
        #[command(subcommand)]
        command: DiagnoseCommand,
    },
    Improvement {
        #[command(subcommand)]
        command: ImprovementCommand,
    },
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ImprovementCommand {
    Validate {
        #[arg(long)]
        experiment: PathBuf,
        #[arg(long)]
        equivalence: PathBuf,
        #[arg(long)]
        candidate_artifact: PathBuf,
        #[arg(long)]
        benchmark_manifest: PathBuf,
        #[arg(long, default_value = ".")]
        artifact_root: PathBuf,
    },
    Run {
        #[arg(long)]
        experiment: PathBuf,
        #[arg(long)]
        equivalence: PathBuf,
        #[arg(long)]
        candidate_artifact: PathBuf,
        #[arg(long)]
        benchmark_manifest: PathBuf,
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
        #[arg(long)]
        experiment: PathBuf,
        #[arg(long)]
        equivalence: PathBuf,
        #[arg(long)]
        candidate_artifact: PathBuf,
        #[arg(long, value_enum, default_value_t = RetryArgument::None)]
        retry: RetryArgument,
    },
    Status {
        benchmark_directory: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
    Verify {
        bundle: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DiagnoseCommand {
    /// Generate an offline diagnosis from one independently verified run.
    Run {
        run: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Generate per-run diagnosis and recurrence for a stored benchmark matrix.
    Benchmark {
        benchmark: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Verify every content-addressed artifact in a diagnostic bundle.
    Verify {
        bundle: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DatasetCommand {
    /// Render a content-addressed approval after an independent human review.
    ReviewRecord {
        episode: PathBuf,
        #[arg(long)]
        review_policy: PathBuf,
        #[arg(long)]
        review_id: String,
        #[arg(long)]
        reviewer_id: String,
        #[arg(long)]
        reviewed_at: String,
        #[arg(long, required = true)]
        confirm_independent_approval: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum DeploymentCommand {
    Conformance {
        deployment: PathBuf,
        #[arg(last = true)]
        arguments: Vec<String>,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
}

#[derive(Debug, Args)]
pub(crate) struct RunArguments {
    #[command(subcommand)]
    pub(crate) command: Option<RunCommand>,
    #[arg(long, requires = "deployment")]
    pub(crate) episode: Option<PathBuf>,
    #[arg(long, requires = "episode")]
    pub(crate) deployment: Option<PathBuf>,
    #[arg(long, default_value = "runs")]
    pub(crate) output: PathBuf,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RunCommand {
    Verify {
        directory: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum SystemCommand {
    Check {
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
    SecretScan {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
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
    TopologyReport {
        #[arg(long)]
        experiment: PathBuf,
        #[arg(long)]
        baseline_topology: PathBuf,
        #[arg(long)]
        candidate_topology: PathBuf,
        #[arg(long)]
        statistical_policy: PathBuf,
        #[arg(long)]
        scoring_profile: PathBuf,
        #[arg(long)]
        observations: PathBuf,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long, value_enum, default_value_t = ReportFormatArgument::Json)]
        format: ReportFormatArgument,
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
    Verify {
        report: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormatArgument::Text)]
        format: OutputFormatArgument,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ReportFormatArgument {
    Json,
    Html,
}

#[cfg(test)]
mod tests;
