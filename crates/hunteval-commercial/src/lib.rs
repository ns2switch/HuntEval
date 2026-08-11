//! Fail-closed contracts for optional commercial security-platform connectors.
#![forbid(unsafe_code)]

mod catalog;
mod gateway;
mod http;
mod policy;
mod service;
mod vendor;
mod vendor_request;
mod vendor_validation;
mod worker;

pub use catalog::{CommercialOperation, CommercialPlatform};
pub use gateway::{CommercialGateway, GatewayRequest, GatewayResponse};
pub use http::{BearerSecret, HttpTransport, SecretResolver};
pub use policy::{CommercialMode, CommercialPolicy, ResolvedAddress, SecretReference};
pub use service::{
    CommercialError, CommercialRequest, CommercialResponse, CommercialService, ReadOnlyTransport,
};
pub use vendor::{
    HttpMethod, OperationDescriptor, normalize_vendor_response, operation_descriptor,
};
pub use vendor_request::{PreparedVendorRequest, VendorTarget, prepare_vendor_request};

#[doc(hidden)]
pub use worker::{CommercialWorkerCommand, CommercialWorkerResponse, execute_worker_command};
