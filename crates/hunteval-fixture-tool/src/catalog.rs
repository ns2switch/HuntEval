#[derive(Debug, Clone, Copy)]
pub(crate) struct EpisodeSpec {
    pub provider: &'static str,
    pub id: &'static str,
    pub category: &'static str,
    pub table: &'static str,
    pub telemetry_file: &'static str,
}

pub(crate) const EPISODES: [EpisodeSpec; 9] = [
    EpisodeSpec {
        provider: "aws",
        id: "aws-iam-001",
        category: "identity_compromise",
        table: "aws_cloudtrail",
        telemetry_file: "cloudtrail.parquet",
    },
    EpisodeSpec {
        provider: "aws",
        id: "aws-iam-002",
        category: "privilege_escalation",
        table: "aws_cloudtrail",
        telemetry_file: "cloudtrail.parquet",
    },
    EpisodeSpec {
        provider: "aws",
        id: "aws-iam-003",
        category: "persistence_credential_creation",
        table: "aws_cloudtrail",
        telemetry_file: "cloudtrail.parquet",
    },
    EpisodeSpec {
        provider: "azure",
        id: "azure-iam-001",
        category: "identity_compromise",
        table: "azure_activity",
        telemetry_file: "activity.parquet",
    },
    EpisodeSpec {
        provider: "azure",
        id: "azure-iam-002",
        category: "privilege_escalation",
        table: "azure_activity",
        telemetry_file: "activity.parquet",
    },
    EpisodeSpec {
        provider: "azure",
        id: "azure-iam-003",
        category: "persistence_credential_creation",
        table: "azure_activity",
        telemetry_file: "activity.parquet",
    },
    EpisodeSpec {
        provider: "gcp",
        id: "gcp-iam-001",
        category: "identity_compromise",
        table: "gcp_audit",
        telemetry_file: "audit.parquet",
    },
    EpisodeSpec {
        provider: "gcp",
        id: "gcp-iam-002",
        category: "privilege_escalation",
        table: "gcp_audit",
        telemetry_file: "audit.parquet",
    },
    EpisodeSpec {
        provider: "gcp",
        id: "gcp-iam-003",
        category: "persistence_credential_creation",
        table: "gcp_audit",
        telemetry_file: "audit.parquet",
    },
];
