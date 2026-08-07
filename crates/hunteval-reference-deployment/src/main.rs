use std::process::ExitCode;

use clap::Parser;
use hunteval_reference_deployment::{ReferenceOptions, run_stdio};

fn main() -> ExitCode {
    let options = ReferenceOptions::parse();
    match run_stdio(options.topology) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("reference deployment failed: {error}");
            ExitCode::FAILURE
        }
    }
}
