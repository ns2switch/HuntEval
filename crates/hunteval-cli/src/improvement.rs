use std::{path::Path, process::ExitCode};

use hunteval_domain::{
    ImprovementEquivalenceResult, ImprovementEquivalenceStatus, ImprovementExperiment, Sha256Digest,
};
use hunteval_runner::ImprovementVerificationStatus;

use crate::{
    args::{ImprovementCommand, OutputFormatArgument},
    benchmark,
};

pub(crate) fn execute(command: ImprovementCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        ImprovementCommand::Validate {
            experiment,
            equivalence,
            candidate_artifact,
            benchmark_manifest,
            artifact_root,
        } => {
            validate_inputs(
                &experiment,
                &equivalence,
                &candidate_artifact,
                Some((&benchmark_manifest, &artifact_root)),
            )?;
            println!("improvement experiment is eligible");
            Ok(ExitCode::SUCCESS)
        }
        ImprovementCommand::Run {
            experiment,
            equivalence,
            candidate_artifact,
            benchmark_manifest,
            output,
            jobs,
            fail_fast,
            artifact_root,
            deployment_executable,
            duckdb_worker,
            schema_contract,
        } => {
            validate_inputs(
                &experiment,
                &equivalence,
                &candidate_artifact,
                Some((&benchmark_manifest, &artifact_root)),
            )?;
            benchmark::run_new(
                &benchmark_manifest,
                &output,
                jobs,
                fail_fast,
                &artifact_root,
                deployment_executable,
                duckdb_worker,
                schema_contract,
            )
        }
        ImprovementCommand::Resume {
            benchmark_directory,
            experiment,
            equivalence,
            candidate_artifact,
            retry,
        } => {
            validate_inputs(&experiment, &equivalence, &candidate_artifact, None)?;
            benchmark::resume(&benchmark_directory, retry)
        }
        ImprovementCommand::Status {
            benchmark_directory,
            format,
        } => benchmark::status(&benchmark_directory, format),
        ImprovementCommand::Verify { bundle, format } => {
            let result = hunteval_runner::verify_improvement_bundle(&bundle);
            match format {
                OutputFormatArgument::Json => {
                    println!("{}", serde_json::to_string(&result)?);
                }
                OutputFormatArgument::Text => {
                    println!("status: {:?}", result.status);
                    println!("checked artifacts: {}", result.checked_artifacts);
                    for reason in &result.reason_codes {
                        println!("reason: {reason}");
                    }
                }
            }
            Ok(
                if result.status == ImprovementVerificationStatus::Verified {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                },
            )
        }
    }
}

fn validate_inputs(
    experiment_path: &Path,
    equivalence_path: &Path,
    candidate_path: &Path,
    benchmark: Option<(&Path, &Path)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let experiment_bytes = safe_read(experiment_path, 1024 * 1024)?;
    let equivalence_bytes = safe_read(equivalence_path, 1024 * 1024)?;
    let candidate_bytes = safe_read(candidate_path, 1024 * 1024)?;
    let experiment: ImprovementExperiment = serde_json::from_slice(&experiment_bytes)?;
    experiment.validate()?;
    let equivalence: ImprovementEquivalenceResult = serde_json::from_slice(&equivalence_bytes)?;
    if equivalence.status != ImprovementEquivalenceStatus::Eligible
        || equivalence.experiment_sha256 != Sha256Digest::from_bytes(&experiment_bytes)
        || experiment.candidate_artifact_sha256 != Sha256Digest::from_bytes(&candidate_bytes)
    {
        return Err(std::io::Error::other("improvement inputs are ineligible or stale").into());
    }
    if let Some((manifest, artifact_root)) = benchmark {
        let definition = hunteval_runner::resolve_benchmark(manifest, artifact_root)?;
        let cell_ids = definition
            .cells()?
            .into_iter()
            .map(|cell| cell.cell_id.to_string())
            .collect::<std::collections::BTreeSet<_>>();
        if experiment.paired_cells.iter().any(|pair| {
            !cell_ids.contains(&pair.baseline_cell_id)
                || !cell_ids.contains(&pair.candidate_cell_id)
        }) {
            return Err(std::io::Error::other("improvement pairs do not resolve").into());
        }
    }
    Ok(())
}

fn safe_read(path: &Path, maximum: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err(std::io::Error::other("input is not a bounded regular file").into());
    }
    Ok(std::fs::read(path)?)
}
