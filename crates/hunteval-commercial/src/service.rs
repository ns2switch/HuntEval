use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{CommercialOperation, CommercialPlatform, CommercialPolicy, ResolvedAddress};

/// Typed commercial connector failures safe for orchestration logic.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CommercialError {
    #[error("commercial policy is invalid")]
    InvalidPolicy,
    #[error("commercial origin is invalid")]
    InvalidOrigin,
    #[error("commercial resolved address is denied")]
    DeniedAddress,
    #[error("commercial secret reference is invalid")]
    InvalidSecretReference,
    #[error("commercial operation is denied")]
    DeniedOperation,
    #[error("commercial request is invalid or oversized")]
    InvalidRequest,
    #[error("commercial response is invalid or oversized")]
    InvalidResponse,
    #[error("commercial transport failed")]
    TransportFailure,
}

impl CommercialError {
    /// Stable public reason code without transport, endpoint, or secret detail.
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPolicy => "invalid_policy",
            Self::InvalidOrigin => "invalid_origin",
            Self::DeniedAddress => "denied_address",
            Self::InvalidSecretReference => "invalid_secret_reference",
            Self::DeniedOperation => "denied_operation",
            Self::InvalidRequest => "invalid_request",
            Self::InvalidResponse => "invalid_response",
            Self::TransportFailure => "transport_failure",
        }
    }
}

/// Request with no URL, method, headers, or credential fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommercialRequest {
    pub platform: CommercialPlatform,
    pub operation: CommercialOperation,
    pub tenant_alias: String,
    pub region: String,
    pub arguments: BTreeMap<String, Value>,
}

/// Bounded source-provenanced response returned by a transport adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommercialResponse {
    pub records: Vec<BTreeMap<String, Value>>,
    pub truncated: bool,
    pub more_available: bool,
}

/// Infrastructure transport implementing one finite platform catalog.
pub trait ReadOnlyTransport {
    fn resolve(&self, origin: &str) -> Result<Vec<ResolvedAddress>, CommercialError>;
    fn execute(
        &self,
        policy: &CommercialPolicy,
        request: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError>;
}

/// Applies runner-owned policy before an injected transport can execute.
#[derive(Debug)]
pub struct CommercialService<T> {
    policy: CommercialPolicy,
    transport: T,
    used_requests: u32,
}

impl<T: ReadOnlyTransport> CommercialService<T> {
    pub fn new(policy: CommercialPolicy, transport: T) -> Result<Self, CommercialError> {
        policy.validate()?;
        Ok(Self {
            policy,
            transport,
            used_requests: 0,
        })
    }

    pub fn execute(
        &mut self,
        request: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError> {
        self.validate_request(request)?;
        if self.used_requests >= self.policy.max_requests {
            return Err(CommercialError::InvalidRequest);
        }
        let addresses = self.transport.resolve(&self.policy.origin)?;
        if addresses.is_empty() || addresses.len() > 32 {
            return Err(CommercialError::DeniedAddress);
        }
        addresses
            .iter()
            .try_for_each(|address| address.validate())?;
        let response = self.transport.execute(&self.policy, request)?;
        self.validate_response(&response)?;
        self.used_requests += 1;
        Ok(response)
    }

    fn validate_request(&self, request: &CommercialRequest) -> Result<(), CommercialError> {
        let valid_identity =
            valid_identifier(&request.tenant_alias) && valid_identifier(&request.region);
        let bytes =
            serde_json::to_vec(&request.arguments).map_err(|_| CommercialError::InvalidRequest)?;
        if request.platform != self.policy.platform
            || !self.policy.operations.contains(&request.operation)
            || !request.platform.supports(request.operation)
            || !valid_identity
            || bytes.len() > 64 * 1024
            || request
                .arguments
                .iter()
                .any(|(key, value)| forbidden_request_value(key, value, 0))
        {
            return Err(CommercialError::DeniedOperation);
        }
        Ok(())
    }

    fn validate_response(&self, response: &CommercialResponse) -> Result<(), CommercialError> {
        let bytes = serde_json::to_vec(response).map_err(|_| CommercialError::InvalidResponse)?;
        if response.records.len() > self.policy.max_records as usize
            || bytes.len() as u64 > self.policy.max_response_bytes
            || response.records.iter().any(|record| {
                record
                    .iter()
                    .any(|(key, value)| forbidden_response_value(key, value, 0))
            })
        {
            return Err(CommercialError::InvalidResponse);
        }
        Ok(())
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && b"._:-".contains(&byte))
        })
}

fn forbidden_request_value(key: &str, value: &Value, depth: usize) -> bool {
    forbidden_key(key)
        || depth > 16
        || match value {
            Value::Object(values) => values
                .iter()
                .any(|(nested_key, nested)| forbidden_request_value(nested_key, nested, depth + 1)),
            Value::Array(values) => values
                .iter()
                .any(|nested| forbidden_request_value("value", nested, depth + 1)),
            _ => false,
        }
}

fn forbidden_response_value(key: &str, value: &Value, depth: usize) -> bool {
    sensitive_key(key)
        || depth > 16
        || match value {
            Value::Object(values) => values.iter().any(|(nested_key, nested)| {
                forbidden_response_value(nested_key, nested, depth + 1)
            }),
            Value::Array(values) => values
                .iter()
                .any(|nested| forbidden_response_value("value", nested, depth + 1)),
            _ => false,
        }
}

fn forbidden_key(value: &str) -> bool {
    sensitive_key(value)
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "endpoint" | "headers" | "host" | "method" | "url"
        )
}

fn sensitive_key(value: &str) -> bool {
    let normalized = value
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric())
        .map(|byte| byte.to_ascii_lowercase())
        .collect::<Vec<_>>();
    matches!(
        normalized.as_slice(),
        b"authorization"
            | b"bearer"
            | b"cookie"
            | b"credential"
            | b"password"
            | b"secret"
            | b"setcookie"
            | b"token"
            | b"accesstoken"
            | b"refreshtoken"
            | b"clientsecret"
            | b"apikey"
    ) || normalized.ends_with(b"password")
        || normalized.ends_with(b"secret")
        || normalized.ends_with(b"token")
}
