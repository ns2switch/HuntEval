# v0.7.2 implementation evidence

## Status

v0.7.2 has an implemented local worker and offline safety foundation but is not release-complete. Live support and production-scored mode are not enabled.

## Implemented behavior

- infrastructure-only `hunteval-commercial` crate with no dependency on the HuntEval domain, evaluation, scoring, reporting, or runner crates;
- typed platforms and finite read-only operation catalogs for CrowdStrike Falcon, Google SecOps, Microsoft Sentinel, Elastic Security, and Cortex XSIAM;
- exact HTTPS origin policy, IP-literal rejection, connection-time public-address validation, opaque secret references, one-way secret-reference identities, and request/response limits;
- rejection of private, loopback, link-local, metadata, multicast, documentation, unique-local, and mapped private destinations;
- request contracts without URL, method, header, cookie, password, token, or secret fields;
- recursive rejection of transport or secret fields in Python and Rust boundaries;
- fixture mode without credentials and live-read-only mode requiring an opaque secret reference;
- content-addressed synthetic request/response fixtures and deterministic network-free replay;
- a fail-closed recording sanitizer with versioned policy identity, explicit field inventories, finite safe literals, deterministic synthetic replacements, and post-sanitization fixture validation;
- normalized results that retain platform, operation, pseudonymous tenant, region, mode, request hash, response hash, truncation, and availability;
- dedicated local and GitHub Actions `Commercial connector replay` gate.
- a request/action-correlated Managed Tool Gateway that returns typed safe reason codes;
- platform-specific finite method/path descriptors and bounded request builders for the five planned platforms;
- bounded vendor response normalization with explicit truncation and pagination availability;
- a fixed zero-retry, single-page transport policy that never follows remote pagination tokens implicitly;
- an HTTPS-only Rustls transport with pinned validated DNS results, no proxy, no redirects, identity encoding, response bounds, and secret-reflection rejection;
- a one-command worker binary with a separate bearer frame, one-shot secret resolution, owned-buffer zeroization, and no ambient environment contract;
- an offline negative live-harness test proving typed failure and canary absence from stdout, stderr, and the public attestation;
- a protected live conformance harness that supervises a bounded process group, kills the worker process tree on timeout, caps output files, writes only bounded pseudonymous attestations, and requires a worker-bound external egress-enforcement declaration;
- a manual platform-scoped GitHub Actions workflow restricted to protected environments and externally controlled `hunteval-commercial-egress` runners.

## Local acceptance evidence

The following passed on the current worktree:

```text
cargo test -p hunteval-commercial
cargo clippy -p hunteval-commercial --all-targets --all-features -- -D warnings
./scripts/ci/v072-commercial-connectors.sh
./scripts/ci/quality.sh all
```

Twenty-one focused Rust tests and six Python fixture, sanitizer, and replay tests cover policy, gateway correlation, vendor mappings, request construction, normalization, sensitive-field rejection, secret handling, worker framing, and the offline boundary. The live harness adds a network-free failure-path test.

## Open release evidence

- external host egress enforcement around the implemented worker;
- platform-native authentication/token issuance rather than injection of an already issued short-lived bearer;
- authorized live-read-only conformance for CrowdStrike Falcon and Google SecOps;
- authorized live-read-only conformance for at least one of Sentinel, Elastic, or Cortex;
- configured protected environments and runner-side egress controls, passing live workflow evidence, and protected-branch requirement on the exact closure revision.

These items require external enforcement, non-production tenants, least-privilege credentials, and additional integration evidence. Offline fixtures cannot substitute for that evidence. Production scored SIEM execution and every mutation remain unavailable.
