#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Difficulty {
    Introductory,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Introductory => "introductory",
            Self::Intermediate => "intermediate",
            Self::Advanced => "advanced",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VolumeTier {
    Small,
    Medium,
    Large,
}

impl VolumeTier {
    pub(crate) const fn event_count(self) -> usize {
        match self {
            Self::Small => 16,
            Self::Medium => 28,
            Self::Large => 40,
        }
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScenarioFamily {
    AdministrativeActivity,
    AutomationActivity,
    CredentialActivity,
    PermissionChange,
    CredentialPersistence,
    BoundaryRoleActivity,
    BoundaryDataActivity,
    SecretAccess,
    KeyUsage,
    StorageAccess,
    ServerlessControl,
    ContainerControl,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ScenarioTemplate {
    pub number: u8,
    pub category: &'static str,
    pub family: ScenarioFamily,
    pub difficulty: Difficulty,
    pub volume: VolumeTier,
    pub malicious: bool,
    pub path_length: usize,
    pub multi_stage: bool,
    pub cross_boundary: bool,
    pub techniques: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
struct ScenarioShape {
    malicious: bool,
    path_length: usize,
    multi_stage: bool,
    cross_boundary: bool,
}

pub(crate) const EXPANDED_TEMPLATES: [ScenarioTemplate; 12] = [
    template(
        7,
        "administrative_activity_review",
        ScenarioFamily::AdministrativeActivity,
        Difficulty::Introductory,
        VolumeTier::Small,
        shape(false, 0, false, false),
        &[],
    ),
    template(
        8,
        "automation_activity_review",
        ScenarioFamily::AutomationActivity,
        Difficulty::Introductory,
        VolumeTier::Small,
        shape(false, 0, false, true),
        &[],
    ),
    template(
        9,
        "credential_activity_review",
        ScenarioFamily::CredentialActivity,
        Difficulty::Introductory,
        VolumeTier::Small,
        shape(true, 2, false, false),
        &["T1078.004"],
    ),
    template(
        10,
        "permission_change_review",
        ScenarioFamily::PermissionChange,
        Difficulty::Introductory,
        VolumeTier::Small,
        shape(true, 2, false, false),
        &["T1098.003"],
    ),
    template(
        11,
        "credential_persistence_review",
        ScenarioFamily::CredentialPersistence,
        Difficulty::Intermediate,
        VolumeTier::Medium,
        shape(true, 4, true, false),
        &["T1098.001", "T1136.003"],
    ),
    template(
        12,
        "boundary_role_activity",
        ScenarioFamily::BoundaryRoleActivity,
        Difficulty::Advanced,
        VolumeTier::Large,
        shape(true, 5, true, true),
        &["T1078.004", "T1098.003"],
    ),
    template(
        13,
        "boundary_data_activity",
        ScenarioFamily::BoundaryDataActivity,
        Difficulty::Intermediate,
        VolumeTier::Large,
        shape(true, 5, true, true),
        &["T1078.004", "T1530"],
    ),
    template(
        14,
        "secret_access_review",
        ScenarioFamily::SecretAccess,
        Difficulty::Intermediate,
        VolumeTier::Medium,
        shape(true, 4, true, false),
        &["T1078.004", "T1555.006"],
    ),
    template(
        15,
        "key_usage_review",
        ScenarioFamily::KeyUsage,
        Difficulty::Intermediate,
        VolumeTier::Medium,
        shape(true, 4, true, false),
        &["T1078.004", "T1555.006"],
    ),
    template(
        16,
        "storage_access_review",
        ScenarioFamily::StorageAccess,
        Difficulty::Intermediate,
        VolumeTier::Medium,
        shape(true, 5, true, false),
        &["T1078.004", "T1530"],
    ),
    template(
        17,
        "serverless_control_review",
        ScenarioFamily::ServerlessControl,
        Difficulty::Intermediate,
        VolumeTier::Large,
        shape(true, 5, true, false),
        &["T1078.004", "T1098.003"],
    ),
    template(
        18,
        "container_control_review",
        ScenarioFamily::ContainerControl,
        Difficulty::Advanced,
        VolumeTier::Large,
        shape(true, 6, true, true),
        &["T1078.004", "T1098.006"],
    ),
];

const fn template(
    number: u8,
    category: &'static str,
    family: ScenarioFamily,
    difficulty: Difficulty,
    volume: VolumeTier,
    shape: ScenarioShape,
    techniques: &'static [&'static str],
) -> ScenarioTemplate {
    ScenarioTemplate {
        number,
        category,
        family,
        difficulty,
        volume,
        malicious: shape.malicious,
        path_length: shape.path_length,
        multi_stage: shape.multi_stage,
        cross_boundary: shape.cross_boundary,
        techniques,
    }
}

const fn shape(
    malicious: bool,
    path_length: usize,
    multi_stage: bool,
    cross_boundary: bool,
) -> ScenarioShape {
    ScenarioShape {
        malicious,
        path_length,
        multi_stage,
        cross_boundary,
    }
}

pub(crate) const PROVIDERS: [&str; 3] = ["aws", "azure", "gcp"];

pub(crate) fn table(provider: &str) -> &'static str {
    match provider {
        "aws" => "aws_cloudtrail",
        "azure" => "azure_activity",
        "gcp" => "gcp_audit",
        _ => "unsupported",
    }
}

pub(crate) fn telemetry_file(provider: &str) -> &'static str {
    match provider {
        "aws" => "cloudtrail.parquet",
        "azure" => "activity.parquet",
        "gcp" => "audit.parquet",
        _ => "unsupported.parquet",
    }
}
