use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterfaceKind {
    Artifact,
    Cli,
    CommercialConnector,
    Extension,
    FrameworkConnector,
    Knowledge,
    Metric,
    Platform,
    Protocol,
    Report,
    Schema,
    ScoringProfile,
    Sdk,
    Topology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityClass {
    StableCandidate,
    Retained,
    Preview,
    Experimental,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    Public,
    EvaluatorPrivate,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreconditionStatus {
    Satisfied,
    Pending,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceEntry {
    pub interface_id: String,
    pub kind: InterfaceKind,
    pub owner: String,
    pub stability: StabilityClass,
    pub version_range: String,
    pub fixture_path: Option<String>,
    pub verification_gate: Option<String>,
    pub projection: Projection,
    pub authority: String,
    pub trust_boundary: String,
    pub bounds_documented: bool,
    pub parser_behavior_documented: bool,
    pub precondition_status: PreconditionStatus,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceInventory {
    pub schema_version: String,
    pub inventory_id: String,
    pub baseline_revision: String,
    pub pre_r8_status: PreconditionStatus,
    pub interfaces: Vec<InterfaceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreezeExclusion {
    pub interface_id: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterfaceFreezeManifest {
    pub schema_version: String,
    pub inventory_sha256: String,
    pub eligible_interfaces: Vec<String>,
    pub exclusions: Vec<FreezeExclusion>,
}
