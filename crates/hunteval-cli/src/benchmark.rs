use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use crate::args::{BenchmarkCommand, OutputFormatArgument, RetryArgument};
use hunteval_runner::{
    BenchmarkRunOptions, BenchmarkService, ComparisonStatus, ProductionCellExecutor, RetryPolicy,
    load_stored_definition, resolve_execution_plan,
};

mod controller;
mod topology;

use controller::{
    ControllerConfig, canonical_directory, canonical_regular, create_output_directory,
    read_controller, sibling_binary_directory, write_controller,
};

pub(crate) fn execute(command: BenchmarkCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        BenchmarkCommand::Validate {
            manifest,
            artifact_root,
        } => {
            let benchmark = hunteval_runner::resolve_benchmark(&manifest, &artifact_root)?;
            println!(
                "{}",
                serde_json::json!({"benchmark_id": benchmark.id, "run_cells": benchmark.cell_count()?})
            );
            Ok(ExitCode::SUCCESS)
        }
        BenchmarkCommand::Run {
            manifest,
            output,
            jobs,
            fail_fast,
            artifact_root,
            deployment_executable,
            duckdb_worker,
            schema_contract,
        } => run_new(
            &manifest,
            &output,
            jobs,
            fail_fast,
            &artifact_root,
            deployment_executable,
            duckdb_worker,
            schema_contract,
        ),
        BenchmarkCommand::Resume {
            benchmark_directory,
            retry,
        } => resume(&benchmark_directory, retry),
        BenchmarkCommand::Status {
            benchmark_directory,
            format,
        } => status(&benchmark_directory, format),
        BenchmarkCommand::Compare {
            benchmark_directory,
            left,
            right,
        } => compare(&benchmark_directory, &left, &right),
        BenchmarkCommand::TopologyReport {
            experiment,
            baseline_topology,
            candidate_topology,
            statistical_policy,
            scoring_profile,
            observations,
            seed,
            format,
        } => topology::render(
            topology::InputPaths {
                experiment: &experiment,
                baseline_topology: &baseline_topology,
                candidate_topology: &candidate_topology,
                statistical_policy: &statistical_policy,
                scoring_profile: &scoring_profile,
                observations: &observations,
            },
            seed,
            format,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn run_new(
    manifest: &Path,
    output: &Path,
    jobs: usize,
    fail_fast: bool,
    artifact_root: &Path,
    deployment_executable: Option<PathBuf>,
    duckdb_worker: Option<PathBuf>,
    schema_contract: Option<PathBuf>,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if !(1..=256).contains(&jobs) {
        return Err(std::io::Error::other("jobs must be between 1 and 256").into());
    }
    let manifest = canonical_regular(manifest)?;
    let artifact_root = canonical_directory(artifact_root)?;
    let sibling = sibling_binary_directory()?;
    let deployment_executable = canonical_regular(
        &deployment_executable.unwrap_or_else(|| sibling.join("hunteval-reference-deployment")),
    )?;
    let duckdb_worker = canonical_regular(
        &duckdb_worker.unwrap_or_else(|| sibling.join("hunteval-duckdb-worker")),
    )?;
    let schema_contract = canonical_regular(
        &schema_contract
            .unwrap_or_else(|| artifact_root.join("schemas/v0.3/protocol-message.schema.json")),
    )?;
    let output = create_output_directory(output)?;
    let config = ControllerConfig {
        manifest,
        artifact_root,
        deployment_executable,
        duckdb_worker,
        schema_contract,
        jobs,
        fail_fast,
    };
    write_controller(&output, &config)?;
    execute_plan(&output, &config, RetryPolicy::None)
}

fn resume(
    benchmark_directory: &Path,
    retry: RetryArgument,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let root = canonical_directory(benchmark_directory)?;
    let config = read_controller(&root)?;
    let retry = match retry {
        RetryArgument::Failed => RetryPolicy::Failed,
        RetryArgument::Interrupted => RetryPolicy::Interrupted,
        RetryArgument::None => RetryPolicy::None,
    };
    execute_plan(&root, &config, retry)
}

fn execute_plan(
    root: &Path,
    config: &ControllerConfig,
    retry: RetryPolicy,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    validate_controller(config)?;
    let mut plan = resolve_execution_plan(&config.manifest, &config.artifact_root)?;
    let runner_executable = std::env::current_exe()?.canonicalize()?;
    plan.bind_runtime_artifacts(
        &config.deployment_executable,
        &config.duckdb_worker,
        &config.schema_contract,
        &runner_executable,
    )?;
    let executor = ProductionCellExecutor::new(
        plan.clone(),
        config.deployment_executable.clone(),
        config.duckdb_worker.clone(),
        config.schema_contract.clone(),
    );
    let summary = BenchmarkService::new(&executor).run(
        &plan,
        root,
        BenchmarkRunOptions {
            jobs: config.jobs,
            fail_fast: config.fail_fast,
            retry,
        },
    )?;
    println!("{}", serde_json::to_string(&summary)?);
    Ok(if summary.failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn validate_controller(config: &ControllerConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !(1..=256).contains(&config.jobs)
        || canonical_regular(&config.manifest)? != config.manifest
        || canonical_directory(&config.artifact_root)? != config.artifact_root
        || canonical_regular(&config.deployment_executable)? != config.deployment_executable
        || canonical_regular(&config.duckdb_worker)? != config.duckdb_worker
        || canonical_regular(&config.schema_contract)? != config.schema_contract
    {
        return Err(std::io::Error::other("benchmark controller configuration is unsafe").into());
    }
    Ok(())
}

fn status(
    benchmark_directory: &Path,
    format: OutputFormatArgument,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let root = canonical_directory(benchmark_directory)?;
    let definition = load_stored_definition(&root)?;
    let summary = BenchmarkService::status(&root, &definition)?;
    match format {
        OutputFormatArgument::Json => println!("{}", serde_json::to_string(&summary)?),
        OutputFormatArgument::Text => {
            println!("total: {}", summary.total);
            println!("completed: {}", summary.completed);
            println!("failed: {}", summary.failed);
            println!("pending: {}", summary.pending);
            println!("non-comparable: {}", summary.non_comparable);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn compare(
    benchmark_directory: &Path,
    left: &str,
    right: &str,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let root = canonical_directory(benchmark_directory)?;
    let definition = load_stored_definition(&root)?;
    let left = left.parse()?;
    let right = right.parse()?;
    let eligibility = BenchmarkService::compare(&root, &definition, &left, &right)?;
    println!("{}", serde_json::to_string(&eligibility)?);
    Ok(if eligibility.status == ComparisonStatus::Eligible {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    })
}
