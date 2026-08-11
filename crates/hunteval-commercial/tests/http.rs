use std::collections::BTreeMap;

use hunteval_commercial::{
    BearerSecret, CommercialError, CommercialMode, CommercialOperation, CommercialPlatform,
    CommercialPolicy, CommercialRequest, HttpTransport, ReadOnlyTransport, SecretReference,
    SecretResolver, VendorTarget,
};

#[derive(Debug)]
struct FixtureSecrets;

impl SecretResolver for FixtureSecrets {
    fn resolve(&self, reference: &SecretReference) -> Result<BearerSecret, CommercialError> {
        if reference.as_str() != "fixture-read-only" {
            return Err(CommercialError::InvalidSecretReference);
        }
        BearerSecret::new("canary-value-that-must-not-leak".to_owned())
    }
}

#[test]
fn bearer_secret_debug_output_is_always_redacted() {
    let secret = BearerSecret::new("canary-value-that-must-not-leak".to_owned())
        .unwrap_or_else(|error| unreachable!("valid secret fixture: {error}"));
    let debug = format!("{secret:?}");
    assert_eq!(debug, "BearerSecret([REDACTED])");
    assert!(!debug.contains("canary-value"));
}

#[test]
fn live_transport_rejects_execution_without_a_pinned_resolution() {
    let target = VendorTarget::empty();
    let transport = HttpTransport::new(target, FixtureSecrets);
    let policy = CommercialPolicy {
        policy_version: "0.1".to_owned(),
        mode: CommercialMode::LiveReadOnly,
        platform: CommercialPlatform::CrowdstrikeFalcon,
        origin: "https://api.crowdstrike.example".to_owned(),
        operations: vec![CommercialOperation::DetectionsSearch],
        secret_reference: Some(
            SecretReference::try_from("fixture-read-only".to_owned())
                .unwrap_or_else(|error| unreachable!("valid reference: {error}")),
        ),
        max_requests: 1,
        max_response_bytes: 4_096,
        max_records: 10,
        timeout_ms: 1_000,
    };
    let request = CommercialRequest {
        platform: CommercialPlatform::CrowdstrikeFalcon,
        operation: CommercialOperation::DetectionsSearch,
        tenant_alias: "tenant-test".to_owned(),
        region: "region-test".to_owned(),
        arguments: BTreeMap::from([("limit".to_owned(), 1.into())]),
    };
    assert_eq!(
        transport.execute(&policy, &request),
        Err(CommercialError::DeniedAddress)
    );
}

#[test]
fn malformed_secret_values_are_rejected() {
    assert!(BearerSecret::new("short".to_owned()).is_err());
    assert!(BearerSecret::new("value-with-newline\n".to_owned()).is_err());
}
