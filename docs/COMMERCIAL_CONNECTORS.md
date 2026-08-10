# Commercial connector previews

HuntEval currently implements only deterministic offline replay for commercial security-platform connector previews. No connector performs network access, resolves credentials, modifies remote state, or executes a scored benchmark against a production SIEM.

The infrastructure-only `hunteval-commercial` Rust crate additionally implements the fail-closed boundary required by a future live worker: finite typed operations, exact HTTPS origins, public resolved-address checks, opaque secret references, operating modes, request budgets, and response limits. Network execution remains unavailable because no production transport implements that boundary yet.

## Implemented offline catalogs

| Platform | Read-only operation families | Current status |
|---|---|---|
| CrowdStrike Falcon | detection search/retrieval, incident retrieval, threat-intelligence search | offline fixture replay implemented |
| Google Security Operations | UDM validation/search, event, alert, and case retrieval | offline fixture replay implemented |
| Microsoft Sentinel | hunting query, incident, alert, and entity retrieval | offline fixture replay implemented |
| Elastic Security | security search, alert and investigation retrieval | offline fixture replay implemented |
| Cortex XSIAM | alert, incident, query, and audit retrieval | offline fixture replay implemented |

The catalogs are finite allowlists. Mutation names, unknown operations, caller-provided URLs, endpoints, methods, headers, authorization values, tokens, passwords, and secret fields fail closed.

## Fixture identity

`CommercialRequest` binds:

- platform and operation;
- pseudonymous tenant alias and region;
- bounded JSON arguments.

`CommercialFixture` binds the exact request digest and response digest. `FixtureReplayConnector` returns a fixture only when the request SHA-256 identity matches exactly. Changed arguments require a new reviewed fixture and cannot reuse previous conformance.

Remote data remains an untrusted observation. A fixture classification is never HuntEval ground truth, and unsupported fields remain unavailable.

## Deliberately unavailable

- live HTTP transport;
- credential or secret-reference resolution;
- production tenants;
- production scored execution;
- containment, response, remediation, case mutation, detection mutation, and policy mutation;
- platform-native agent attribution without documented observable exports.

Live-read-only support requires the separately planned supervised network worker, host enforcement, authentication policy, secret redaction, protected CI environment, and authorized non-production tenant evidence. Offline replay does not satisfy those release gates.

## Local verification

```bash
./scripts/ci/v072-commercial-connectors.sh
```

This gate performs no DNS, socket, credential, provider, or tenant access.
