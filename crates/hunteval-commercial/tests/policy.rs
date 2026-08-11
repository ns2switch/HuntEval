use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr},
};

use hunteval_commercial::{
    CommercialError, CommercialMode, CommercialOperation, CommercialPlatform, CommercialPolicy,
    CommercialRequest, CommercialResponse, CommercialService, ReadOnlyTransport, ResolvedAddress,
    SecretReference,
};

#[derive(Debug)]
struct FixtureTransport {
    address: IpAddr,
}

impl ReadOnlyTransport for FixtureTransport {
    fn resolve(&self, _: &str) -> Result<Vec<ResolvedAddress>, CommercialError> {
        Ok(vec![ResolvedAddress(self.address)])
    }

    fn execute(
        &self,
        _: &CommercialPolicy,
        _: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError> {
        Ok(CommercialResponse {
            records: vec![BTreeMap::from([("source_id".into(), "record-1".into())])],
            truncated: false,
            more_available: false,
        })
    }
}

#[derive(Debug)]
struct SensitiveResponseTransport;

impl ReadOnlyTransport for SensitiveResponseTransport {
    fn resolve(&self, _: &str) -> Result<Vec<ResolvedAddress>, CommercialError> {
        Ok(vec![ResolvedAddress(IpAddr::from([8, 8, 8, 8]))])
    }

    fn execute(
        &self,
        _: &CommercialPolicy,
        _: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError> {
        Ok(CommercialResponse {
            records: vec![BTreeMap::from([(
                "access_token".into(),
                "remote-secret".into(),
            )])],
            truncated: false,
            more_available: false,
        })
    }
}

fn policy() -> CommercialPolicy {
    CommercialPolicy {
        policy_version: "0.1".into(),
        mode: CommercialMode::LiveReadOnly,
        platform: CommercialPlatform::CrowdstrikeFalcon,
        origin: "https://api.crowdstrike.example".into(),
        operations: vec![CommercialOperation::DetectionsSearch],
        secret_reference: Some(
            SecretReference::try_from("falcon-read-only".to_owned())
                .unwrap_or_else(|error| unreachable!("valid fixture secret reference: {error}")),
        ),
        max_requests: 1,
        max_response_bytes: 4096,
        max_records: 10,
        timeout_ms: 1000,
    }
}

fn request() -> CommercialRequest {
    CommercialRequest {
        platform: CommercialPlatform::CrowdstrikeFalcon,
        operation: CommercialOperation::DetectionsSearch,
        tenant_alias: "tenant-test".into(),
        region: "region-test".into(),
        arguments: BTreeMap::from([("limit".into(), 1.into())]),
    }
}

#[test]
fn exact_read_only_request_passes_with_public_resolution() {
    let transport = FixtureTransport {
        address: "8.8.8.8"
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };
    let mut service = CommercialService::new(policy(), transport)
        .unwrap_or_else(|error| unreachable!("valid policy fixture: {error}"));
    let response = service
        .execute(&request())
        .unwrap_or_else(|error| unreachable!("valid request fixture: {error}"));
    assert_eq!(response.records.len(), 1);
    assert_eq!(
        service.execute(&request()),
        Err(CommercialError::InvalidRequest)
    );
}

#[test]
fn local_metadata_and_private_destinations_fail_closed() {
    for address in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "192.0.2.1",
        "::1",
        "fc00::1",
    ] {
        let parsed = address.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let mut service = CommercialService::new(policy(), FixtureTransport { address: parsed })
            .unwrap_or_else(|error| unreachable!("valid policy fixture: {error}"));
        assert_eq!(
            service.execute(&request()),
            Err(CommercialError::DeniedAddress)
        );
    }
}

#[test]
fn arbitrary_transport_and_cross_platform_operations_are_unrepresentable_or_denied() {
    let mut changed = request();
    changed.arguments.insert(
        "filter".into(),
        serde_json::json!({"nested": {"url": "https://attacker.invalid"}}),
    );
    let mut service = CommercialService::new(
        policy(),
        FixtureTransport {
            address: "8.8.8.8"
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        },
    )
    .unwrap_or_else(|error| unreachable!("valid policy fixture: {error}"));
    assert_eq!(
        service.execute(&changed),
        Err(CommercialError::DeniedOperation)
    );

    let mut changed = request();
    changed.operation = CommercialOperation::UdmSearch;
    assert_eq!(
        service.execute(&changed),
        Err(CommercialError::DeniedOperation)
    );
}

#[test]
fn policy_rejects_non_https_ip_and_path_origins() {
    for origin in [
        "http://vendor.example",
        "https://127.0.0.1",
        "https://vendor.example/path",
    ] {
        let mut changed = policy();
        changed.origin = origin.into();
        assert!(changed.validate().is_err());
    }
    assert_eq!(
        policy()
            .secret_reference
            .as_ref()
            .map(SecretReference::identity_sha256)
            .map(|value| value.len()),
        Some(64)
    );
    assert_eq!(policy().sha256().map(|value| value.len()), Ok(64));
}

#[test]
fn credentials_are_required_only_for_live_read_only_mode() {
    let mut fixture = policy();
    fixture.mode = CommercialMode::FixtureReplay;
    fixture.secret_reference = None;
    assert!(fixture.validate().is_ok());

    let mut invalid_live = policy();
    invalid_live.secret_reference = None;
    assert_eq!(invalid_live.validate(), Err(CommercialError::InvalidPolicy));

    let mut invalid_fixture = fixture;
    invalid_fixture.secret_reference = Some(
        SecretReference::try_from("unexpected-secret".to_owned())
            .unwrap_or_else(|error| unreachable!("valid fixture reference: {error}")),
    );
    assert_eq!(
        invalid_fixture.validate(),
        Err(CommercialError::InvalidPolicy)
    );
}

#[test]
fn sensitive_request_and_response_field_variants_fail_closed() {
    let mut sensitive_request = request();
    sensitive_request
        .arguments
        .insert("client_secret".into(), "not-allowed".into());
    let mut service = CommercialService::new(
        policy(),
        FixtureTransport {
            address: IpAddr::from([8, 8, 8, 8]),
        },
    )
    .unwrap_or_else(|error| unreachable!("valid policy fixture: {error}"));
    assert_eq!(
        service.execute(&sensitive_request),
        Err(CommercialError::DeniedOperation)
    );

    let mut service = CommercialService::new(policy(), SensitiveResponseTransport)
        .unwrap_or_else(|error| unreachable!("valid policy fixture: {error}"));
    assert_eq!(
        service.execute(&request()),
        Err(CommercialError::InvalidResponse)
    );
}
