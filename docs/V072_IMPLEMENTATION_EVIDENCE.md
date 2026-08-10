# v0.7.2 implementation evidence

## Status

v0.7.2 has a complete offline safety foundation but is not release-complete. No live connector or production-scored mode is enabled.

## Implemented behavior

- infrastructure-only `hunteval-commercial` crate with no dependency on the HuntEval domain, evaluation, scoring, reporting, or runner crates;
- typed platforms and finite read-only operation catalogs for CrowdStrike Falcon, Google SecOps, Microsoft Sentinel, Elastic Security, and Cortex XSIAM;
- exact HTTPS origin policy, IP-literal rejection, connection-time public-address validation, opaque secret references, one-way secret-reference identities, and request/response limits;
- rejection of private, loopback, link-local, metadata, multicast, documentation, unique-local, and mapped private destinations;
- request contracts without URL, method, header, cookie, password, token, or secret fields;
- recursive rejection of transport or secret fields in Python and Rust boundaries;
- fixture mode without credentials and live-read-only mode requiring an opaque secret reference;
- content-addressed synthetic request/response fixtures and deterministic network-free replay;
- normalized results that retain platform, operation, pseudonymous tenant, region, mode, request hash, response hash, truncation, and availability;
- dedicated local and GitHub Actions `Commercial connector replay` gate.

## Local acceptance evidence

The following passed on the current worktree:

```text
cargo test -p hunteval-commercial
cargo clippy -p hunteval-commercial --all-targets --all-features -- -D warnings
./scripts/ci/v072-commercial-connectors.sh
./scripts/ci/quality.sh all
```

Five Rust policy tests and four Python fixture-replay tests cover the current offline boundary.

## Open release evidence

- a supervised live transport worker with TLS, redirect, proxy, DNS-rebinding, retry, pagination, decompression, cancellation, and process-tree enforcement;
- runtime secret resolution, zeroization where supported, redaction canaries, and protected-environment integration;
- documented vendor request builders and response normalizers against current public APIs;
- authorized live-read-only conformance for CrowdStrike Falcon and Google SecOps;
- authorized live-read-only conformance for at least one of Sentinel, Elastic, or Cortex;
- protected live workflow evidence and protected-branch requirement on the exact closure revision.

These items require additional implementation plus external non-production tenants and credentials. Offline fixtures cannot substitute for that evidence. Production scored SIEM execution and every mutation remain unavailable.
