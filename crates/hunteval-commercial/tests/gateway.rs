use std::{collections::BTreeMap, net::IpAddr};

use hunteval_commercial::{
    CommercialError, CommercialGateway, CommercialMode, CommercialOperation, CommercialPlatform,
    CommercialPolicy, CommercialRequest, CommercialResponse, CommercialService, GatewayRequest,
    GatewayResponse, ReadOnlyTransport, ResolvedAddress, normalize_vendor_response,
    operation_descriptor,
};

#[derive(Debug)]
struct FixtureTransport;

impl ReadOnlyTransport for FixtureTransport {
    fn resolve(&self, _: &str) -> Result<Vec<ResolvedAddress>, CommercialError> {
        Ok(vec![ResolvedAddress(IpAddr::from([8, 8, 8, 8]))])
    }

    fn execute(
        &self,
        _: &CommercialPolicy,
        _: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError> {
        Ok(CommercialResponse {
            records: vec![BTreeMap::from([("source_id".to_owned(), "alert-1".into())])],
            truncated: false,
            more_available: false,
        })
    }
}

fn gateway() -> CommercialGateway<FixtureTransport> {
    let policy = CommercialPolicy {
        policy_version: "0.1".to_owned(),
        mode: CommercialMode::FixtureReplay,
        platform: CommercialPlatform::CrowdstrikeFalcon,
        origin: "https://api.crowdstrike.example".to_owned(),
        operations: vec![CommercialOperation::DetectionsSearch],
        secret_reference: None,
        max_requests: 1,
        max_response_bytes: 4_096,
        max_records: 10,
        timeout_ms: 1_000,
    };
    let service = CommercialService::new(policy, FixtureTransport)
        .unwrap_or_else(|error| unreachable!("valid fixture policy: {error}"));
    CommercialGateway::new(service)
}

fn request() -> GatewayRequest {
    GatewayRequest {
        request_id: "request-1".to_owned(),
        agent_id: "agent-1".to_owned(),
        task_id: "task-1".to_owned(),
        action_id: "action-1".to_owned(),
        request: CommercialRequest {
            platform: CommercialPlatform::CrowdstrikeFalcon,
            operation: CommercialOperation::DetectionsSearch,
            tenant_alias: "tenant-test".to_owned(),
            region: "region-test".to_owned(),
            arguments: BTreeMap::from([("limit".to_owned(), 1.into())]),
        },
    }
}

#[test]
fn gateway_returns_a_correlated_runner_managed_result() {
    let response = gateway().execute(&request());
    assert!(matches!(
        response,
        GatewayResponse::Success { request_id, action_id, result }
            if request_id == "request-1" && action_id == "action-1" && result.records.len() == 1
    ));
}

#[test]
fn gateway_rejects_invalid_identity_without_losing_correlation() {
    let mut invalid = request();
    invalid.agent_id = "../agent".to_owned();
    let response = gateway().execute(&invalid);
    assert!(matches!(
        response,
        GatewayResponse::Error { request_id, action_id, reason_code }
            if request_id == "request-1" && action_id == "action-1" && reason_code == "invalid_request"
    ));
}

#[test]
fn vendor_descriptors_are_relative_and_normalization_is_bounded() {
    for operation in [
        CommercialOperation::DetectionsSearch,
        CommercialOperation::UdmSearch,
        CommercialOperation::HuntingQuery,
        CommercialOperation::SecuritySearch,
        CommercialOperation::QueriesRun,
    ] {
        let platform = match operation {
            CommercialOperation::DetectionsSearch => CommercialPlatform::CrowdstrikeFalcon,
            CommercialOperation::UdmSearch => CommercialPlatform::GoogleSecops,
            CommercialOperation::HuntingQuery => CommercialPlatform::MicrosoftSentinel,
            CommercialOperation::SecuritySearch => CommercialPlatform::ElasticSecurity,
            CommercialOperation::QueriesRun => CommercialPlatform::CortexXsiam,
            _ => unreachable!("operation fixture is exhaustive"),
        };
        let descriptor = operation_descriptor(platform, operation)
            .unwrap_or_else(|error| unreachable!("valid operation mapping: {error}"));
        assert!(descriptor.relative_path.starts_with('/'));
        assert!(!descriptor.relative_path.contains("://"));
    }

    let normalized = normalize_vendor_response(
        CommercialPlatform::CrowdstrikeFalcon,
        CommercialOperation::DetectionsSearch,
        &serde_json::json!({
            "resources": [{"id":"one"}, {"id":"two"}],
            "meta": {"pagination": {"after":"next"}}
        }),
        1,
    )
    .unwrap_or_else(|error| unreachable!("valid vendor fixture: {error}"));
    assert_eq!(normalized.records.len(), 1);
    assert!(normalized.truncated);
    assert!(normalized.more_available);
}

#[test]
fn malformed_vendor_collections_fail_closed() {
    assert_eq!(
        normalize_vendor_response(
            CommercialPlatform::GoogleSecops,
            CommercialOperation::UdmSearch,
            &serde_json::json!({"events":"not-a-collection"}),
            10,
        ),
        Err(CommercialError::InvalidResponse)
    );
}

#[test]
fn shared_operation_names_resolve_by_platform() {
    let google = operation_descriptor(
        CommercialPlatform::GoogleSecops,
        CommercialOperation::AlertsGet,
    )
    .unwrap_or_else(|error| unreachable!("valid Google mapping: {error}"));
    let sentinel = operation_descriptor(
        CommercialPlatform::MicrosoftSentinel,
        CommercialOperation::AlertsGet,
    )
    .unwrap_or_else(|error| unreachable!("valid Sentinel mapping: {error}"));
    assert_ne!(google.relative_path, sentinel.relative_path);
    assert_eq!(
        operation_descriptor(
            CommercialPlatform::ElasticSecurity,
            CommercialOperation::UdmSearch,
        ),
        Err(CommercialError::DeniedOperation)
    );
}
