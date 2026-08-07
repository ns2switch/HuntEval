use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use hunteval_domain::{DigestParseError, Sha256Digest};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub fn hash_file(path: &Path) -> Result<Sha256Digest, HashingError> {
    let mut file = File::open(path).map_err(HashingError::Open)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(HashingError::Read)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    format!("{:x}", hasher.finalize())
        .parse()
        .map_err(HashingError::Digest)
}

#[derive(Debug, Error)]
pub enum HashingError {
    #[error("could not open artifact: {0}")]
    Open(io::Error),
    #[error("could not read artifact: {0}")]
    Read(io::Error),
    #[error("computed digest could not be represented: {0}")]
    Digest(DigestParseError),
}
