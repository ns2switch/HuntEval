use std::{collections::BTreeMap, sync::Mutex};

use serde::{Deserialize, Serialize};

use crate::{
    BearerSecret, CommercialError, CommercialGateway, CommercialMode, CommercialPolicy,
    CommercialService, GatewayRequest, GatewayResponse, HttpTransport, SecretReference,
    SecretResolver, VendorTarget,
};

/// Secret-free live worker command. Bearer material travels as a separate bounded frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommercialWorkerCommand {
    pub policy: CommercialPolicy,
    pub target: BTreeMap<String, String>,
    pub request: GatewayRequest,
}

/// Safe worker result written to the bounded stdout channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommercialWorkerResponse {
    Completed { response: GatewayResponse },
    Failure { reason_code: String },
}

/// Execute one live command with one separately framed short-lived bearer secret.
#[must_use]
pub fn execute_worker_command(
    command: CommercialWorkerCommand,
    secret: BearerSecret,
) -> CommercialWorkerResponse {
    match execute(command, secret) {
        Ok(response) => CommercialWorkerResponse::Completed { response },
        Err(error) => CommercialWorkerResponse::Failure {
            reason_code: error.reason_code().to_owned(),
        },
    }
}

fn execute(
    command: CommercialWorkerCommand,
    secret: BearerSecret,
) -> Result<GatewayResponse, CommercialError> {
    if command.policy.mode != CommercialMode::LiveReadOnly
        || command.policy.platform != command.request.request.platform
    {
        return Err(CommercialError::InvalidPolicy);
    }
    command.policy.validate()?;
    let target = VendorTarget::new(command.target)?;
    let resolver = OneShotSecretResolver {
        expected_reference: command
            .policy
            .secret_reference
            .clone()
            .ok_or(CommercialError::InvalidPolicy)?,
        value: Mutex::new(Some(secret)),
    };
    let transport = HttpTransport::new(target, resolver);
    let service = CommercialService::new(command.policy, transport)?;
    Ok(CommercialGateway::new(service).execute(&command.request))
}

struct OneShotSecretResolver {
    expected_reference: SecretReference,
    value: Mutex<Option<BearerSecret>>,
}

impl std::fmt::Debug for OneShotSecretResolver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OneShotSecretResolver")
            .field("expected_reference", &"[OPAQUE]")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

impl SecretResolver for OneShotSecretResolver {
    fn resolve(&self, reference: &SecretReference) -> Result<BearerSecret, CommercialError> {
        if reference != &self.expected_reference {
            return Err(CommercialError::InvalidSecretReference);
        }
        self.value
            .lock()
            .map_err(|_| CommercialError::InvalidSecretReference)?
            .take()
            .ok_or(CommercialError::InvalidSecretReference)
    }
}
