use serde::{Deserialize, Serialize};

use crate::{ContractValidationError, SchemaVersion};

use crate::extension::{bounded_id, require_version};

const MAX_ADAPTER_MESSAGE_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedToolAdapterRequest {
    pub schema_version: SchemaVersion,
    pub request_id: String,
    pub tool: String,
    pub arguments: serde_json::Value,
}

impl ManagedToolAdapterRequest {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        require_version(self.schema_version)?;
        bounded_id(&self.request_id, "managed_tool.request_id")?;
        bounded_id(&self.tool, "managed_tool.tool")?;
        validate_adapter_message(self, "managed_tool.request")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum ManagedToolAdapterResponse {
    Success {
        schema_version: SchemaVersion,
        request_id: String,
        result: serde_json::Value,
    },
    Error {
        schema_version: SchemaVersion,
        request_id: String,
        reason_code: String,
    },
}

impl ManagedToolAdapterResponse {
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        let (version, request_id) = match self {
            Self::Success {
                schema_version,
                request_id,
                ..
            }
            | Self::Error {
                schema_version,
                request_id,
                ..
            } => (*schema_version, request_id),
        };
        require_version(version)?;
        bounded_id(request_id, "managed_tool.request_id")?;
        if let Self::Error { reason_code, .. } = self {
            bounded_id(reason_code, "managed_tool.reason_code")?;
        }
        validate_adapter_message(self, "managed_tool.response")
    }

    #[must_use]
    pub fn request_id(&self) -> &str {
        match self {
            Self::Success { request_id, .. } | Self::Error { request_id, .. } => request_id,
        }
    }
}

fn validate_adapter_message(
    message: &impl Serialize,
    field: &'static str,
) -> Result<(), ContractValidationError> {
    let bytes = serde_json::to_vec(message).map_err(|_| {
        ContractValidationError::new(field, "managed-tool message cannot be serialized")
    })?;
    if bytes.len() > MAX_ADAPTER_MESSAGE_BYTES {
        return Err(ContractValidationError::new(
            field,
            "managed-tool message exceeds its byte limit",
        ));
    }
    Ok(())
}
