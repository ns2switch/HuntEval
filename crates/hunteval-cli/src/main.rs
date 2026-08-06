//! HuntEval command-line entry point.

use clap::Parser;

/// Evaluate threat-hunting deployments against reproducible episodes.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
