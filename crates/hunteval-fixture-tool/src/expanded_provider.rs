use crate::expanded_catalog::ScenarioFamily;

pub(crate) fn stage_actions(provider: &str, family: ScenarioFamily) -> &'static [&'static str] {
    match (provider, family) {
        ("aws", ScenarioFamily::AdministrativeActivity) => {
            &["AssumeRole", "AttachRolePolicy", "UpdateAssumeRolePolicy"]
        }
        ("aws", ScenarioFamily::AutomationActivity) => &[
            "AssumeRole",
            "CreateAccessKey",
            "GetSecretValue",
            "GetObject",
        ],
        ("aws", ScenarioFamily::CredentialActivity) => &["ConsoleLogin", "GetCallerIdentity"],
        ("aws", ScenarioFamily::PermissionChange) => &["PutRolePolicy", "AttachRolePolicy"],
        ("aws", ScenarioFamily::CredentialPersistence) => &[
            "ConsoleLogin",
            "CreateAccessKey",
            "UpdateAssumeRolePolicy",
            "GetCallerIdentity",
        ],
        ("aws", ScenarioFamily::BoundaryRoleActivity) => &[
            "ConsoleLogin",
            "AssumeRole",
            "ListRoles",
            "AssumeRole",
            "GetCallerIdentity",
        ],
        ("aws", ScenarioFamily::BoundaryDataActivity) => &[
            "AssumeRole",
            "ListBuckets",
            "ListObjectsV2",
            "GetObject",
            "Decrypt",
        ],
        ("aws", ScenarioFamily::SecretAccess) => &[
            "GetCallerIdentity",
            "ListSecrets",
            "DescribeSecret",
            "GetSecretValue",
        ],
        ("aws", ScenarioFamily::KeyUsage) => &[
            "GetCallerIdentity",
            "DescribeKey",
            "GenerateDataKey",
            "Decrypt",
        ],
        ("aws", ScenarioFamily::StorageAccess) => &[
            "ListBuckets",
            "GetBucketLocation",
            "ListObjectsV2",
            "HeadObject",
            "GetObject",
        ],
        ("aws", ScenarioFamily::ServerlessControl) => &[
            "GetFunction",
            "UpdateFunctionCode",
            "AddPermission",
            "UpdateFunctionConfiguration",
            "Invoke",
        ],
        ("aws", ScenarioFamily::ContainerControl) => &[
            "DescribeCluster",
            "AccessKubernetesApi",
            "CreateAccessEntry",
            "AssociateAccessPolicy",
            "UpdateClusterConfig",
            "ListSecrets",
        ],
        ("azure", ScenarioFamily::AdministrativeActivity) => {
            &["SignIn", "RoleAssignmentWrite", "PolicyAssignmentWrite"]
        }
        ("azure", ScenarioFamily::AutomationActivity) => &[
            "ServicePrincipalSignIn",
            "RoleAssignmentWrite",
            "SecretGet",
            "BlobRead",
        ],
        ("azure", ScenarioFamily::CredentialActivity) => &["SignIn", "TokenIssued"],
        ("azure", ScenarioFamily::PermissionChange) => {
            &["RoleAssignmentWrite", "DirectoryRoleMemberAdd"]
        }
        ("azure", ScenarioFamily::CredentialPersistence) => &[
            "SignIn",
            "ServicePrincipalCredentialAdd",
            "ApplicationUpdate",
            "TokenIssued",
        ],
        ("azure", ScenarioFamily::BoundaryRoleActivity) => &[
            "SignIn",
            "RoleAssignmentWrite",
            "SubscriptionList",
            "ManagedIdentityToken",
            "ResourceRead",
        ],
        ("azure", ScenarioFamily::BoundaryDataActivity) => &[
            "ManagedIdentityToken",
            "StorageAccountListKeys",
            "ContainerList",
            "BlobRead",
            "KeyDecrypt",
        ],
        ("azure", ScenarioFamily::SecretAccess) => {
            &["SignIn", "VaultRead", "SecretList", "SecretGet"]
        }
        ("azure", ScenarioFamily::KeyUsage) => &["SignIn", "KeyRead", "KeyWrap", "KeyDecrypt"],
        ("azure", ScenarioFamily::StorageAccess) => &[
            "StorageAccountList",
            "ContainerList",
            "BlobPropertiesRead",
            "BlobList",
            "BlobRead",
        ],
        ("azure", ScenarioFamily::ServerlessControl) => &[
            "FunctionRead",
            "FunctionCodeWrite",
            "FunctionConfigWrite",
            "RoleAssignmentWrite",
            "FunctionInvoke",
        ],
        ("azure", ScenarioFamily::ContainerControl) => &[
            "ManagedClusterRead",
            "ListClusterCredentials",
            "RoleAssignmentWrite",
            "AgentPoolWrite",
            "ManagedClusterWrite",
            "SecretGet",
        ],
        ("gcp", ScenarioFamily::AdministrativeActivity) => &[
            "GenerateAccessToken",
            "SetIamPolicy",
            "ServiceAccountUpdate",
        ],
        ("gcp", ScenarioFamily::AutomationActivity) => &[
            "GenerateAccessToken",
            "CreateServiceAccountKey",
            "AccessSecretVersion",
            "StorageObjectsGet",
        ],
        ("gcp", ScenarioFamily::CredentialActivity) => {
            &["GenerateAccessToken", "GetServiceAccount"]
        }
        ("gcp", ScenarioFamily::PermissionChange) => &["SetIamPolicy", "SetOrgPolicy"],
        ("gcp", ScenarioFamily::CredentialPersistence) => &[
            "GenerateAccessToken",
            "CreateServiceAccountKey",
            "SetIamPolicy",
            "GetServiceAccount",
        ],
        ("gcp", ScenarioFamily::BoundaryRoleActivity) => &[
            "GenerateAccessToken",
            "SetIamPolicy",
            "ProjectsList",
            "GenerateAccessToken",
            "GetIamPolicy",
        ],
        ("gcp", ScenarioFamily::BoundaryDataActivity) => &[
            "GenerateAccessToken",
            "StorageBucketsList",
            "StorageObjectsList",
            "StorageObjectsGet",
            "Decrypt",
        ],
        ("gcp", ScenarioFamily::SecretAccess) => &[
            "GenerateAccessToken",
            "ListSecrets",
            "GetSecret",
            "AccessSecretVersion",
        ],
        ("gcp", ScenarioFamily::KeyUsage) => {
            &["GenerateAccessToken", "GetCryptoKey", "Encrypt", "Decrypt"]
        }
        ("gcp", ScenarioFamily::StorageAccess) => &[
            "StorageBucketsList",
            "StorageBucketsGet",
            "StorageObjectsList",
            "StorageObjectsGetIamPolicy",
            "StorageObjectsGet",
        ],
        ("gcp", ScenarioFamily::ServerlessControl) => &[
            "FunctionsGet",
            "FunctionsUpdate",
            "RunServicesUpdate",
            "SetIamPolicy",
            "RunRoutesInvoke",
        ],
        ("gcp", ScenarioFamily::ContainerControl) => &[
            "ClustersGet",
            "ClustersGetCredentials",
            "SetIamPolicy",
            "ClustersUpdate",
            "PodsCreate",
            "SecretsList",
        ],
        _ => &["Unsupported"],
    }
}

pub(crate) fn services(provider: &str, family: ScenarioFamily) -> &'static [&'static str] {
    match (provider, family) {
        ("aws", ScenarioFamily::BoundaryRoleActivity) => {
            &["CloudTrail", "IAM", "STS", "Organizations"]
        }
        ("aws", ScenarioFamily::BoundaryDataActivity) => &["CloudTrail", "STS", "S3", "KMS"],
        ("aws", ScenarioFamily::SecretAccess) => &["CloudTrail", "IAM", "SecretsManager"],
        ("aws", ScenarioFamily::KeyUsage) => &["CloudTrail", "IAM", "KMS"],
        ("aws", ScenarioFamily::StorageAccess) => &["CloudTrail", "S3"],
        ("aws", ScenarioFamily::ServerlessControl) => &["CloudTrail", "IAM", "Lambda"],
        ("aws", ScenarioFamily::ContainerControl) => &["CloudTrail", "IAM", "EKS"],
        ("aws", _) => &["CloudTrail", "IAM", "STS"],
        ("azure", ScenarioFamily::BoundaryRoleActivity) => {
            &["EntraID", "AzureActivity", "RBAC", "ManagedIdentity"]
        }
        ("azure", ScenarioFamily::BoundaryDataActivity) => &[
            "AzureActivity",
            "ManagedIdentity",
            "StorageAccounts",
            "KeyVault",
        ],
        ("azure", ScenarioFamily::SecretAccess | ScenarioFamily::KeyUsage) => {
            &["EntraID", "AzureActivity", "KeyVault"]
        }
        ("azure", ScenarioFamily::StorageAccess) => &["AzureActivity", "StorageAccounts"],
        ("azure", ScenarioFamily::ServerlessControl) => {
            &["AzureActivity", "EntraID", "AzureFunctions"]
        }
        ("azure", ScenarioFamily::ContainerControl) => &["AzureActivity", "EntraID", "AKS"],
        ("azure", _) => &["EntraID", "AzureActivity", "RBAC"],
        ("gcp", ScenarioFamily::BoundaryRoleActivity) => {
            &["CloudAuditLogs", "IAM", "ServiceAccountCredentials"]
        }
        ("gcp", ScenarioFamily::BoundaryDataActivity) => {
            &["CloudAuditLogs", "IAM", "CloudStorage", "CloudKMS"]
        }
        ("gcp", ScenarioFamily::SecretAccess) => &["CloudAuditLogs", "IAM", "SecretManager"],
        ("gcp", ScenarioFamily::KeyUsage) => &["CloudAuditLogs", "IAM", "CloudKMS"],
        ("gcp", ScenarioFamily::StorageAccess) => &["CloudAuditLogs", "CloudStorage"],
        ("gcp", ScenarioFamily::ServerlessControl) => {
            &["CloudAuditLogs", "IAM", "CloudFunctions", "CloudRun"]
        }
        ("gcp", ScenarioFamily::ContainerControl) => &["CloudAuditLogs", "IAM", "GKE"],
        ("gcp", _) => &["CloudAuditLogs", "IAM", "ServiceAccounts"],
        _ => &["Unsupported"],
    }
}

pub(crate) fn noise_actions(provider: &str) -> &'static [&'static str] {
    match provider {
        "aws" => &[
            "ListUsers",
            "DescribeRegions",
            "GetAccountSummary",
            "ListBuckets",
            "DescribeInstances",
            "ListFunctions",
        ],
        "azure" => &[
            "UserRead",
            "SubscriptionRead",
            "ResourceGroupRead",
            "VaultRead",
            "StorageAccountRead",
            "FunctionRead",
        ],
        "gcp" => &[
            "ProjectsGet",
            "ServiceAccountsList",
            "GetIamPolicy",
            "StorageBucketsGet",
            "InstancesList",
            "FunctionsGet",
        ],
        _ => &["Unsupported"],
    }
}
