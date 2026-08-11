# Pre-R8 connector implementation plan

## 1. Purpose and release position

This document coordinates the implementation of the connector capabilities planned between the completed R7 release and the R8 release candidate. It does not replace the normative detailed plans:

- `V071_FRAMEWORK_CONNECTOR_PLAN.md` governs native framework connectors and the MCP interoperability server;
- `V072_COMMERCIAL_CONNECTOR_PLAN.md` governs read-only commercial security-platform connectors.

The user-visible outcome is a coherent extension layer with three entry paths:

1. native connectors for explicitly supported agent frameworks;
2. a bounded Model Context Protocol (MCP) server for frameworks without a native connector;
3. runner-owned read-only connectors for selected commercial threat-hunting platforms.

R7 remains complete and unchanged. v0.7.1 and v0.7.2 are in progress and make no completion claim. On 2026-08-11, roadmap governance authorized R8-00 inventory and freeze-policy work while both additions remain pending; their interfaces are excluded from the stable freeze set. R8 closure still requires both additions to satisfy their release gates or roadmap governance to revise an unmet dependency explicitly.

### Delivery status

Status is evidence-based. `planned` makes no implementation claim. `implemented` requires focused behavior and tests. `complete` additionally requires every applicable release gate, package and documentation evidence, and passing GitHub Actions on the exact closure revision.

| Group | Status | Outcome |
|---|---|---|
| G0 | implemented | code boundaries, compatibility policy, threat models, exact candidate-version inventory, and accepted ADR-084 through ADR-097 exist |
| G1 | implemented | common framework lifecycle and CrewAI regression baseline pass focused tests |
| G2 | implemented | native structural adapters pass deterministic fixture conformance |
| G3 | implemented | bounded MCP session and fixed catalog pass focused adversarial tests |
| G4 | in progress | five adapters pass an equivalent deterministic supervisor-worker matrix; full scored benchmark evidence remains open |
| G5 | in progress | local connector CI and guides exist; release closure remains open |
| G6 | implemented | worker, HTTPS, network policy, secret framing, fixtures, and replay pass local adversarial tests; external egress certification remains open |
| G7 | implemented | finite vendor request and normalization adapters exist; authorized live conformance remains open |
| G8 | in progress | normalized replay CI and a protected live workflow exist; protected environments, live runs, and closure evidence remain open |
| G9 | planned | v0.7.2 release closure |

No G0 through G9 group is complete. Current evidence is local and offline.

## 2. Scope and release gates

### v0.7.1 — Framework connector pack

Required outcomes:

- preserve CrewAI as the compatibility baseline;
- implement native connectors for LangGraph, AutoGen AgentChat, and Google ADK/A2A;
- produce either a passing bounded Semantic Kernel preview or an explicit typed unsupported result;
- implement the local supervised MCP `stdio` interoperability server;
- demonstrate equivalent single-agent and declared multi-agent execution through the common lifecycle;
- produce paired cross-framework benchmark and topology-equivalence evidence;
- close packaging, documentation, compatibility, migration, rollback, and CI evidence.

### v0.7.2 — Commercial platform connector preview

Required outcomes:

- implement the runner-owned deny-by-default network policy and supervised connector worker;
- implement opaque secret references, authentication lifecycle, redaction, and audit provenance;
- implement content-addressed sanitized fixtures and deterministic offline replay;
- implement CrowdStrike Falcon and Google Security Operations read-only connectors;
- implement at least one live-read-only connector from Microsoft Sentinel, Elastic Security, or Cortex XSIAM/AgentiX;
- record explicit feasibility and support results for every other planned platform;
- produce normalized evidence, protected live conformance, documentation, migration, rollback, and CI evidence.

Production SIEM scored execution, unrestricted network access, mutations, response actions, UI automation, undocumented APIs, and autonomous policy changes remain unavailable.

## 3. Mandatory implementation principles

Every pull request in this plan must:

- preserve Clean Architecture and keep framework, MCP, vendor, HTTP, and authentication dependencies outside the domain, evaluation, scoring, reporting, and runner-policy cores;
- retain the existing HuntEval deployment protocol as the process-neutral authority;
- execute scored tools only through HuntEval-managed tools and runner-owned budgets;
- keep ground truth, hidden-test membership, evaluator diagnostics, scoring internals, secrets, and private paths unavailable to deployments and connectors;
- record only observable actions, messages, events, evidence, findings, structured decisions, and concise reason codes;
- never request or store private chain of thought, framework scratchpads, or hidden model/provider state;
- treat every framework event, MCP message, remote response, document, and agent-produced string as bounded untrusted input;
- preserve provenance from deployment and agent through task, action, tool result, evidence, finding, and final submission;
- leave missing or unsupported observations unavailable rather than inventing equivalent metrics;
- preserve the authoritative raw metric vector and existing scoring-profile and constraint-first ranking semantics;
- use explicit contract versions, deterministic serialization, stable opaque identifiers, content hashes, typed errors, and fail-closed validation;
- keep production Rust files cohesive and human-readable, split modules before they become difficult to review, and avoid production `unwrap()`, `expect()`, first-party `unsafe`, or panic shortcuts;
- update tests, documentation, ADRs, migration, rollback, and known limitations with the owning change.

## 4. Architecture and authority boundaries

```text
native framework
  -> optional framework connector ----------+
                                             |
unsupported framework                       v
  -> supervised MCP stdio server ------> HuntEval deployment protocol
                                             |
                                             v
                                  runner policy and budgets
                                             |
                                             v
                                      managed tools
                                             |
                       +---------------------+----------------------+
                       |                                            |
                 offline fixtures                      commercial worker
                                                                    |
                                                                    v
                                                    allowlisted read-only API

domain / evaluation / scoring / reporting
  <- no framework, MCP transport, vendor SDK, HTTP, or credential dependency
```

Native framework connectors and the MCP server are deployment adapters. Commercial connectors are managed-tool adapters. Neither path may become an alternate orchestration, authorization, scoring, or tool-execution authority.

## 5. Dependency order

```text
G0 governance and contract freeze
  -> G1 common framework lifecycle
       -> G2 native framework connectors --------+
       -> G3 MCP interoperability server ---------+-> G4 comparative evidence
                                                        -> G5 v0.7.1 closure
                                                             -> G6 commercial safety foundation
                                                                  -> G7 vendor connectors
                                                                       -> G8 conformance and reports
                                                                            -> G9 v0.7.2 closure
                                                                                 -> R8
```

G2 connector implementations may proceed in parallel after G1. G3 may proceed in parallel with G2 after its contract and threat-model decisions are accepted. Vendor implementations in G7 may proceed in parallel only after G6 freezes network, secret, operation-catalog, fixture, and normalized-result contracts.

## 6. G0 — Governance, contract inventory, and threat models

Map to `F71-00` and the planning portion of `P72-00`.

Actions:

1. inventory exact public upstream versions and APIs;
2. identify additive contract changes and select schema versions without changing schema 0.3 through 0.9 or protocol 0.3 artifacts;
3. accept or revise ADR-084 through ADR-097;
4. add threat-model deltas for framework callbacks, MCP clients, MCP tool metadata, remote services, credentials, tenant data, and network transports;
5. freeze status values, compatibility policy, removal policy, and support-matrix fields;
6. define canonical positive and malformed artifacts for every new normative contract.

Exit criteria:

- affected contracts, bounds, compatibility, security impact, and ground-truth-isolation impact are reviewable;
- every proposed ADR has an accepted or explicitly rejected disposition;
- no implementation depends on a private or undocumented upstream API;
- unsupported capabilities have typed outcomes rather than fallback inference.

## 7. G1 — Common framework lifecycle

Map to `F71-01` and `F71-02`.

Actions:

1. implement the framework-neutral Python lifecycle over the existing HuntEval protocol;
2. map declared topology, agents, roles, tasks, delegation, managed-tool correlation, evidence, findings, terminal state, and resource usage;
3. migrate CrewAI to this lifecycle while preserving its supported behavior;
4. create deterministic framework doubles and common conformance fixtures;
5. test malformed transitions, duplicate identities, correlation failures, process failures, bounds, cancellation, and replay.

Exit criteria:

- the base Python SDK imports and installs without a framework dependency;
- CrewAI normalized output remains compatible where contracts are unchanged;
- direct scored-tool execution and framework authority escalation fail closed;
- the common conformance suite is reusable without provider credentials or network access.

## 8. G2 — Native framework connectors

Map to `F71-03` through `F71-06`.

Delivery order:

1. LangGraph;
2. AutoGen AgentChat;
3. Google ADK/A2A with remote A2A disabled until separately authorized;
4. Semantic Kernel preview or an explicit unsupported decision.

Each connector must have an isolated optional dependency group, an exact supported-version matrix, deterministic framework fixtures, package-isolation tests, lifecycle and topology tests, malformed-input tests, process-failure tests, and documented unavailable observations.

Exit criteria:

- required connectors pass the common suite;
- no framework package is imported by the base SDK or linked into a Rust core crate;
- framework-specific differences and unavailable metrics are explicit;
- Semantic Kernel is labeled preview unless stable public APIs and conformance evidence justify a stronger status.

## 9. G3 — MCP interoperability server

Map to `F71-07`.

Actions:

1. implement an optional supervised MCP server over local `stdio`;
2. pin the supported MCP revision and implement strict lifecycle and capability negotiation;
3. expose only the finite HuntEval catalog required for deployment-visible observations, task and delegation events, runner-mediated tool calls, evidence, findings, resource declarations, terminal state, and final submission;
4. disable sampling, elicitation, roots, server-provided prompts, arbitrary resources, resource subscriptions, dynamic tool changes, custom transports, and remote Streamable HTTP;
5. bind every MCP session to one run and validate all deployment, agent, task, action, evidence, and finding identities;
6. create a language-neutral client harness representing a framework with no native HuntEval connector;
7. complete one canonical single-agent run and one declared multi-agent run through MCP;
8. add adversarial tests for JSON-RPC framing, version and capability attacks, identity replay, cross-run access, injection, oversized messages, floods, cancellation races, stdout contamination, stderr leakage, timeout, crash, and shutdown.

Exit criteria:

- MCP never bypasses the HuntEval deployment protocol, runner policy, managed-tool mediation, or budgets;
- generic MCP compatibility is reported separately from native framework support;
- unsupported MCP capabilities and transports fail closed;
- deterministic replay produces identical normalized public artifacts;
- no ground truth, secret, private path, private reasoning, or evaluator-only field is exposed.

## 10. G4 and G5 — Framework evidence and v0.7.1 closure

Map to `F71-08` and `F71-09`.

Actions:

1. run paired benchmark cells for CrewAI, LangGraph, AutoGen, and Google ADK using equivalent declared controls;
2. include Semantic Kernel only if its preview passes the same applicable requirements;
3. report generic MCP conformance as a distinct interoperability class;
4. separate framework overhead from investigation quality and preserve raw metric vectors;
5. publish exact connector, framework, SDK, MCP, protocol, schema, fixture, configuration, and binary identities;
6. publish the support matrix, package evidence, migration, rollback, limitations, and completion evidence;
7. add and protect the dedicated `Framework connectors` CI job.

v0.7.1 closes only when every exit criterion in `V071_FRAMEWORK_CONNECTOR_PLAN.md` passes on the exact closure revision.

## 11. G6 — Commercial connector safety foundation

Map to `P72-00` through `P72-03`.

Actions:

1. complete the vendor feasibility matrix and freeze a finite read-only operation catalog;
2. implement a supervised out-of-process commercial connector worker;
3. implement runner-owned network policy with exact origins, ports, methods, regions, operations, DNS behavior, limits, and connector identities;
4. enforce HTTPS, certificate validation, redirect policy, DNS rebinding protection, SSRF protection, proxy restrictions, and bounded retries, pagination, concurrency, time, rows, and bytes;
5. resolve credentials only from opaque runtime secret references after authorization;
6. implement secret canaries, redaction, least-privilege scope checks, audit provenance, and protected-environment rules;
7. implement deterministic sanitization, fixture bundles, tamper detection, and fully offline replay.

Exit criteria:

- network remains denied unless manifest, runner policy, host enforcement, operating mode, endpoint, operation, and secret inventory agree;
- evaluated deployments cannot create arbitrary URLs, methods, headers, bodies, scopes, or vendor operations;
- every write-capable or unknown operation is rejected;
- credentials and tenant-sensitive data are absent from public artifacts, packages, logs, and failures;
- offline replay performs no DNS, socket, credential, provider, or uncontrolled clock access.

## 12. G7 — Commercial platform connectors

Map to `P72-04` through `P72-08`.

Required order and evidence:

1. CrowdStrike Falcon: deterministic fixtures plus authorized non-production live-read-only conformance;
2. Google Security Operations: deterministic fixtures plus authorized non-production live-read-only conformance;
3. Microsoft Sentinel, Elastic Security, and Cortex XSIAM/AgentiX: feasibility and explicit support results for all three, with at least one passing live-read-only connector.

Platform-native agents are evaluated only when documented public APIs expose the stable actions, evidence, and results needed by HuntEval. UI scraping, browser automation, undocumented endpoints, guessed schemas, and synthetic attribution are prohibited.

Exit criteria:

- each supported operation has positive, negative, malformed-response, schema-drift, rate-limit, pagination, truncation, timeout, authentication, redaction, replay, and worker-failure coverage;
- support is limited to the exact tested platform version, region, license, operation, and permission matrix;
- platform classifications and conclusions remain source-provenanced observations, never HuntEval ground truth;
- live conformance emits only bounded public attestations and never stores raw tenant responses.

## 13. G8 and G9 — Commercial evidence and v0.7.2 closure

Map to `P72-09` and `P72-10`.

Actions:

1. normalize connector results and preserve request, response, connector, operation, policy, tenant-alias, region, pagination, timing, retry, truncation, and redaction provenance;
2. keep connector correctness, platform availability, investigation quality, topology behavior, and platform-native agent quality separate;
3. add required offline `Commercial connector replay` CI without secrets or network;
4. add protected `Commercial connector live conformance` with environment approval and least-privilege credentials;
5. publish conformance matrices, package evidence, external prerequisites, migration, rollback, limitations, and completion evidence;
6. verify the protected branch requires all new canonical jobs on the closure revision.

v0.7.2 closes only when every exit criterion in `V072_COMMERCIAL_CONNECTOR_PLAN.md` passes. Missing non-production tenant access leaves the affected connector implemented but not complete and blocks any release criterion that requires its live evidence.

## 14. Pull-request sequence

The detailed PR identifiers remain authoritative. The coordinated order is:

1. F71-00 and F71-01: contracts and common lifecycle;
2. F71-02: CrewAI regression baseline;
3. F71-03 through F71-07: native connectors and MCP, parallel where safe;
4. F71-08: comparative evidence and CI;
5. F71-09: v0.7.1 closure;
6. P72-00 through P72-03: commercial security foundation;
7. P72-04 through P72-08: vendor connectors, parallel where safe;
8. P72-09: normalized evidence and both CI workflows;
9. P72-10: v0.7.2 closure;
10. R8 closure occurs only after the exact completion evidence revisions are accepted or the dependency is revised explicitly through roadmap governance.

No pull request may combine a common contract change with multiple vendor implementations. Every connector must be independently removable and reviewable.

## 15. Required test matrix

| Area | Positive | Negative and malformed | Determinism and failure |
|---|---|---|---|
| common framework lifecycle | canonical single and multi-agent runs | identities, transitions, correlations, bounds | replay, timeout, crash, cancellation |
| native connector | supported topology and tool path | direct-tool bypass, unknown events, dependency isolation | golden fixtures, version matrix, package install |
| MCP | lifecycle, capability negotiation, canonical runs | capability escalation, cross-run access, injection, framing | replay, flood limits, timeout, crash, clean shutdown |
| network worker | authorized read-only operation | SSRF, DNS rebinding, redirects, TLS, proxy, mutation | retry limits, decompression limits, termination |
| secrets | authorized opaque reference | wrong scope, tenant, token canary, untrusted CI | redaction across logs, artifacts, packages, crashes |
| vendor adapter | supported fixture and live operation | schema drift, auth, rate, pagination, truncation | offline replay, tamper detection, worker failure |
| reporting | normalized source provenance | unsupported and unavailable fields | escaped deterministic JSON and HTML verification |

Every metric introduced by connector work must define its range, direction, denominator, edge cases, unavailable behavior, and focused tests before it enters reports.

## 16. Canonical quality gates

Every implementation pull request runs the applicable focused tests plus:

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
./scripts/ci/e2e.sh
```

After they exist, the following gates are also mandatory for their owning releases:

```bash
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/v072-commercial-connectors.sh
```

Release closure additionally requires documentation, package, offline verification, protected-branch, and passing GitHub Actions evidence for the exact candidate revision. Live commercial conformance remains a separate protected workflow and must never expose credentials or raw tenant data.

## 17. Required artifacts

v0.7.1 must produce:

- accepted ADRs and threat-model delta;
- framework and MCP compatibility matrices;
- common lifecycle and MCP normative fixtures;
- deterministic connector transcripts and normalized events;
- paired comparison and topology-equivalence artifacts;
- package inventories and dependency evidence;
- migration, rollback, limitations, and completion evidence.

v0.7.2 must produce:

- accepted ADRs and network threat-model delta;
- vendor feasibility, operation, scope, region, version, and support matrices;
- network and secret policy artifacts;
- sanitized content-addressed fixture bundles and replay evidence;
- bounded offline and live-read-only conformance attestations;
- normalized public connector results and reports;
- package inventories, external prerequisites, migration, rollback, limitations, and completion evidence.

## 18. Migration and rollback strategy

- Existing R2 through R7 schemas, protocol fixtures, artifacts, and completion evidence remain unchanged.
- CrewAI retains a documented compatibility window through the common-lifecycle migration.
- Each framework connector remains an optional package extra and can be disabled independently.
- The MCP server remains an optional adapter inventory entry and can be removed without changing stored runs.
- Commercial network access remains disabled by default; removing a vendor manifest disables that connector without changing offline execution.
- Rollback restores prior binaries, manifests, policies, and package versions. It never rewrites historical artifacts or changes their declared identities.
- Any change to a connector binary, MCP revision, operation catalog, endpoint inventory, fixture, authentication policy, or redaction policy invalidates the affected conformance result.

## 19. Known external prerequisites and blockers

- Framework API instability may require a preview or unsupported status for a connector.
- Remote A2A and remote MCP are unavailable until a separately accepted network and authentication policy supports them.
- CrowdStrike and Google SecOps closure require authorized non-production tenants and least-privilege read-only credentials.
- v0.7.2 additionally requires authorized live-read-only evidence for at least one of Sentinel, Elastic, or Cortex.
- Vendor licensing, regional API availability, quota, and export limitations may make platform-native agent evaluation unavailable.
- Hidden or UI-only platform activity cannot be used for attribution or causal claims.

An unavailable prerequisite does not justify weakening a security boundary, using an undocumented API, fabricating evidence, or marking a milestone complete.

## 20. Final acceptance

The pre-R8 connector program is complete only when:

- v0.7.1 and v0.7.2 satisfy their detailed release exit criteria;
- every required quality and protected-branch gate passes on the exact completion revisions;
- all implementation evidence is content-addressed, independently verifiable, and free of ground truth, secrets, tenant-sensitive data, and private chain of thought;
- R7 completion evidence remains unchanged;
- production scored SIEM execution, unrestricted network access, and unsupported MCP capabilities remain unavailable;
- R8 documentation identifies the exact connector contracts selected for compatibility freeze.

This planning change implements no connector, MCP server, network worker, vendor integration, schema, CI job, or release evidence.
