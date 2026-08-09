//! Deterministic reports derived only from normalized trusted artifacts.

mod benchmark;
mod html;
mod topology;
mod types;

pub use benchmark::{
    ArtifactDigestReference, BenchmarkArtifact, BenchmarkCellSummary, BenchmarkClaim,
    BenchmarkClaimSource, BenchmarkConstraintSummary, BenchmarkDeploymentSummary,
    BenchmarkJsonRenderer, BenchmarkMetricSummary, BenchmarkPairwiseComparison,
    BenchmarkRankingGroup, BenchmarkResult, BenchmarkResultError, BenchmarkStaticHtmlRenderer,
};
pub use html::StaticHtmlRenderer;
pub use topology::{ConstraintFirstStatus, TopologyComparisonReport, TopologyReportError};
pub use types::{
    ArtifactLink, BenchmarkReport, DiagnosticFinding, DiagnosticReport, DiagnosticValidationStatus,
    JsonRenderer, ReportClaim, ReportError, ReportFormat, ReportRenderer, RunReport,
};
