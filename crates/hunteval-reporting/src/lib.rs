//! Deterministic reports derived only from normalized trusted artifacts.

mod analytical;
mod benchmark;
mod diagnostic;
mod html;
mod improvement;
mod topology;
mod types;

pub use analytical::{AnalyticalReport, AnalyticalReportMatch};
pub use benchmark::{
    ArtifactDigestReference, BenchmarkArtifact, BenchmarkCellSummary, BenchmarkClaim,
    BenchmarkClaimSource, BenchmarkConstraintSummary, BenchmarkDeploymentSummary,
    BenchmarkJsonRenderer, BenchmarkMetricSummary, BenchmarkPairwiseComparison,
    BenchmarkRankingGroup, BenchmarkResult, BenchmarkResultError, BenchmarkStaticHtmlRenderer,
};
pub use diagnostic::{
    DiagnosticArtifactKind, DiagnosticArtifactReference, DiagnosticClaim, DiagnosticClaimStage,
    DiagnosticJsonRenderer, DiagnosticReport, DiagnosticReportScope, DiagnosticStaticHtmlRenderer,
    DiagnosticValidationStatus,
};
pub use html::StaticHtmlRenderer;
pub use improvement::*;
pub use topology::{ConstraintFirstStatus, TopologyComparisonReport, TopologyReportError};
pub use types::{
    ArtifactLink, BenchmarkReport, JsonRenderer, LegacyDiagnosticFinding, LegacyDiagnosticReport,
    LegacyDiagnosticValidationStatus, ReportClaim, ReportError, ReportFormat, ReportRenderer,
    RunReport,
};
