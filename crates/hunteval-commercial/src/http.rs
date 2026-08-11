use std::{
    fmt,
    io::Read,
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::Mutex,
    time::Duration,
};

use reqwest::{
    blocking::Client,
    header::{ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH},
    redirect::Policy,
};
use zeroize::Zeroizing;

use crate::{
    CommercialError, CommercialPolicy, CommercialRequest, CommercialResponse, HttpMethod,
    ReadOnlyTransport, ResolvedAddress, SecretReference, VendorTarget, normalize_vendor_response,
    prepare_vendor_request,
};

const MAX_SECRET_BYTES: usize = 64 * 1024;
const HTTPS_PORT: u16 = 443;

/// Short-lived bearer material that zeroizes its owned buffer on drop.
pub struct BearerSecret(Zeroizing<String>);

impl BearerSecret {
    pub fn new(value: String) -> Result<Self, CommercialError> {
        if value.len() < 8 || value.len() > MAX_SECRET_BYTES || value.contains(['\0', '\r', '\n']) {
            return Err(CommercialError::InvalidSecretReference);
        }
        Ok(Self(Zeroizing::new(value)))
    }

    fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for BearerSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerSecret([REDACTED])")
    }
}

/// Runtime-only secret broker. Implementations must not log or persist values.
pub trait SecretResolver: Send + Sync {
    fn resolve(&self, reference: &SecretReference) -> Result<BearerSecret, CommercialError>;
}

/// HTTPS-only transport with no proxy, redirects, ambient credentials, or DNS re-resolution.
#[derive(Debug)]
pub struct HttpTransport<R> {
    target: VendorTarget,
    secrets: R,
    pinned: Mutex<Option<PinnedResolution>>,
}

#[derive(Debug, Clone)]
struct PinnedResolution {
    origin: String,
    host: String,
    addresses: Vec<SocketAddr>,
}

impl<R> HttpTransport<R> {
    #[must_use]
    pub fn new(target: VendorTarget, secrets: R) -> Self {
        Self {
            target,
            secrets,
            pinned: Mutex::new(None),
        }
    }
}

impl<R: SecretResolver> ReadOnlyTransport for HttpTransport<R> {
    fn resolve(&self, origin: &str) -> Result<Vec<ResolvedAddress>, CommercialError> {
        let host = origin_host(origin)?;
        let addresses = (host.as_str(), HTTPS_PORT)
            .to_socket_addrs()
            .map_err(|_| CommercialError::TransportFailure)?
            .collect::<Vec<_>>();
        if addresses.is_empty() || addresses.len() > 32 {
            return Err(CommercialError::DeniedAddress);
        }
        let mut unique = addresses;
        unique.sort_unstable();
        unique.dedup();
        let result = unique
            .iter()
            .map(|address| ResolvedAddress(address.ip()))
            .collect::<Vec<_>>();
        result.iter().try_for_each(|address| address.validate())?;
        let mut pinned = self
            .pinned
            .lock()
            .map_err(|_| CommercialError::TransportFailure)?;
        *pinned = Some(PinnedResolution {
            origin: origin.to_owned(),
            host,
            addresses: unique,
        });
        Ok(result)
    }

    fn execute(
        &self,
        policy: &CommercialPolicy,
        request: &CommercialRequest,
    ) -> Result<CommercialResponse, CommercialError> {
        let pinned = self
            .pinned
            .lock()
            .map_err(|_| CommercialError::TransportFailure)?
            .take()
            .ok_or(CommercialError::DeniedAddress)?;
        if pinned.origin != policy.origin {
            return Err(CommercialError::DeniedAddress);
        }
        pinned
            .addresses
            .iter()
            .try_for_each(|address| ResolvedAddress(address.ip()).validate())?;
        let secret_reference = policy
            .secret_reference
            .as_ref()
            .ok_or(CommercialError::InvalidPolicy)?;
        let secret = self.secrets.resolve(secret_reference)?;
        let prepared = prepare_vendor_request(
            request.platform,
            request.operation,
            &request.arguments,
            &self.target,
            policy.max_records,
        )?;
        let client = secure_client(policy, &pinned)?;
        let url = format!("{}{}", policy.origin, prepared.relative_path);
        let builder = match prepared.method {
            HttpMethod::Get => client.get(url),
            HttpMethod::Post => client.post(url),
        }
        .header(ACCEPT, "application/json")
        .header(ACCEPT_ENCODING, "identity")
        .bearer_auth(secret.expose())
        .query(&prepared.query);
        let builder = match prepared.body {
            Some(body) => builder.json(&body),
            None => builder,
        };
        let mut response = builder
            .send()
            .map_err(|_| CommercialError::TransportFailure)?;
        if !response.status().is_success() {
            return Err(CommercialError::TransportFailure);
        }
        validate_response_headers(&response, policy.max_response_bytes)?;
        let bytes = read_bounded(&mut response, policy.max_response_bytes)?;
        if !secret.expose().is_empty()
            && bytes
                .windows(secret.expose().len())
                .any(|window| window == secret.expose().as_bytes())
        {
            return Err(CommercialError::InvalidResponse);
        }
        let value = serde_json::from_slice(&bytes).map_err(|_| CommercialError::InvalidResponse)?;
        normalize_vendor_response(
            request.platform,
            request.operation,
            &value,
            policy.max_records as usize,
        )
    }
}

fn secure_client(
    policy: &CommercialPolicy,
    pinned: &PinnedResolution,
) -> Result<Client, CommercialError> {
    Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_millis(policy.timeout_ms.min(30_000)))
        .timeout(Duration::from_millis(policy.timeout_ms))
        .resolve_to_addrs(&pinned.host, &pinned.addresses)
        .build()
        .map_err(|_| CommercialError::TransportFailure)
}

fn validate_response_headers(
    response: &reqwest::blocking::Response,
    maximum_bytes: u64,
) -> Result<(), CommercialError> {
    if response
        .headers()
        .get(CONTENT_ENCODING)
        .is_some_and(|value| value.as_bytes() != b"identity")
    {
        return Err(CommercialError::InvalidResponse);
    }
    if response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|value| value > maximum_bytes)
    {
        return Err(CommercialError::InvalidResponse);
    }
    Ok(())
}

fn read_bounded(reader: &mut impl Read, maximum_bytes: u64) -> Result<Vec<u8>, CommercialError> {
    let read_limit = maximum_bytes
        .checked_add(1)
        .ok_or(CommercialError::InvalidResponse)?;
    let mut bytes = Vec::new();
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| CommercialError::TransportFailure)?;
    if bytes.len() as u64 > maximum_bytes {
        return Err(CommercialError::InvalidResponse);
    }
    Ok(bytes)
}

fn origin_host(origin: &str) -> Result<String, CommercialError> {
    let host = origin
        .strip_prefix("https://")
        .ok_or(CommercialError::InvalidOrigin)?;
    if host.is_empty() || host.contains(['/', '?', '#', '@', ':']) || host.parse::<IpAddr>().is_ok()
    {
        return Err(CommercialError::InvalidOrigin);
    }
    Ok(host.to_owned())
}
