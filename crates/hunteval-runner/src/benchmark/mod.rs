mod journal;
mod manifest;
mod resolver;

use hunteval_domain::{BenchmarkDefinitionError, SchemaVersion};
use thiserror::Error;

pub use journal::{
    BenchmarkCellState, BenchmarkCellStatus, BenchmarkEvent, BenchmarkEventKind, BenchmarkJournal,
    BenchmarkJournalError, BenchmarkState,
};
pub use manifest::{
    AuthoredBenchmarkManifest, AuthoredRunCell, BenchmarkManifest, RunCell, load_benchmark,
};
pub use resolver::resolve_benchmark;

/// Failures while loading or resolving an authored benchmark manifest.
#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("benchmark manifest is invalid")]
    InvalidManifest,
    #[error("benchmark schema version {0} is unsupported")]
    UnsupportedSchema(SchemaVersion),
    #[error("benchmark dimensions must not be empty")]
    EmptyDimension,
    #[error("benchmark contains a duplicate {0}")]
    DuplicateDimension(&'static str),
    #[error("benchmark repetitions must equal the listed seed count")]
    RepetitionMismatch,
    #[error("field {0} is not valid for this benchmark schema version")]
    IncompatibleField(&'static str),
    #[error("benchmark contains an unsafe artifact path")]
    UnsafePath,
    #[error("benchmark artifact path resolves outside the configured root")]
    ArtifactOutsideRoot,
    #[error("benchmark artifacts must not contain symbolic links")]
    SymlinkArtifact,
    #[error("benchmark artifact exceeds the bounded file or byte count")]
    ArtifactLimit,
    #[error("benchmark artifact descriptor is invalid")]
    InvalidDescriptor,
    #[error("resolved benchmark definition is invalid: {0}")]
    Domain(#[from] BenchmarkDefinitionError),
    #[error("benchmark could not be read: {0}")]
    Io(#[from] std::io::Error),
}
