use hunteval_domain::{MetricDirection, ResourceProvenance, SchemaVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricContract {
    pub name: &'static str,
    pub version: SchemaVersion,
    pub direction: MetricDirection,
    pub required_resource_provenance: Option<ResourceProvenance>,
}

const V03: SchemaVersion = SchemaVersion::new(0, 3);
const V04: SchemaVersion = SchemaVersion::new(0, 4);
const HIGHER: MetricDirection = MetricDirection::HigherIsBetter;
const LOWER: MetricDirection = MetricDirection::LowerIsBetter;

const METRICS: &[MetricContract] = &[
    metric("event_precision", V03, HIGHER, None),
    metric("event_recall", V03, HIGHER, None),
    metric("entity_precision", V03, HIGHER, None),
    metric("entity_recall", V03, HIGHER, None),
    metric("evidence_grounding", V03, HIGHER, None),
    metric("provenance_validity", V03, HIGHER, None),
    metric("task_completion", V03, HIGHER, None),
    metric(
        "tool_call_utilization",
        V03,
        LOWER,
        Some(ResourceProvenance::Measured),
    ),
    metric("resilience", V03, HIGHER, None),
    metric("graceful_degradation", V03, HIGHER, None),
    metric("attack_path_precision", V04, HIGHER, None),
    metric("attack_path_recall", V04, HIGHER, None),
    metric("timeline_precision", V04, HIGHER, None),
    metric("timeline_recall", V04, HIGHER, None),
    metric("conclusion_correctness", V04, HIGHER, None),
    metric("technique_precision", V04, HIGHER, None),
    metric("technique_recall", V04, HIGHER, None),
    metric("evidence_event_coverage", V04, HIGHER, None),
    metric("evidence_entity_coverage", V04, HIGHER, None),
    metric("evidence_sufficiency", V04, HIGHER, None),
    metric("duplicate_tool_work", V04, LOWER, None),
    metric("useful_communication", V04, HIGHER, None),
    metric(
        "measured_duration_utilization",
        V04,
        LOWER,
        Some(ResourceProvenance::Measured),
    ),
    metric(
        "verified_cost_utilization",
        V04,
        LOWER,
        Some(ResourceProvenance::VerifiedAdapter),
    ),
    metric("submission_stability", V04, HIGHER, None),
    metric("metric_stability", V04, HIGHER, None),
];

#[must_use]
pub fn metric_contract(name: &str, version: SchemaVersion) -> Option<MetricContract> {
    METRICS
        .iter()
        .copied()
        .find(|metric| metric.name == name && metric.version == version)
}

#[must_use]
pub const fn metric_contracts() -> &'static [MetricContract] {
    METRICS
}

const fn metric(
    name: &'static str,
    version: SchemaVersion,
    direction: MetricDirection,
    required_resource_provenance: Option<ResourceProvenance>,
) -> MetricContract {
    MetricContract {
        name,
        version,
        direction,
        required_resource_provenance,
    }
}
