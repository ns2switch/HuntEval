//! Deterministic reports derived only from normalized trusted artifacts.

mod html;
mod types;

pub use html::StaticHtmlRenderer;
pub use types::{
    ArtifactLink, BenchmarkReport, DiagnosticFinding, DiagnosticReport, DiagnosticValidationStatus,
    JsonRenderer, ReportClaim, ReportError, ReportFormat, ReportRenderer, RunReport,
};
