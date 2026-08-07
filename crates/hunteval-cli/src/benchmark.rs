use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

use hunteval_runner::{
    BenchmarkRunOptions, BenchmarkService, ComparisonStatus, ProductionCellExecutor, RetryPolicy,
    load_stored_definition, resolve_execution_plan,
};
use serde::{Deserialize, Serialize};

use crate::args::{BenchmarkCommand, OutputFormatArgument, RetryArgument};

const CONTROLLER_FILE: &str = "benchmark-controller.json";
const MAX_CONTROLLER_BYTES: u64 = 64 * 1024;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControllerConfig {
    manifest: PathBuf,
    artifact_root: PathBuf,
    deployment_executable: PathBuf,
    duckdb_worker: PathBuf,
    schema_contract: PathBuf,
    jobs: usize,
    fail_fast: bool,
}

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

fn sibling_binary_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .ok_or_else(|| std::io::Error::other("CLI executable has no parent directory"))?
        .to_path_buf())
}

fn canonical_regular(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(std::io::Error::other("expected a regular non-symlink file").into());
    }
    Ok(path.canonicalize()?)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(std::io::Error::other("expected a non-symlink directory").into());
    }
    Ok(path.canonicalize()?)
}

fn create_output_directory(path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "benchmark output already exists",
        )
        .into());
    }
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("benchmark output has no directory name"))?;
    if !matches!(path.components().next_back(), Some(Component::Normal(_))) {
        return Err(std::io::Error::other("benchmark output path is unsafe").into());
    }
    let parent = canonical_directory(path.parent().unwrap_or_else(|| Path::new(".")))?;
    let output = parent.join(name);
    fs::create_dir(&output)?;
    Ok(output.canonicalize()?)
}

fn write_controller(
    root: &Path,
    config: &ControllerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = serde_json::to_vec_pretty(config)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(root.join(CONTROLLER_FILE))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

fn read_controller(root: &Path) -> Result<ControllerConfig, Box<dyn std::error::Error>> {
    let path = root.join(CONTROLLER_FILE);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_CONTROLLER_BYTES
    {
        return Err(std::io::Error::other("benchmark controller file is unsafe").into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_CONTROLLER_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONTROLLER_BYTES {
        return Err(std::io::Error::other("benchmark controller file is too large").into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
