# v0.7.1 framework connector pack implementation plan

## 1. Purpose and release position

This document defines the governed implementation sequence for the v0.7.1 framework connector pack. It extends the completed R7 release without reopening or changing any R7 completion claim. It must finish before v0.7.2 commercial platform connector closure and before its interfaces can enter the v1.0 stable freeze set. The governed R8-00 inventory may proceed while this work remains pending only by excluding these interfaces from that set.

The user-visible outcome is that equivalent HuntEval deployments can be authored with multiple agent frameworks and evaluated through the same process-neutral protocol, scored-tool mediation, topology artifacts, budgets, and reporting semantics.

The planned framework set is:

1. CrewAI as the existing reference connector and compatibility baseline;
2. LangGraph;
3. AutoGen AgentChat;
4. Google Agent Development Kit, including A2A interoperability where its stable public contract permits it;
5. Semantic Kernel agent orchestration, explicitly labeled experimental while its upstream orchestration API remains experimental.

Framework support never makes a framework, model provider, or hosted runtime a dependency of the Rust core. A single-agent deployment remains a valid baseline.

## 2. Delivery status

Status is evidence-based. `planned` makes no implementation claim. `in progress` means only part of the milestone behavior or governance exists. `implemented` requires focused local behavior and tests. `complete` additionally requires all canonical gates, package evidence, documentation, and passing GitHub Actions on the exact evidence revision.

| Milestone | Status | Outcome |
|---|---|---|
| F71-00 | implemented | code boundaries, support matrix, threat-model delta, exact candidate-version inventory and local public-API conformance, and accepted ADR-084 through ADR-089 exist |
| F71-01 | implemented | framework-neutral connector lifecycle and observable-event mapping with focused tests |
| F71-02 | implemented | CrewAI uses the common lifecycle without fixture regression |
| F71-03 | implemented | dependency-free LangGraph structural adapter and deterministic fixture conformance |
| F71-04 | implemented | async-capable AutoGen AgentChat structural adapter and deterministic fixture conformance |
| F71-05 | implemented | adapter aligned to the public local Google ADK `Runner.run` event stream; remote A2A remains unavailable |
| F71-06 | implemented | bounded Semantic Kernel preview structural adapter and status enforcement |
| F71-07 | implemented | bounded MCP session, fixed tool catalog, stdio processor, and adversarial focused tests |
| F71-08 | in progress | five adapters preserve equivalent controls and protocol activity in a deterministic supervisor-worker matrix; full scored benchmark evidence remains open |
| F71-09 | implemented | guides, exact optional dependency groups, upstream conformance harness, package inventory, and local/Actions CI gates exist; full protected matrix, migration rehearsal, and release closure remain open |

No F71 milestone is complete. Current implementation evidence is local and does not establish upstream-version or provider-backed support. `implemented` is not a release-support claim.

## 3. Normative boundaries

Every v0.7.1 change must:

- preserve Clean Architecture and keep framework packages outside domain, evaluation, scoring, reporting, and runner policy;
- use the existing HuntEval deployment protocol as the authority for registration, tasks, delegation, managed-tool calls, evidence, findings, terminal state, and final submission;
- keep scored-tool execution in HuntEval and reject framework-native direct execution of an equivalent scored tool;
- preserve agent, role, task, action, tool-result, evidence, finding, and final-submission provenance;
- record only observable messages, actions, events, structured decisions, and concise reason codes;
- never request or store private chain of thought, framework scratchpads, hidden model state, or provider reasoning fields;
- treat framework events, metadata, tool arguments, messages, and generated text as untrusted and bounded;
- keep ground truth, hidden-test membership, evaluator diagnostics, and scoring internals unavailable to connectors and deployments;
- preserve the raw metric vector, scoring profiles, constraint-first ranking, and missing-value semantics;
- fail closed on unknown event types, duplicate identities, invalid correlations, unsupported lifecycle transitions, and undeclared capabilities;
- make optional framework dependencies explicit extras rather than base SDK dependencies;
- pin and record connector, framework, Python, protocol, schema, and fixture versions in conformance artifacts;
- support deterministic fixture replay without model-provider or network access;
- preserve stable Rust, typed errors, no production `unwrap()` or `expect()`, and the repository file-size conventions.

## 4. Architecture and dependency direction

```text
framework deployment
  -> framework-specific Python connector
       -> hunteval_sdk process-protocol peer
            -> HuntEval supervised deployment process
                 -> runner-owned managed tools and budgets
                      -> append-only public trajectory artifacts

Rust domain/evaluation/reporting
  <- no framework dependency
```

Framework connectors translate public framework lifecycle events into HuntEval protocol messages. They do not translate HuntEval policy into framework-native authority. If a framework cannot expose a required observable event or stable identity, the affected metric or topology field remains unavailable.

The base Python SDK must remain installable without any framework package. Each connector must use an optional dependency group and an explicit supported-version range. Importing the base SDK must not import a framework transitively.

## 5. F71-00 — Contracts, support policy, and ADRs

F71-00 must inventory the exact public APIs used for every connector and choose the next additive schema version if new normative artifacts are required. Existing schema 0.3 through 0.9 and protocol 0.3 artifacts remain byte-compatible.

The implementation review must accept or revise these proposed decisions:

- ADR-084: framework connectors are optional out-of-core protocol peers;
- ADR-085: observable framework events map through one versioned connector event vocabulary;
- ADR-086: unavailable framework observations remain unavailable and are never inferred;
- ADR-087: framework dependencies use isolated optional package extras and a tested support matrix;
- ADR-088: connector conformance uses deterministic framework doubles before provider-backed tests;
- ADR-089: an MCP interoperability server is a bounded protocol adapter and never an alternate policy or tool-execution authority.

The support matrix must distinguish `supported`, `preview`, `unsupported`, and `removed`. It must name exact framework versions, Python versions, connector versions, protocol versions, known limitations, and removal policy.

Exit criteria:

- affected contracts and compatibility behavior are explicit;
- every public field has bounds, validation, and provenance rules;
- security and ground-truth-isolation impacts are reviewed;
- no framework package appears in a Rust core dependency graph;
- canonical examples and malformed examples exist for any new normative artifact.

## 6. F71-01 — Common connector lifecycle

Introduce a small framework-neutral connector layer in the Python SDK that maps observable framework activity to existing protocol messages. It must cover:

- deployment registration and declared topology;
- stable agent identity, role, specialization, and parent/peer relationships;
- task creation, ownership, delegation, start, completion, failure, and cancellation;
- sequential, parallel, handoff, supervisor/worker, and supervisor/specialist activity where observable;
- managed-tool request/result correlation;
- evidence and finding production;
- explicit terminal state and structured final submission;
- token, tool, time, and other observable resource usage when authoritatively available.

The common layer must not invent a universal framework state model. It provides bounded adapters to HuntEval's existing public protocol and rejects ambiguous or conflicting mappings.

Tests must cover duplicate agents, duplicate tasks, unknown parents, delegation cycles where forbidden, late events, invalid transitions, mismatched tool correlations, multiple terminal messages, oversized metadata, unknown event kinds, and malformed framework output.

## 7. F71-02 through F71-06 — Framework connectors

### F71-02 — CrewAI baseline

- move the existing CrewAI connector behind the common lifecycle without changing its public supported behavior;
- preserve single-agent and multi-agent crews, delegation, runner-mediated tools, and structured final submission;
- add regression fixtures that prove byte-equivalent normalized protocol output where the common contract is unchanged;
- record any unavoidable migration as an additive connector-version change.

### F71-03 — LangGraph

- map nodes, subgraphs, routing, fan-out, joins, handoffs, and terminal nodes when exposed by public APIs;
- preserve explicit node/agent/task identity rather than deriving it from display text;
- distinguish graph topology from runtime execution order;
- reject direct scored-tool bindings that bypass HuntEval.

### F71-04 — AutoGen AgentChat

- support single agents and declared teams using stable public team events;
- cover round-robin, selector, swarm/handoff, and graph flow only for supported upstream versions;
- capture observable team messages and handoffs without persisting hidden model context;
- reject inconsistent saved state, participant identity changes, and missing termination conditions.

### F71-05 — Google ADK/A2A

- support sequential, parallel, and loop agents where their events are observable;
- keep A2A transport identity separate from HuntEval agent and task identity;
- authenticate remote A2A endpoints only through the later v0.7.2 network policy; F71 fixtures remain offline;
- reject undeclared remote agents, changed agent cards, and unsupported streaming events.

### F71-06 — Semantic Kernel preview

- target only documented public orchestration APIs and exact supported versions;
- cover sequential, concurrent, handoff, and group-chat patterns where stable enough for conformance;
- label the connector `preview` while upstream orchestration remains experimental;
- return a typed unsupported result rather than implementing reflection, private APIs, or version guessing.

Each connector must ship positive, negative, malformed-input, lifecycle, timeout, crash, deterministic-replay, and package-isolation tests.

## 8. F71-07 — MCP interoperability server

Provide a HuntEval Model Context Protocol (MCP) server so a framework without a native HuntEval connector can participate through a documented, framework-neutral interface. It is an optional out-of-process adapter over the existing HuntEval deployment protocol; it does not replace that protocol or make MCP a dependency of the Rust domain, evaluation, scoring, reporting, or runner-policy cores.

The first supported transport must be local supervised `stdio`. Remote Streamable HTTP is unavailable in v0.7.1 and may be considered only through the deny-by-default network, authentication, origin, DNS, SSRF, secret, and budget controls defined by v0.7.2. Legacy HTTP+SSE, custom transports, arbitrary command execution, and caller-selected server binaries are unsupported.

The server must:

- pin and record the exact MCP protocol revision, server binary digest, HuntEval protocol version, schema version, and declared capability set;
- negotiate capabilities explicitly and reject undeclared, added-after-initialization, or unsupported capabilities;
- expose a finite versioned catalog for public observation retrieval, task and delegation events, runner-mediated managed-tool requests, evidence, findings, resource-usage declarations, terminal state, and final submission;
- require opaque deployment, session, agent, role, task, action, evidence, and finding identifiers supplied through validated fields rather than deriving identities from display text;
- bind every request to one supervised HuntEval run and reject cross-run handles, replayed request identifiers, confused-deputy attempts, and capability escalation;
- expose only public deployment-visible episode material and managed-tool results; ground truth, hidden-test membership, evaluator diagnostics, scoring internals, secret values, and private paths remain inaccessible;
- keep MCP tool execution subject to the same HuntEval policy, budgets, correlation, provenance, timeout, cancellation, and append-only trajectory rules as native connectors;
- disable MCP sampling, elicitation, roots, server-provided prompts, arbitrary resources, resource subscriptions, dynamic tool-catalog changes, and every other capability not required by the accepted adapter contract;
- treat tool descriptions, resource metadata, arguments, messages, errors, progress notifications, and client-provided text as bounded untrusted input;
- record only observable activity and never request or store private chain of thought, client scratchpads, or hidden model/provider state;
- report topology and coordination fields as unavailable when the generic client does not supply authoritative observable events.

A small language-neutral conformance harness must prove that an otherwise unsupported framework can complete the canonical single-agent run and one declared multi-agent run exclusively through MCP. This demonstrates protocol interoperability only; it does not grant that framework native-support status or imply that framework-specific topology and utilization metrics are observable.

Tests must cover initialization order, version negotiation, capability downgrade and escalation, duplicate request identifiers, malformed JSON-RPC, batches where unsupported, unsolicited responses, notification floods, cancellation races, oversized messages, invalid UTF-8, stdout contamination, stderr secret leakage, process crash, timeout, cross-run identifiers, direct scored-tool bypass, ground-truth requests, prompt/tool injection, deterministic replay, and clean shutdown.

## 9. F71-08 — Cross-framework benchmark evidence

Run the same paired benchmark cells through equivalent reference deployments for CrewAI, LangGraph, AutoGen, and Google ADK. Semantic Kernel participates only if its preview connector passes the same conformance requirements.

Control variables must include:

- episode, dataset, seed, budgets, model configuration, tool policy, scoring profile, topology declaration, and stopping conditions;
- exact framework, connector, SDK, protocol, schema, prompt/configuration, and binary identities;
- every intentional framework-specific difference.

The comparison must separate framework overhead from investigation quality. Raw metrics remain authoritative. A different observable capability must be reported as a limitation, not converted into an equivalent metric. Results are framework-version-dependent and cannot establish universal causal claims without controlled ablations.

Required evidence includes connector transcripts, normalized protocol events, topology artifacts, run verification, replay verification, resource measurements, unavailable-field declarations, and report limitations.

The MCP conformance deployments participate as a separate interoperability class. Reports must not group generic MCP compatibility with native connector support or infer framework-specific metrics that the MCP client did not emit authoritatively.

## 10. Security and ground-truth tests

The connector suite must prove that:

- framework tools cannot invoke scored capabilities except through HuntEval;
- framework callbacks cannot read private manifests, hidden partitions, evaluator diagnostics, or ground truth;
- untrusted framework text cannot alter runner policy or connector capability declarations;
- environment variables, provider credentials, and framework secrets are absent from artifacts and error output;
- forged agent/task/action identities and cross-run correlations fail closed;
- hidden reasoning and framework scratchpad fields are ignored or rejected;
- cancellation, timeout, process crash, invalid UTF-8, oversized output, and partial JSONL do not terminate the runner;
- equivalent fixture replay produces identical normalized public artifacts.

The MCP suite must additionally prove that clients cannot enumerate or invoke undeclared tools, resources, prompts, roots, sampling, elicitation, or remote transports, and that protocol metadata cannot alter runner policy.

Provider-backed smoke tests, if added, must be opt-in, non-scored, secret-safe, and excluded from deterministic CI evidence.

## 11. Quality gates and CI

Add a canonical `scripts/ci/v071-framework-connectors.sh` gate and a required `Framework connectors` GitHub Actions job. The gate must run without provider credentials or external network access.

Before v0.7.1 closure, run:

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
./scripts/ci/e2e.sh
```

The new gate must additionally verify base-SDK installation without framework extras, each supported extra in isolation, MCP `stdio` conformance, cross-framework golden fixtures, dependency licenses, package contents, and absence of secrets or private artifacts.

## 12. Pull-request sequence

1. PR-F71-00: contracts, support matrix, ADRs, threat model, schemas, and fixtures.
2. PR-F71-01: common Python connector lifecycle and negative tests.
3. PR-F71-02: CrewAI migration and regression evidence.
4. PR-F71-03: LangGraph connector.
5. PR-F71-04: AutoGen AgentChat connector.
6. PR-F71-05: Google ADK/A2A connector.
7. PR-F71-06: Semantic Kernel preview or typed unsupported decision.
8. PR-F71-07: MCP interoperability server, language-neutral conformance harness, and adversarial protocol tests.
9. PR-F71-08: paired benchmark, reports, conformance matrix, and CI.
10. PR-F71-09: documentation, package, migration, rollback, and exact closure evidence.

Every pull request must be independently reviewable and must update its owning tests and documentation. Parallel connector work may begin only after F71-00 and F71-01 freeze the common contract.

## 13. Migration and rollback

Existing CrewAI users retain the current supported entry point during one documented compatibility window. Any replacement API must provide an explicit adapter or typed rejection with migration instructions. Existing R7 artifacts remain valid and unchanged.

Each framework connector can be disabled or removed from an optional dependency group without changing the base SDK or Rust core. The MCP server can be removed from the allowed adapter inventory independently. Rollback restores the prior connector or MCP package and support-matrix entry; it never rewrites stored run artifacts or silently changes their declared connector identity.

## 14. Known limitations

- framework APIs and event models evolve independently of HuntEval;
- equivalent topology declarations do not guarantee equivalent framework behavior;
- provider token usage, hidden state, and scheduling details may be unavailable;
- deterministic fixtures validate integration semantics, not provider availability or model quality;
- Semantic Kernel support may remain preview or unavailable;
- MCP compatibility proves only the finite generic HuntEval contract and does not provide native framework instrumentation;
- remote MCP transport, server-provided prompts, sampling, elicitation, roots, arbitrary resources, and dynamic tool discovery remain unavailable in v0.7.1;
- distributed hosted framework runtimes and unrestricted remote A2A execution are outside v0.7.1;
- no connector may claim support outside its exact conformance matrix.

## 15. Release exit criteria

v0.7.1 is complete only when:

- R7 remains complete and unchanged;
- the base SDK has no mandatory framework dependency;
- CrewAI, LangGraph, AutoGen, and Google ADK pass the common conformance suite;
- Semantic Kernel has either passing preview evidence or an explicit typed unsupported result;
- an unsupported-framework fixture completes canonical single-agent and multi-agent runs through the bounded MCP `stdio` interface;
- MCP capability negotiation, lifecycle, policy mediation, isolation, malformed input, process failure, and deterministic replay tests pass;
- unsupported MCP features and remote transports fail closed and no MCP request can bypass HuntEval-managed tools;
- paired benchmark artifacts preserve all declared control variables and changed variables;
- every tool action remains runner-mediated and provenance-complete;
- unavailable observations remain unavailable;
- framework text, state, and metadata remain bounded and untrusted;
- ground truth and hidden tests remain isolated;
- all canonical local and protected-branch gates pass on the exact closure revision;
- documentation records support versions, migration, rollback, and limitations;
- no v0.8 interface freeze is claimed by this release.

## 16. Upstream references

- LangGraph overview: <https://docs.langchain.com/oss/python/langgraph/overview>
- AutoGen AgentChat teams: <https://microsoft.github.io/autogen/stable/reference/python/autogen_agentchat.teams.html>
- Google ADK agent package: <https://google.github.io/adk-docs/api-reference/java/com/google/adk/agents/package-use.html>
- Google Agents CLI A2A templates: <https://google.github.io/agents-cli/guide/templates/>
- Semantic Kernel orchestration: <https://learn.microsoft.com/en-us/semantic-kernel/frameworks/agent/agent-orchestration/>
- Model Context Protocol architecture and capability negotiation: <https://modelcontextprotocol.io/specification/2025-11-25/architecture>
- Model Context Protocol transports: <https://modelcontextprotocol.io/specification/2025-11-25/basic/transports>
- Model Context Protocol security guidance: <https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices>

These references describe upstream capabilities only. The accepted support matrix and HuntEval conformance evidence remain authoritative for connector support.
