use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{CommercialError, CommercialOperation, CommercialPlatform, CommercialResponse};

/// HTTP methods selected by trusted vendor code, never by an evaluated agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
}

/// Finite transport mapping for a documented read-only vendor operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationDescriptor {
    pub method: HttpMethod,
    pub relative_path: &'static str,
    pub response_collection: &'static str,
}

/// Resolve one platform operation to a trusted relative path and response collection.
pub fn operation_descriptor(
    platform: CommercialPlatform,
    operation: CommercialOperation,
) -> Result<OperationDescriptor, CommercialError> {
    use CommercialOperation as O;
    use CommercialPlatform as P;
    if !platform.supports(operation) {
        return Err(CommercialError::DeniedOperation);
    }
    let value = match (platform, operation) {
        (P::CrowdstrikeFalcon, O::DetectionsSearch) => {
            descriptor(HttpMethod::Get, "/detects/queries/detects/v1", "resources")
        }
        (P::CrowdstrikeFalcon, O::DetectionsGet) => descriptor(
            HttpMethod::Post,
            "/detects/entities/summaries/GET/v1",
            "resources",
        ),
        (P::CrowdstrikeFalcon, O::IncidentsGet) => descriptor(
            HttpMethod::Post,
            "/incidents/entities/incidents/GET/v1",
            "resources",
        ),
        (P::CrowdstrikeFalcon, O::ThreatIntelSearch) => {
            descriptor(HttpMethod::Get, "/intel/queries/indicators/v1", "resources")
        }
        (P::GoogleSecops, O::UdmQueryValidate) => descriptor(
            HttpMethod::Get,
            "/v1alpha/{instance}:validateQuery",
            "result",
        ),
        (P::GoogleSecops, O::UdmSearch) => {
            descriptor(HttpMethod::Get, "/v1alpha/{instance}:udmSearch", "events")
        }
        (P::GoogleSecops, O::EventsGet) => {
            descriptor(HttpMethod::Get, "/v1alpha/{instance}/events", "events")
        }
        (P::GoogleSecops, O::AlertsGet) => descriptor(
            HttpMethod::Get,
            "/v1alpha/{instance}/legacy:legacyGetAlert",
            "alerts",
        ),
        (P::GoogleSecops, O::CasesGet) => {
            descriptor(HttpMethod::Get, "/v1beta/{instance}/cases", "cases")
        }
        (P::MicrosoftSentinel, O::HuntingQuery) => descriptor(
            HttpMethod::Post,
            "/v1/workspaces/{workspace}/query",
            "tables",
        ),
        (P::MicrosoftSentinel, O::IncidentsGet) => descriptor(
            HttpMethod::Get,
            "/subscriptions/{subscription}/resourceGroups/{resource_group}/providers/Microsoft.OperationalInsights/workspaces/{workspace}/providers/Microsoft.SecurityInsights/incidents",
            "value",
        ),
        (P::MicrosoftSentinel, O::AlertsGet) => descriptor(
            HttpMethod::Get,
            "/subscriptions/{subscription}/resourceGroups/{resource_group}/providers/Microsoft.OperationalInsights/workspaces/{workspace}/providers/Microsoft.SecurityInsights/incidents/{incident}/alerts",
            "value",
        ),
        (P::MicrosoftSentinel, O::EntitiesGet) => descriptor(
            HttpMethod::Get,
            "/subscriptions/{subscription}/resourceGroups/{resource_group}/providers/Microsoft.OperationalInsights/workspaces/{workspace}/providers/Microsoft.SecurityInsights/incidents/{incident}/entities",
            "entities",
        ),
        (P::ElasticSecurity, O::SecuritySearch) => {
            descriptor(HttpMethod::Post, "/{index}/_search", "hits.hits")
        }
        (P::ElasticSecurity, O::AlertsGet) => descriptor(
            HttpMethod::Post,
            "/api/detection_engine/signals/search",
            "hits.hits",
        ),
        (P::ElasticSecurity, O::InvestigationsGet) => {
            descriptor(HttpMethod::Get, "/api/attack_discovery/_find", "data")
        }
        (P::CortexXsiam, O::AlertsGet) => descriptor(
            HttpMethod::Post,
            "/public_api/v1/alerts/get_alerts_multi_events",
            "reply.alerts",
        ),
        (P::CortexXsiam, O::IncidentsGet) => descriptor(
            HttpMethod::Post,
            "/public_api/v1/incidents/get_incidents",
            "reply.incidents",
        ),
        (P::CortexXsiam, O::QueriesRun) => descriptor(
            HttpMethod::Post,
            "/public_api/v1/xql/start_xql_query",
            "reply",
        ),
        (P::CortexXsiam, O::AuditGet) => descriptor(
            HttpMethod::Post,
            "/public_api/v1/audits/management_logs",
            "reply.data",
        ),
        _ => return Err(CommercialError::DeniedOperation),
    };
    Ok(value)
}

const fn descriptor(
    method: HttpMethod,
    relative_path: &'static str,
    response_collection: &'static str,
) -> OperationDescriptor {
    OperationDescriptor {
        method,
        relative_path,
        response_collection,
    }
}

/// Normalize a bounded vendor JSON response without treating it as ground truth.
pub fn normalize_vendor_response(
    platform: CommercialPlatform,
    operation: CommercialOperation,
    response: &Value,
    max_records: usize,
) -> Result<CommercialResponse, CommercialError> {
    if max_records == 0 || max_records > 100_000 {
        return Err(CommercialError::InvalidResponse);
    }
    let descriptor = operation_descriptor(platform, operation)?;
    let collection = select_path(response, descriptor.response_collection)
        .ok_or(CommercialError::InvalidResponse)?;
    let values: Vec<&Value> = match collection {
        Value::Array(values) => values.iter().collect(),
        Value::Object(_) => vec![collection],
        _ => return Err(CommercialError::InvalidResponse),
    };
    let more_available = values.len() > max_records || pagination_present(response);
    let records = values
        .into_iter()
        .take(max_records)
        .map(normalize_record)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CommercialResponse {
        truncated: more_available,
        more_available,
        records,
    })
}

fn select_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |current, component| {
        current.as_object()?.get(component)
    })
}

fn normalize_record(value: &Value) -> Result<BTreeMap<String, Value>, CommercialError> {
    match value {
        Value::Object(record) => Ok(record
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()),
        scalar => Ok(BTreeMap::from([("value".to_owned(), scalar.clone())])),
    }
}

fn pagination_present(response: &Value) -> bool {
    [
        "nextLink",
        "next_page_token",
        "nextPageToken",
        "after",
        "cursor",
    ]
    .into_iter()
    .any(|key| response.get(key).is_some_and(non_empty))
        || response
            .get("meta")
            .and_then(|value| value.get("pagination"))
            .is_some_and(|pagination| {
                ["after", "offset", "next"]
                    .into_iter()
                    .any(|key| pagination.get(key).is_some_and(non_empty))
            })
}

fn non_empty(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
        _ => true,
    }
}
