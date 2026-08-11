# Commercial connector previews

HuntEval provides deterministic offline replay plus a disabled-by-default live-read-only worker foundation for commercial security-platform connector previews. No live connector is release-certified, no connector modifies remote state, and production-scored SIEM execution remains unavailable.

The infrastructure-only `hunteval-commercial` Rust crate implements a correlated managed-tool gateway, finite typed vendor request builders, bounded response normalization, exact HTTPS origins, public resolved-address checks, opaque secret references, operating modes, request budgets, response limits, and a one-call worker protocol. The HTTPS transport pins validated DNS results, disables proxies and redirects, requires Rustls certificate verification, requests identity encoding, bounds response bytes, and rejects a response containing the active bearer secret.

The preview transport performs exactly one HTTP attempt and consumes at most one response page. It reports `more_available` and truncation explicitly but never follows a vendor pagination token automatically. This zero-retry, single-page policy is the current fail-closed retry and pagination control; multi-page retrieval requires a future reviewed policy change.

## Implemented offline catalogs

| Platform | Read-only operation families | Current status |
|---|---|---|
| CrowdStrike Falcon | detection search/retrieval, incident retrieval, threat-intelligence search | offline fixture replay implemented |
| Google Security Operations | UDM validation/search, event, alert, and case retrieval | offline fixture replay implemented |
| Microsoft Sentinel | hunting query, incident, alert, and entity retrieval | offline fixture replay implemented |
| Elastic Security | security search, alert and investigation retrieval | offline fixture replay implemented |
| Cortex XSIAM | alert, incident, query, and audit retrieval | offline fixture replay implemented |

The catalogs are finite allowlists. Mutation names, unknown operations, caller-provided URLs, endpoints, methods, headers, authorization values, tokens, passwords, and secret fields fail closed.

## Managed Tool Gateway and live worker boundary

`CommercialGateway` correlates every result with the original `request_id` and `action_id`. Agents provide only a platform, finite operation, pseudonymous tenant/region, and operation-specific bounded arguments. Trusted vendor code selects the relative path, HTTP method, query/body shape, and response collection.

`hunteval-commercial-worker` accepts one secret-free JSON command followed by one separately framed short-lived bearer value. The worker clears ambient environment use at its supervised caller boundary, resolves the opaque reference once, zeroizes the owned bearer buffer, returns only typed safe errors, and never places the secret in its serialized contracts.

`scripts/ci/v072-live-conformance.sh` is the protected live harness. It requires an exact worker-bound external egress-enforcement attestation, a mode-`0600` secret file, a secret-free command, and a new output path. The worker runs in a dedicated process group with a strict timeout and file-size bounds; timeout kills the complete process tree. Raw vendor responses remain private and only a bounded pseudonymous conformance attestation is written. A failed live request makes the gate fail even though its typed failure attestation is retained.

The manual `Commercial live conformance` workflow selects a platform-specific protected GitHub environment and runs only on a self-hosted runner carrying the `hunteval-commercial-egress` label. That label is an operational assertion, not proof created by repository code: an administrator must independently verify the runner's host firewall, network namespace, or egress proxy before approving the environment. The environment supplies the exact command, short-lived bearer, and worker-bound enforcement declaration. Only a passing public attestation is uploaded.

This repository cannot establish the declared host firewall, network namespace, or egress proxy by itself. Live execution must remain disabled until the protected environment supplies that enforcement and an authorized non-production tenant.

## Fixture identity

`CommercialRequest` binds:

- platform and operation;
- pseudonymous tenant alias and region;
- bounded JSON arguments.

`CommercialFixture` binds the exact request digest and response digest. `FixtureReplayConnector` returns a fixture only when the request SHA-256 identity matches exactly. Changed arguments require a new reviewed fixture and cannot reuse previous conformance.

Remote data remains an untrusted observation. A fixture classification is never HuntEval ground truth, and unsupported fields remain unavailable.

## Recording sanitization

`sanitize_recording` converts a reviewed private response recording into a synthetic `CommercialFixture`. Its `RecordingSanitizationPolicy` is content-addressed and declares every permitted field plus a finite set of safe classification literals. Undeclared fields, credential-like field names, non-finite values, unsupported envelopes, excessive nesting, and oversized collections fail closed. All other strings and numbers are replaced deterministically from the public policy identity and structural path; the private source value is not embedded in the replacement digest.

The sanitized response is passed through normal fixture validation and receives a new response hash. Raw recordings, customer identifiers, and private sanitization receipts remain evaluator-private and must never be committed. A sanitization pass proves only that the bounded transformation succeeded; it does not establish licensing approval or permission to redistribute a source recording.

## Deliberately unavailable or uncertified

- release-certified live platform support;
- an in-repository production secret manager;
- host egress enforcement supplied by an authorized execution environment;
- production tenants;
- production scored execution;
- containment, response, remediation, case mutation, detection mutation, and policy mutation;
- platform-native agent attribution without documented observable exports.

Live-read-only support still requires host enforcement, platform-specific least-privilege token issuance, a protected CI environment, and authorized non-production tenant evidence. The implemented worker and offline replay do not satisfy those external release gates by themselves.

## Local verification

```bash
./scripts/ci/v072-commercial-connectors.sh
```

This gate performs no DNS, socket, credential, provider, or tenant access.
