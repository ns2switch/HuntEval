use std::io::{self, Read};

use hunteval_commercial::{
    BearerSecret, CommercialWorkerCommand, CommercialWorkerResponse, execute_worker_command,
};
use zeroize::Zeroizing;

const MAX_COMMAND_BYTES: usize = 1_048_576;
const MAX_SECRET_BYTES: usize = 64 * 1024;
const MAX_INPUT_BYTES: u64 = (MAX_COMMAND_BYTES + MAX_SECRET_BYTES + 1) as u64;

fn main() {
    let response = read_input()
        .map(|(command, secret)| execute_worker_command(command, secret))
        .unwrap_or_else(|reason_code| CommercialWorkerResponse::Failure { reason_code });
    if serde_json::to_writer(io::stdout().lock(), &response).is_err() {
        std::process::exit(1);
    }
}

fn read_input() -> Result<(CommercialWorkerCommand, BearerSecret), String> {
    let mut input = Zeroizing::new(Vec::new());
    io::stdin()
        .lock()
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut input)
        .map_err(|_| "worker_protocol".to_owned())?;
    if input.len() as u64 > MAX_INPUT_BYTES {
        return Err("worker_input_limit".to_owned());
    }
    let separator = input
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or_else(|| "worker_protocol".to_owned())?;
    if separator == 0
        || separator > MAX_COMMAND_BYTES
        || input.len().saturating_sub(separator + 1) > MAX_SECRET_BYTES
    {
        return Err("worker_input_limit".to_owned());
    }
    let command =
        serde_json::from_slice(&input[..separator]).map_err(|_| "worker_protocol".to_owned())?;
    let secret = String::from_utf8(input[separator + 1..].to_vec())
        .map_err(|_| "worker_protocol".to_owned())?;
    let secret = BearerSecret::new(secret).map_err(|error| error.reason_code().to_owned())?;
    Ok((command, secret))
}
