//! Deterministic reports derived only from normalized trusted artifacts.

mod benchmark;
mod html;
mod types;

pub use benchmark::{
    ArtifactDigestReference, BenchmarkArtifact, BenchmarkCellSummary, BenchmarkClaim,
    BenchmarkClaimSource, BenchmarkConstraintSummary, BenchmarkDeploymentSummary,
    BenchmarkJsonRenderer, BenchmarkMetricSummary, BenchmarkPairwiseComparison,
    BenchmarkRankingGroup, BenchmarkResult, BenchmarkResultError, BenchmarkStaticHtmlRenderer,
};
pub use html::StaticHtmlRenderer;
pub use types::{
    ArtifactLink, BenchmarkReport, DiagnosticFinding, DiagnosticReport, DiagnosticValidationStatus,
    JsonRenderer, ReportClaim, ReportError, ReportFormat, ReportRenderer, RunReport,
};
