use std::collections::BTreeMap;

use hunteval_commercial::{
    CommercialError, CommercialOperation, CommercialPlatform, HttpMethod, VendorTarget,
    prepare_vendor_request,
};

#[test]
fn builds_finite_requests_without_caller_transport_fields() {
    let request = prepare_vendor_request(
        CommercialPlatform::CrowdstrikeFalcon,
        CommercialOperation::DetectionsSearch,
        &BTreeMap::from([
            (
                "filter".to_owned(),
                "created_timestamp:>'2026-01-01'".into(),
            ),
            ("limit".to_owned(), 25.into()),
        ]),
        &VendorTarget::empty(),
        100,
    )
    .unwrap_or_else(|error| unreachable!("valid CrowdStrike request: {error}"));
    assert_eq!(request.method, HttpMethod::Get);
    assert_eq!(request.relative_path, "/detects/queries/detects/v1");
    assert!(request.body.is_none());
    assert!(
        request
            .query
            .contains(&("limit".to_owned(), "25".to_owned()))
    );
}

#[test]
fn target_expansion_is_runner_owned_and_path_safe() {
    let target = VendorTarget::new(BTreeMap::from([(
        "instance".to_owned(),
        "projects/project-1/locations/europe/instances/instance-1".to_owned(),
    )]))
    .unwrap_or_else(|error| unreachable!("valid target: {error}"));
    let request = prepare_vendor_request(
        CommercialPlatform::GoogleSecops,
        CommercialOperation::UdmSearch,
        &BTreeMap::from([
            (
                "query".to_owned(),
                "metadata.event_type = USER_LOGIN".into(),
            ),
            ("start_time".to_owned(), "2026-08-10T00:00:00Z".into()),
            ("end_time".to_owned(), "2026-08-10T01:00:00Z".into()),
            ("limit".to_owned(), 50.into()),
        ]),
        &target,
        100,
    )
    .unwrap_or_else(|error| unreachable!("valid Google request: {error}"));
    assert!(request.relative_path.contains("projects/project-1"));
    assert!(!request.relative_path.contains('{'));

    assert_eq!(
        VendorTarget::new(BTreeMap::from([(
            "instance".to_owned(),
            "../private".to_owned(),
        )])),
        Err(CommercialError::InvalidPolicy)
    );
}

#[test]
fn arbitrary_fields_invalid_limits_and_cross_platform_operations_fail() {
    let arbitrary = BTreeMap::from([("url".to_owned(), "https://attacker.invalid".into())]);
    assert_eq!(
        prepare_vendor_request(
            CommercialPlatform::CrowdstrikeFalcon,
            CommercialOperation::DetectionsSearch,
            &arbitrary,
            &VendorTarget::empty(),
            10,
        ),
        Err(CommercialError::DeniedOperation)
    );
    assert_eq!(
        prepare_vendor_request(
            CommercialPlatform::ElasticSecurity,
            CommercialOperation::UdmSearch,
            &BTreeMap::new(),
            &VendorTarget::empty(),
            10,
        ),
        Err(CommercialError::DeniedOperation)
    );
}

#[test]
fn every_shared_platform_operation_has_an_independent_template() {
    let fixtures = [
        (
            CommercialPlatform::GoogleSecops,
            VendorTarget::new(BTreeMap::from([(
                "instance".to_owned(),
                "projects/p/locations/l/instances/i".to_owned(),
            )]))
            .unwrap_or_else(|error| unreachable!("valid target: {error}")),
            BTreeMap::from([("alert_id".to_owned(), "alert-1".into())]),
        ),
        (
            CommercialPlatform::MicrosoftSentinel,
            VendorTarget::new(BTreeMap::from([
                ("subscription".to_owned(), "subscription-1".to_owned()),
                ("resource_group".to_owned(), "resource-group-1".to_owned()),
                ("workspace".to_owned(), "workspace-1".to_owned()),
            ]))
            .unwrap_or_else(|error| unreachable!("valid target: {error}")),
            BTreeMap::from([("incident_id".to_owned(), "incident-1".into())]),
        ),
        (
            CommercialPlatform::ElasticSecurity,
            VendorTarget::empty(),
            BTreeMap::from([
                ("query".to_owned(), serde_json::json!({"match_all": {}})),
                ("limit".to_owned(), 10.into()),
            ]),
        ),
        (
            CommercialPlatform::CortexXsiam,
            VendorTarget::empty(),
            BTreeMap::from([("limit".to_owned(), 10.into())]),
        ),
    ];
    for (platform, target, arguments) in fixtures {
        let request = prepare_vendor_request(
            platform,
            CommercialOperation::AlertsGet,
            &arguments,
            &target,
            100,
        );
        assert!(request.is_ok(), "missing mapping for {platform:?}");
    }
}
