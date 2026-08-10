use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum KnowledgeFormat {
    Json,
    Html,
}

#[derive(Debug, Subcommand)]
pub(crate) enum KnowledgeCommand {
    Validate {
        manifest: PathBuf,
    },
    Build {
        manifest: PathBuf,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    Query {
        manifest: PathBuf,
        query: PathBuf,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        audit: PathBuf,
        #[arg(long, value_enum, default_value = "json")]
        format: KnowledgeFormat,
    },
    Verify {
        manifest: PathBuf,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        audit: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ExtensionCommand {
    Validate {
        manifest: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    Conformance {
        manifest: PathBuf,
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        executable: PathBuf,
        #[arg(long = "arg")]
        arguments: Vec<String>,
    },
}
