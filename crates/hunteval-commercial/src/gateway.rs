use serde::{Deserialize, Serialize};

use crate::{
    CommercialError, CommercialRequest, CommercialResponse, CommercialService, ReadOnlyTransport,
};

const MAX_ID_BYTES: usize = 128;

/// Agent-visible request envelope without transport or credential authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GatewayRequest {
    pub request_id: String,
    pub agent_id: String,
    pub task_id: String,
    pub action_id: String,
    pub request: CommercialRequest,
}

impl GatewayRequest {
    fn validate(&self) -> Result<(), CommercialError> {
        if [
            &self.request_id,
            &self.agent_id,
            &self.task_id,
            &self.action_id,
        ]
        .into_iter()
        .all(|value| valid_id(value))
        {
            Ok(())
        } else {
            Err(CommercialError::InvalidRequest)
        }
    }
}

/// Correlated bounded result returned to the runner-managed tool path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum GatewayResponse {
    Success {
        request_id: String,
        action_id: String,
        result: CommercialResponse,
    },
    Error {
        request_id: String,
        action_id: String,
        reason_code: String,
    },
}

/// Correlation-preserving gateway over one runner-authorized commercial service.
#[derive(Debug)]
pub struct CommercialGateway<T> {
    service: CommercialService<T>,
}

impl<T: ReadOnlyTransport> CommercialGateway<T> {
    #[must_use]
    pub fn new(service: CommercialService<T>) -> Self {
        Self { service }
    }

    /// Execute one request and return a safe typed outcome without leaking error detail.
    pub fn execute(&mut self, request: &GatewayRequest) -> GatewayResponse {
        let result = request
            .validate()
            .and_then(|()| self.service.execute(&request.request));
        match result {
            Ok(result) => GatewayResponse::Success {
                request_id: request.request_id.clone(),
                action_id: request.action_id.clone(),
                result,
            },
            Err(error) => GatewayResponse::Error {
                request_id: request.request_id.clone(),
                action_id: request.action_id.clone(),
                reason_code: error.reason_code().to_owned(),
            },
        }
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_BYTES
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && b"._:-".contains(&byte))
        })
}
