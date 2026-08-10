//! Fail-closed contracts for optional commercial security-platform connectors.
#![forbid(unsafe_code)]

mod catalog;
mod policy;
mod service;

pub use catalog::{CommercialOperation, CommercialPlatform};
pub use policy::{CommercialMode, CommercialPolicy, ResolvedAddress, SecretReference};
pub use service::{
    CommercialError, CommercialRequest, CommercialResponse, CommercialService, ReadOnlyTransport,
};
