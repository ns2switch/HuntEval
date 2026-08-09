mod bottleneck;
mod bottleneck_metrics;
mod classification;
mod contribution;
mod recurrence;
mod resolver;
mod sufficiency;
mod taxonomy;

pub use bottleneck::{BottleneckError, evaluate_bottlenecks};
pub use classification::{ClassificationCandidate, DiagnosticInputV07, classify_verified};
pub use contribution::{
    ContributionError, ControlledContributionInput, reduce_controlled_contribution,
};
pub use recurrence::{ComparableDiagnosticCell, RecurrenceError, reduce_recurrence};
pub use resolver::{
    DiagnosticArtifactSet, DiagnosticResolutionError, ResolvedDiagnosticSource, resolve_sources,
};
pub use sufficiency::{EvidenceSufficiency, evaluate_sufficiency};
pub use taxonomy::{
    ClassifierRule, DiagnosticRegistryError, canonical_taxonomy, classifier_registry_digest,
    validate_registry,
};
