use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::{
    CommercialError, CommercialOperation, CommercialPlatform, HttpMethod, operation_descriptor,
    vendor_validation::{
        allow, expand_path, limit, optional_text, optional_u64, push_limit, push_limit_named,
        push_u64, require_keys, required_segment, required_string_array, required_text, text_query,
        valid_name, valid_target_value,
    },
};

const MAX_TARGET_VALUES: usize = 16;
const SENTINEL_API_VERSION: &str = "2025-09-01";
type BuiltArguments = (Vec<(String, String)>, Option<Value>);

/// Runner-owned resource identifiers used to expand trusted path templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VendorTarget {
    values: BTreeMap<String, String>,
}

impl VendorTarget {
    pub fn new(values: BTreeMap<String, String>) -> Result<Self, CommercialError> {
        if values.len() > MAX_TARGET_VALUES
            || values
                .iter()
                .any(|(key, value)| !valid_name(key) || !valid_target_value(value))
        {
            return Err(CommercialError::InvalidPolicy);
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn empty() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }
}

/// Trusted transport request built from a finite operation, not caller HTTP fields.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedVendorRequest {
    pub method: HttpMethod,
    pub relative_path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
}

/// Build a bounded vendor request from an allowlisted operation and argument schema.
pub fn prepare_vendor_request(
    platform: CommercialPlatform,
    operation: CommercialOperation,
    arguments: &BTreeMap<String, Value>,
    target: &VendorTarget,
    maximum_records: u32,
) -> Result<PreparedVendorRequest, CommercialError> {
    if maximum_records == 0 || maximum_records > 100_000 {
        return Err(CommercialError::InvalidPolicy);
    }
    let descriptor = operation_descriptor(platform, operation)?;
    let mut path_values = target.values.clone();
    if let Some(incident) = arguments.get("incident_id") {
        path_values.insert(
            "incident".to_owned(),
            required_segment(incident)?.to_owned(),
        );
    }
    let relative_path = expand_path(descriptor.relative_path, &path_values)?;
    let (query, body) = build_arguments(platform, operation, arguments, maximum_records)?;
    Ok(PreparedVendorRequest {
        method: descriptor.method,
        relative_path,
        query,
        body,
    })
}

fn build_arguments(
    platform: CommercialPlatform,
    operation: CommercialOperation,
    arguments: &BTreeMap<String, Value>,
    maximum_records: u32,
) -> Result<BuiltArguments, CommercialError> {
    use CommercialOperation as O;
    use CommercialPlatform as P;
    match (platform, operation) {
        (P::CrowdstrikeFalcon, O::DetectionsSearch | O::ThreatIntelSearch) => {
            allow(arguments, &["filter", "limit", "offset", "q"])?;
            let mut query = text_query(arguments, &["filter", "q"])?;
            push_limit(&mut query, arguments, "limit", maximum_records)?;
            push_u64(&mut query, arguments, "offset", 100_000)?;
            Ok((query, None))
        }
        (P::CrowdstrikeFalcon, O::DetectionsGet | O::IncidentsGet) => {
            allow(arguments, &["ids"])?;
            let ids = required_string_array(arguments, "ids", maximum_records as usize)?;
            Ok((Vec::new(), Some(serde_json::json!({"ids": ids}))))
        }
        (P::GoogleSecops, O::UdmQueryValidate) => {
            allow(arguments, &["query"])?;
            Ok((
                vec![("query".to_owned(), required_text(arguments, "query")?)],
                None,
            ))
        }
        (P::GoogleSecops, O::UdmSearch) => {
            allow(
                arguments,
                &["end_time", "limit", "page_token", "query", "start_time"],
            )?;
            let mut query = text_query(
                arguments,
                &["query", "start_time", "end_time", "page_token"],
            )?;
            require_keys(arguments, &["query", "start_time", "end_time"])?;
            push_limit(&mut query, arguments, "limit", maximum_records)?;
            Ok((query, None))
        }
        (P::GoogleSecops, O::EventsGet) => {
            allow(arguments, &["event_id"])?;
            Ok((text_query(arguments, &["event_id"])?, None))
        }
        (P::GoogleSecops, O::AlertsGet) => {
            allow(arguments, &["alert_id"])?;
            Ok((
                vec![("alert_id".to_owned(), required_text(arguments, "alert_id")?)],
                None,
            ))
        }
        (P::GoogleSecops, O::CasesGet) => {
            allow(arguments, &["filter", "limit", "page_token"])?;
            let mut query = text_query(arguments, &["filter", "page_token"])?;
            push_limit(&mut query, arguments, "limit", maximum_records)?;
            Ok((query, None))
        }
        (P::MicrosoftSentinel, O::HuntingQuery) => {
            allow(arguments, &["query", "timespan"])?;
            let mut body = Map::new();
            body.insert(
                "query".to_owned(),
                Value::String(required_text(arguments, "query")?),
            );
            if let Some(value) = optional_text(arguments, "timespan")? {
                body.insert("timespan".to_owned(), Value::String(value));
            }
            Ok((Vec::new(), Some(Value::Object(body))))
        }
        (P::MicrosoftSentinel, O::IncidentsGet) => {
            allow(arguments, &["filter", "limit", "orderby", "skip_token"])?;
            let mut query = text_query(arguments, &["filter", "orderby"])?;
            push_limit_named(&mut query, arguments, "limit", "$top", maximum_records)?;
            if let Some(value) = optional_text(arguments, "skip_token")? {
                query.push(("$skipToken".to_owned(), value));
            }
            query.push(("api-version".to_owned(), SENTINEL_API_VERSION.to_owned()));
            Ok((query, None))
        }
        (P::MicrosoftSentinel, O::AlertsGet | O::EntitiesGet) => {
            allow(arguments, &["incident_id"])?;
            required_segment(
                arguments
                    .get("incident_id")
                    .ok_or(CommercialError::InvalidRequest)?,
            )?;
            Ok((
                vec![("api-version".to_owned(), SENTINEL_API_VERSION.to_owned())],
                None,
            ))
        }
        (P::ElasticSecurity, O::SecuritySearch | O::AlertsGet) => {
            allow(arguments, &["from", "limit", "query"])?;
            let mut body = Map::new();
            let query = arguments
                .get("query")
                .cloned()
                .ok_or(CommercialError::InvalidRequest)?;
            if !query.is_object() {
                return Err(CommercialError::InvalidRequest);
            }
            body.insert("query".to_owned(), query);
            body.insert(
                "size".to_owned(),
                Value::from(limit(arguments, "limit", maximum_records)?),
            );
            if let Some(value) = optional_u64(arguments, "from", 100_000)? {
                body.insert("from".to_owned(), Value::from(value));
            }
            Ok((Vec::new(), Some(Value::Object(body))))
        }
        (P::ElasticSecurity, O::InvestigationsGet) => {
            allow(arguments, &["limit", "page"])?;
            let mut query = Vec::new();
            push_limit_named(&mut query, arguments, "limit", "per_page", maximum_records)?;
            push_u64(&mut query, arguments, "page", 10_000)?;
            Ok((query, None))
        }
        (P::CortexXsiam, O::AlertsGet | O::IncidentsGet | O::AuditGet) => {
            allow(arguments, &["filter", "limit"])?;
            let mut body = Map::new();
            if let Some(value) = arguments.get("filter") {
                if !value.is_object() {
                    return Err(CommercialError::InvalidRequest);
                }
                body.insert("filters".to_owned(), value.clone());
            }
            body.insert(
                "limit".to_owned(),
                Value::from(limit(arguments, "limit", maximum_records)?),
            );
            Ok((Vec::new(), Some(Value::Object(body))))
        }
        (P::CortexXsiam, O::QueriesRun) => {
            allow(arguments, &["query", "timeframe"])?;
            let mut body = Map::new();
            body.insert(
                "request_data".to_owned(),
                serde_json::json!({
                    "query": required_text(arguments, "query")?,
                    "timeframe": required_text(arguments, "timeframe")?,
                }),
            );
            Ok((Vec::new(), Some(Value::Object(body))))
        }
        _ => Err(CommercialError::DeniedOperation),
    }
}
