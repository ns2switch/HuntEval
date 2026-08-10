# v0.7.2 commercial platform connector preview implementation plan

## 1. Purpose and release position

This document defines the governed implementation sequence for v0.7.2 commercial security-platform connectors. It follows the completed R7 release and the in-progress v0.7.1 framework connector pack. It must finish before v0.8 decides which interfaces are stable enough to freeze for v1.0.

The user-visible outcome is a secure, auditable way to evaluate agent deployments against authorized read-only observations from commercial threat-hunting platforms while retaining HuntEval ownership of protocol enforcement, budgets, provenance, scoring, and public artifacts.

Initial platform priority is:

1. CrowdStrike Falcon;
2. Google Security Operations;
3. Microsoft Sentinel, with Security Copilot integration only where a documented API exposes auditable outputs;
4. Elastic Security;
5. Palo Alto Networks Cortex XSIAM or Cortex AgentiX, limited to documented read-only APIs and exportable observable activity.

v0.7.2 does not authorize scored execution against a production SIEM. Live tests use explicitly authorized non-production tenants, are read-only, opt-in, and separate from deterministic scoring. Production SIEM scored execution remains deferred until after v1.0.

## 2. Delivery status

Status is evidence-based. `planned` makes no implementation claim. `in progress` means only part of the milestone behavior or governance exists. `implemented` requires focused local behavior and tests. `complete` additionally requires offline conformance, authorized live evidence where required, canonical gates, documentation, and passing GitHub Actions on the exact closure revision.

| Milestone | Status | Outcome |
|---|---|---|
| P72-00 | in progress | finite read-only catalogs, network contracts, threat-model delta, and ADR-090 through ADR-097 exist; vendor feasibility and current-API review remain open |
| P72-01 | in progress | runner-owned deny-by-default policy contracts and address validation exist; the supervised live connector worker and host enforcement remain open |
| P72-02 | in progress | opaque secret references and public one-way identities exist; runtime resolution, authentication lifecycle, zeroization, canary redaction, and audit integration remain open |
| P72-03 | in progress | content-addressed synthetic fixtures and deterministic replay exist; recording sanitizer remains open |
| P72-04 | in progress | CrowdStrike offline operation catalog and replay exist; adapter, scopes, and live evidence remain open |
| P72-05 | in progress | Google SecOps offline operation catalog and replay exist; adapter, scopes, and live evidence remain open |
| P72-06 | in progress | Sentinel offline catalog and replay exist; connector and Security Copilot decision remain open |
| P72-07 | in progress | Elastic offline catalog and replay exist; connector and agent-output decision remain open |
| P72-08 | in progress | Cortex offline catalog and replay exist; connector and native-agent decision remain open |
| P72-09 | in progress | normalized offline result and replay CI exist; matrices and protected live workflow remain open |
| P72-10 | in progress | documentation, package inclusion, and local gates exist; migration, rollback, protected evidence, and release closure remain open |

No P72 milestone is complete. No network worker, secret-value handling, live connector, or production-scored mode exists. `implemented` and `in progress` describe local code only and are not live-support claims.

## 3. Operating modes

Every connector must declare exactly one operating mode per run:

| Mode | Network | Credentials | Scoring | Intended use |
|---|---|---|---|---|
| `fixture_replay` | denied | none | deterministic | required CI and benchmark development |
| `live_read_only` | allowlisted | secret references | non-scored | authorized sandbox conformance and data-shape validation |
| `production_scored` | unavailable | unavailable | unavailable | explicitly deferred until after v1.0 |

A fixture-replay result cannot be labeled live. A live result cannot be labeled reproducible unless it is bound to an immutable authorized snapshot or sanitized recording that passes offline verification. Reports must keep live platform availability, connector correctness, investigation quality, and platform-native agent quality separate.

## 4. Mandatory security boundaries

Every v0.7.2 change must:

- preserve Clean Architecture and keep HTTP clients, vendor SDKs, authentication libraries, and platform schemas outside domain, evaluation, scoring, and reporting cores;
- execute network access only in a supervised connector worker under runner-owned policy;
- deny network by default and require an exact connector, operation, origin, port, region, method, and budget authorization;
- allow HTTPS only for commercial connectors unless an accepted ADR documents a bounded test-only exception;
- verify certificates with supported trust roots and never add an insecure verification bypass;
- reject cross-origin redirects, credential forwarding across origins, unsupported proxies, and undeclared endpoints;
- block loopback, link-local, multicast, metadata-service, private, and otherwise non-public addresses unless a dedicated local test policy explicitly owns the destination;
- defend against DNS rebinding and revalidate resolved destinations at connection time;
- keep evaluated agents unable to construct arbitrary URLs, methods, headers, scopes, or raw HTTP bodies;
- resolve credentials from opaque runtime secret references and never persist secret values, access tokens, refresh tokens, client secrets, cookies, or authorization headers;
- use least-privilege read-only scopes and reject any operation whose effective permission is write-capable or unknown;
- bound connect, TLS, request, response, pagination, retry, concurrency, byte, row, time-range, and cost behavior;
- treat every remote response, error, document, agent output, and vendor message as untrusted input;
- preserve exact connector, operation, tenant alias, region, API version, request hash, response hash, pagination, timing, retry, and redaction provenance;
- keep tenant identifiers pseudonymous in public artifacts and reject raw customer names or account identifiers;
- prevent ground truth, hidden-test membership, evaluator diagnostics, prompts, secrets, and private paths from entering requests or recordings;
- record no private chain of thought from HuntEval deployments or platform-native agents;
- leave unsupported platform-native agent metrics unavailable rather than deriving them from summaries or UI text;
- keep all write, response, containment, remediation, case mutation, detection mutation, and policy mutation operations disabled.

## 5. Architecture and dependency direction

```text
evaluated deployment
  -> HuntEval managed-tool request
       -> runner policy and budget authorization
            -> supervised commercial connector worker
                 -> allowlisted HTTPS platform API
                      -> bounded untrusted response
                           -> normalized observation + exact provenance
                                -> deployment-visible tool result

Rust domain/evaluation/reporting
  <- no vendor SDK or HTTP dependency
```

Framework connectors from v0.7.1 remain deployment adapters. Commercial connectors are managed-tool adapters. Combining both does not grant a framework connector direct network access or allow it to bypass the managed-tool protocol.

Vendor-specific code must live in optional infrastructure packages or separate worker binaries. The generic worker owns transport safety and limits; a vendor adapter owns only documented operation construction and response normalization.

## 6. P72-00 — Contracts, feasibility, and ADRs

P72-00 must verify each platform against current official documentation and record:

- supported API versions and regions;
- authentication methods and minimum read-only scopes;
- documented query, detection, incident, entity, case, and agent-output operations;
- pagination, rate, time-range, response-size, and licensing limits;
- sandbox or development-tenant availability;
- whether platform-native agent actions and evidence are exportable through a documented API;
- prohibited write operations;
- fixture licensing and redistribution constraints.

The implementation review must accept or revise these proposed decisions:

- ADR-090: all commercial network access is runner-owned and out of process;
- ADR-091: credentials are opaque runtime references with redacted lifecycle events;
- ADR-092: connectors expose a finite versioned read-only operation catalog, not arbitrary HTTP;
- ADR-093: deterministic CI uses synthetic sanitized fixtures and content-addressed replay;
- ADR-094: live conformance uses protected environments and emits only bounded public attestations;
- ADR-095: remote response provenance is distinct from evidence asserted by an evaluated deployment;
- ADR-096: platform-native agents are evaluated only through documented observable exports;
- ADR-097: production SIEM scored execution and every mutation remain unavailable before v1.0.

P72-00 must select the next additive schema version if normative artifacts are required. Schema 0.3 through 0.9 and protocol 0.3 remain compatible and unchanged.

## 7. P72-01 — Network policy and worker

Add a versioned network capability policy resolved by the runner before process launch. The effective policy must bind:

- connector binary digest and manifest digest;
- connector kind and supported operation identifiers;
- exact allowed origins, ports, regions, methods, and redirect behavior;
- DNS and resolved-address policy;
- authentication mechanism and secret-reference names, never values;
- request, response, retry, concurrency, pagination, time-range, and wall-clock limits;
- public artifact and redaction policy;
- fixture-replay or live-read-only mode.

The worker must run with no filesystem access except explicitly mounted immutable configuration and a bounded output channel. It must not inherit the caller environment. Network namespace or equivalent host enforcement must be required for live mode; absence of the declared enforcement capability fails closed.

Tests must cover SSRF, encoded and alternate IP forms, DNS rebinding, cross-origin redirect, proxy variables, unsupported schemes, certificate failure, hostname mismatch, expired certificates, connection stalls, retry storms, decompression bombs, chunked oversized bodies, pagination loops, and process termination.

## 8. P72-02 — Authentication, secrets, and audit

Support only authentication mechanisms accepted in P72-00, such as workload identity, OAuth client credentials, short-lived access tokens, or mTLS. Static tokens may be supported only when a platform requires them and an accepted policy bounds their use.

Requirements:

- secret references are resolved after policy authorization and immediately before connector launch;
- secret material exists only in the worker's bounded runtime and is zeroized where supported;
- refresh and retry behavior cannot exceed the run deadline;
- errors expose typed reason codes without response bodies or credentials;
- logs redact authorization, cookies, tokens, tenant identifiers, query literals classified as sensitive, and vendor request identifiers when required;
- public audit events record the secret-reference identity hash, authentication kind, scope inventory, and success/failure state without secret values;
- fork pull requests and untrusted branches never receive live credentials.

Negative tests must use canary secrets and prove their absence from stdout, stderr, JSONL, reports, packages, crash dumps, and uploaded CI artifacts.

## 9. P72-03 — Recording, sanitization, and replay

Define content-addressed connector fixture bundles containing:

- normalized synthetic request metadata;
- bounded synthetic response bodies;
- API version, operation, pagination, region class, and status metadata;
- redaction manifest and validation result;
- request and response hashes;
- expected normalized observations and typed failures;
- fixture license and provenance.

Raw tenant recordings are evaluator-private and cannot become repository fixtures. A deterministic sanitizer must remove or replace customer identifiers, user data, hostnames, addresses, tokens, query literals, vendor request identifiers, and other sensitive values before review. Sanitization changes the content hash and requires revalidation.

Replay must make no DNS, socket, credential, clock, or provider call. Equivalent replay inputs must produce byte-identical normalized outputs and audit artifacts.

## 10. P72-04 and P72-05 — Required reference platforms

### P72-04 — CrowdStrike Falcon

The initial read-only operation catalog should cover documented capabilities for searching and retrieving detections or alerts, incidents, and relevant threat intelligence. Exact operations and scopes are frozen in P72-00.

The connector must reject detection updates, comments, assignment, containment, Real Time Response, policy changes, and every other mutation. Falcon Query Language input must be bounded and associated with an explicit time range and result limit.

Completion requires deterministic synthetic fixtures and a passing authorized live-read-only conformance run against a non-production tenant. Without tenant evidence, support remains `implemented`, not `complete` or `live-supported`.

### P72-05 — Google Security Operations

The initial read-only operation catalog should cover UDM query validation, bounded UDM search, and documented retrieval of events, entities, alerts, or cases selected in P72-00. Region, project alias, customer alias, time range, maximum events, pagination, and API version must be explicit.

The connector must reject feed changes, event ingestion, rule changes, case mutation, response actions, agent-setting changes, and natural-language query translation during deterministic scored evaluation. Generated query helpers may be tested separately but cannot silently alter a declared benchmark query.

Completion requires deterministic synthetic fixtures and a passing authorized live-read-only conformance run against a non-production tenant. Without tenant evidence, support remains `implemented`, not `complete` or `live-supported`.

## 11. P72-06 through P72-08 — Additional platforms

### P72-06 — Microsoft Sentinel and Security Copilot

- implement bounded read-only Sentinel hunting and retrieval operations selected in P72-00;
- keep KQL text, workspace alias, time range, and result limits explicit and auditable;
- evaluate Security Copilot or its Threat Hunting Agent only if a documented API exports action, evidence, and result identifiers required by HuntEval;
- otherwise record platform-native agent evaluation as unsupported without browser automation or UI scraping.

### P72-07 — Elastic Security

- implement bounded read-only search, alert, and investigation retrieval operations selected in P72-00;
- keep index/data-view aliases, ES|QL or other query language, time range, pagination, and limits explicit;
- evaluate AI Assistant, Agent Builder, Attack Discovery, or other agentic output only through documented export APIs;
- keep LLM-provider behavior and Elastic connector configuration outside HuntEval artifacts except for public content hashes and declared limitations.

### P72-08 — Cortex XSIAM/AgentiX

- implement only documented read-only case, alert, query, plan, action, or audit exports selected in P72-00;
- never execute playbooks, scripts, commands, sensitive actions, or response operations;
- require exported stable identifiers for agents, plans, actions, evidence, and terminal results before claiming native-agent evaluation support;
- return typed unsupported capabilities when licenses, regions, permissions, or public APIs do not expose the required observations.

v0.7.2 closure requires a passing live-read-only connector for at least one of P72-06 through P72-08 in addition to both required reference platforms. All three must have reviewed feasibility and explicit support status.

## 12. Normalized evidence and reporting

The normalized connector result must preserve:

- connector, platform, operation, API version, and policy identities;
- pseudonymous tenant and region class;
- request and response content hashes;
- query identity, declared time range, result count, truncation, pagination, and `more available` state;
- latency, retry, rate-limit, and observable cost provenance;
- normalized records with source-specific stable identifiers;
- unavailable fields and limitations;
- fixture-replay or live-read-only mode.

Vendor data must not be presented as HuntEval ground truth. Platform-native classifications and agent conclusions are observations with source provenance. Reports must distinguish platform result quality, connector correctness, deployment investigation quality, and topology behavior.

No global score or missing-value imputation is introduced. Raw metric vectors and constraint-first comparison remain authoritative.

## 13. Tests and conformance

Every connector must include:

- positive fixtures for every supported operation;
- negative authorization, scope, tenant, region, endpoint, and operation tests;
- malformed request, malformed response, unknown-field, invalid-encoding, and schema-drift tests;
- timeout, rate-limit, retry-after, partial-page, duplicate-page, and truncation tests;
- authentication failure and secret-redaction tests;
- deterministic replay and tamper-detection tests;
- worker crash, invalid output, and resource-exhaustion tests;
- ground-truth, hidden-test, private-field, and prompt-injection isolation tests;
- report escaping and offline verification tests;
- optional live conformance using synthetic tenant data.

Conformance artifacts must bind the exact connector binary, manifest, policy, fixture inventory, API version, operation inventory, test tenant alias, and result hashes. Passing conformance does not imply support for untested versions, regions, licenses, or operations.

## 14. CI and live workflow

Add two distinct workflows:

1. required offline `Commercial connector replay`, running on every trusted pull request without network or secrets;
2. protected `Commercial connector live conformance`, invoked manually or on a controlled schedule with environment approval and least-privilege credentials.

The offline canonical gate is `scripts/ci/v072-commercial-connectors.sh`. Before closure, run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
./scripts/ci/security.sh
./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/v072-commercial-connectors.sh
./scripts/ci/e2e.sh
```

The live workflow must not run for forks, must use protected environments, must not persist raw responses, and must upload only bounded conformance attestations that pass secret scanning. A live failure cannot be converted into a passing fixture result.

## 15. Pull-request sequence

1. PR-P72-00: feasibility matrix, threat model, contracts, schemas, ADRs, and synthetic fixtures.
2. PR-P72-01: network policy, supervised worker, host enforcement, and adversarial network tests.
3. PR-P72-02: secret references, authentication, redaction, and audit artifacts.
4. PR-P72-03: recording sanitizer, fixture bundle, replay, and offline verifier.
5. PR-P72-04: CrowdStrike Falcon connector and live conformance.
6. PR-P72-05: Google SecOps connector and live conformance.
7. PR-P72-06: Microsoft Sentinel connector and Security Copilot decision.
8. PR-P72-07: Elastic Security connector and agent-output decision.
9. PR-P72-08: Cortex XSIAM/AgentiX connector or typed unsupported results.
10. PR-P72-09: normalized cross-platform reporting, offline CI, protected live workflow, and additional-platform live evidence.
11. PR-P72-10: documentation, package, migration, rollback, limitations, and exact closure evidence.

Vendor connector work may proceed in parallel only after P72-00 through P72-03 freeze the common network, secret, fixture, and result contracts.

## 16. Migration and rollback

v0.7.2 adds optional connectors and does not change existing offline benchmark behavior. Network remains disabled unless a manifest, runner policy, host capability, operating mode, and authorized secret inventory all agree.

Each connector can be disabled independently by removing its manifest from the allowed inventory. Rollback restores the prior connector worker and policy version; it does not rewrite stored request, response, replay, run, or report artifacts. Revoked credentials are handled outside artifacts and must not require repository history changes.

Changing a connector binary, operation catalog, endpoint inventory, redaction policy, fixture, or authentication policy invalidates previous conformance for the changed identity.

## 17. Known limitations

- commercial tenant access, licenses, regions, API quotas, and test data are external prerequisites;
- vendor APIs and agent features may change independently of HuntEval;
- live results are subject to remote availability and data drift;
- sanitized fixtures cannot prove production scale or tenant-specific authorization;
- platform-native agents may not expose sufficiently detailed public action and evidence APIs;
- UI automation, screen scraping, undocumented endpoints, and private APIs are prohibited;
- every mutation and production scored SIEM execution remain unavailable;
- support claims apply only to the exact conformance matrix.

## 18. Release exit criteria

v0.7.2 is complete only when:

- R7 remains complete and v0.7.1 has passed its release gate;
- network is denied by default and enforced outside the evaluated deployment;
- no arbitrary HTTP interface is exposed to an agent or framework connector;
- certificate, redirect, DNS, SSRF, proxy, timeout, retry, pagination, and size controls fail closed;
- secrets are referenced, least-privilege, redacted, and absent from every artifact and package;
- CrowdStrike Falcon and Google SecOps pass offline and authorized live-read-only conformance;
- at least one additional planned platform passes offline and authorized live-read-only conformance;
- every other planned platform has an explicit supported, preview, or unsupported result backed by evidence;
- fixture replay is deterministic and independently verifiable;
- platform-native agent results are based only on documented observable exports;
- unsupported metrics remain unavailable and no vendor result is treated as ground truth;
- all mutation operations and production-scored mode remain unavailable;
- all canonical local, offline CI, protected live, package, and protected-branch gates pass on the exact closure revisions;
- migration, rollback, support versions, external prerequisites, and limitations are documented;
- v0.8 has not yet frozen the new interfaces.

## 19. Upstream references

- CrowdStrike Falcon API reference: <https://developer.crowdstrike.com/api-reference/>
- CrowdStrike Falcon MCP detection tools: <https://developer.crowdstrike.com/falcon-mcp/modules/detections/>
- Google SecOps REST API: <https://docs.cloud.google.com/chronicle/docs/reference/rest>
- Google SecOps UDM Search MCP tool: <https://docs.cloud.google.com/chronicle/docs/reference/mcp/udm_search>
- Microsoft Sentinel REST API: <https://learn.microsoft.com/en-us/rest/api/securityinsights/>
- Microsoft Security Copilot agents: <https://learn.microsoft.com/en-us/copilot/security/security-copilot-application-card-agents>
- Elastic AI for Security: <https://www.elastic.co/docs/solutions/security/ai>
- Cortex AgentiX documentation: <https://docs-cortex.paloaltonetworks.com/r/Cortex-AgentiX/Cortex-AgentiX-Documentation>

These references describe vendor capabilities only. HuntEval's accepted support matrix, operation catalogs, policies, and conformance evidence remain authoritative.
