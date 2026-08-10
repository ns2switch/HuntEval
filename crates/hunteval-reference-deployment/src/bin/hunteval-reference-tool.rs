use std::io::{self, Read};

use hunteval_domain::{ManagedToolAdapterRequest, ManagedToolAdapterResponse, SchemaVersion};

const MAX_REQUEST_BYTES: u64 = 1_048_576;

fn main() {
    if execute().is_err() {
        std::process::exit(1);
    }
}

fn execute() -> Result<(), ()> {
    let mut bytes = Vec::new();
    io::stdin()
        .lock()
        .take(MAX_REQUEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    if bytes.len() as u64 > MAX_REQUEST_BYTES {
        return Err(());
    }
    let request: ManagedToolAdapterRequest = serde_json::from_slice(&bytes).map_err(|_| ())?;
    request.validate().map_err(|_| ())?;
    let response = ManagedToolAdapterResponse::Success {
        schema_version: SchemaVersion::new(0, 9),
        request_id: request.request_id,
        result: serde_json::json!({"accepted_tool":request.tool}),
    };
    response.validate().map_err(|_| ())?;
    serde_json::to_writer(io::stdout().lock(), &response).map_err(|_| ())
}
