use std::process::ExitCode;

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
            hunteval_runner::validate_improvement_inputs(
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
            hunteval_runner::validate_improvement_inputs(
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
            hunteval_runner::validate_improvement_inputs(
                &experiment,
                &equivalence,
                &candidate_artifact,
                None,
            )?;
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
