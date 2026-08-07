# HuntEval executable implementation plan

**Status:** Approved; implementation in progress  
**Plan version:** 0.2.0  
**Contract baseline:** protocol and schema version 0.3  
**Scope:** MVP vertical slice first; later work remains gated

## 1. Purpose and delivery rules

This plan turns the accepted specification and ADRs into small, reviewable pull requests. The first delivery target is one deterministic offline run that evaluates a two-agent deployment against a synthetic cloud episode while keeping hidden ground truth outside the deployment boundary.

The plan preserves these invariants:

- the evaluated unit is the complete deployment, not an individual model;
- domain types do not depend on the CLI, DuckDB, process management, model providers, storage adapters, or agent frameworks;
- all scored tool execution is mediated and recorded by HuntEval;
- deployment-visible observations never contain hidden ground truth;
- trajectories contain observable events only and never request private chain of thought;
- evidence and findings retain agent-to-action-to-result provenance;
- metric vectors are retained and aggregate weights come only from versioned profiles;
- authored manifests use YAML, protocol streams use JSONL, normalized results use JSON, and telemetry uses Parquet;
- retrieval remains optional and is not on the vertical-slice critical path;
- every public contract has an explicit version and compatibility policy;
- no web UI, Kubernetes, production SIEM integration, distributed execution, unrestricted network access, or autonomous optimization is introduced before the MVP is complete.

Each pull request ends at its stated acceptance criteria. A later pull request must not begin while an earlier dependency has a failing quality gate. All source, comments, schemas, fixtures, CLI output, and public documentation are written in English.

### 1.1 Rust security requirements

All first-party Rust code follows secure-by-default practices:

- every workspace crate inherits `unsafe_code = "forbid"`; unsafe first-party Rust is not permitted in the MVP;
- untrusted JSONL, YAML, JSON, SQL, paths, environment values, process output, retrieved text, and dataset metadata are validated at the boundary before conversion into domain types;
- parsers and process adapters enforce explicit limits for bytes, lines, nesting, rows, memory, time, concurrency, retries, and generated artifacts;
- authorization is enforced in trusted code and never delegated to deployment text, model behavior, or retrieved documents;
- subprocesses use explicit executable paths and argument arrays; first-party code does not construct shell command strings from untrusted input;
- filesystem operations reject traversal and unexpected symlinks, use least privilege, and avoid exposing private paths through errors or logs;
- secrets are not included in fixtures, trajectories, diagnostics, panic messages, or snapshots; sensitive configuration is redacted before persistence;
- production paths use typed errors and explicit recovery; they do not use `unwrap()`, `expect()`, uncontrolled `panic!`, or silently ignored failures;
- arithmetic affecting budgets, sizes, sequence numbers, and costs is checked for overflow and invalid numeric values;
- dependencies are minimized, pinned through `Cargo.lock`, reviewed for maintenance and provenance, and checked for advisories, denied sources, duplicate-risk versions, and incompatible licenses;
- security-sensitive behavior has positive, negative, abuse-case, timeout, and resource-exhaustion tests before merge.

Security controls must be implemented independently of agent instructions. A passing functional test cannot waive a failed security gate.

### 1.2 Clean Architecture requirements

The implementation follows Clean Architecture dependency rules:

- domain entities and policies form the innermost layer and have no knowledge of databases, filesystems, subprocesses, CLI parsing, serialization transport, model providers, or UI/report rendering;
- application use cases orchestrate domain behavior through ports and do not depend on concrete infrastructure adapters;
- protocol, DuckDB, filesystem, process, and reporting components are outer adapters that translate to and from validated domain types;
- traits are defined by the layer that consumes the capability, not by the infrastructure implementation that happens to provide it;
- dependencies point inward; an inner crate must never import an outer crate, and cycles between crates or modules are prohibited;
- transport DTOs, persistence representations, and domain entities remain distinct when their validation or compatibility responsibilities differ;
- composition and concrete adapter selection occur only at the application boundary in the runner or CLI;
- domain rules are testable without DuckDB, subprocesses, network access, CLI invocation, or filesystem fixtures.

New cross-layer dependencies require explicit justification in the pull request. A dependency-direction check is part of CI from PR-01 onward.

### 1.3 Human maintainability and readability requirements

Code is optimized for human review and long-term maintenance:

- names express domain intent; abbreviations, clever one-liners, implicit control flow, and unnecessary generic abstractions are avoided;
- modules have one cohesive responsibility and expose the smallest useful public API;
- functions remain focused, with early extraction when validation, orchestration, transformation, and I/O become mixed;
- comments explain invariants, security assumptions, trade-offs, and non-obvious decisions rather than restating syntax;
- public APIs and security-critical internal functions include concise rustdoc with errors, side effects, and relevant invariants;
- duplicated behavior is extracted only after the shared concept is clear; speculative frameworks and premature abstraction are prohibited;
- generated files are clearly identified and isolated from hand-written source;
- formatting and lint suppressions are scoped narrowly and include a reason; crate-wide or workspace-wide suppressions require an ADR;
- each pull request remains small enough to review coherently and separates mechanical refactors from behavior changes.

No hand-written production Rust source file may exceed 500 physical lines. The design target is at most 300 lines per file; crossing 300 lines triggers a mandatory cohesion review and either a split into named submodules or a written justification in the pull request. Generated code, canonical data fixtures, and snapshot outputs are exempt only when kept outside hand-written production modules. File splitting must follow responsibilities and must not create meaningless numbered fragments.

These requirements are part of every pull request's definition of done even when they are not repeated in its individual acceptance criteria.

## 2. MVP architecture and dependency direction

The initial workspace uses six crates. More crates are deferred until a measured boundary requires them.

```text
hunteval-cli
    -> hunteval-runner
       -> hunteval-protocol
       -> hunteval-duckdb
       -> hunteval-evaluation
       -> hunteval-domain

hunteval-protocol   -> hunteval-domain
hunteval-duckdb     -> hunteval-domain
hunteval-evaluation -> hunteval-domain
hunteval-domain     -> serde, time/UUID/hash primitives only
```

Dependency rules:

1. `hunteval-domain` owns infrastructure-neutral identifiers, manifests, trajectory state, submissions, results, metric values, and typed domain errors.
2. `hunteval-protocol` owns JSONL envelopes, wire DTOs, framing, compatibility checks, and conversion to validated domain commands/events.
3. `hunteval-duckdb` owns SQL policy, table registration, parameter conversion, worker request/response DTOs, and worker lifecycle.
4. `hunteval-evaluation` owns deterministic evaluators and scoring-profile application. It reads stored domain artifacts and never calls a deployment or model provider.
5. `hunteval-runner` owns orchestration, budgets, process adapters, trusted/private episode loading, trajectory persistence, hashing, and artifact layout.
6. `hunteval-cli` owns argument parsing and presentation only.

Deferred crates from the target layout—statistics, resilience, knowledge, reporting, and a Python SDK—must not be created as empty placeholders. Their boundaries are introduced in the pull requests that implement them.

## 3. Core modules and public contracts

### 3.1 Domain crate

Planned modules:

- `ids`: opaque newtypes for `RunId`, `MessageId`, `DeploymentId`, `AgentId`, `TaskId`, `ActionId`, `EvidenceId`, `HypothesisId`, `FindingId`, `EpisodeId`, and `EventId`;
- `version`: `SchemaVersion` and `ProtocolVersion` with supported-range checks;
- `episode`: `EpisodeManifest`, `PublicEpisode`, `TelemetryTable`, `EpisodeLimits`, and private `GroundTruth` loading contract represented without filesystem concerns;
- `deployment`: `DeploymentRegistration`, `AgentRegistration`, `Capability`, and `DeploymentArchitecture`;
- `task`: task state and validated transition types;
- `evidence`: `Evidence`, `Finding`, `Hypothesis`, `FinalSubmission`, and provenance references;
- `trajectory`: append-only `TrajectoryEvent`, causal metadata, and replay state;
- `metrics`: raw metric values, dimension vector, applicability, direction, and denominators;
- `result`: `RunResult`, `RunStatus`, `ResourceUsage`, `ConstraintViolation`, and artifact references;
- `error`: typed contract, validation, transition, provenance, and budget errors.

Production domain paths must not use `unwrap()` or `expect()`. Constructors validate non-empty opaque IDs, confidence ranges, UTC timestamps, duplicate identifiers, and state transitions.

### 3.2 Protocol crate

The wire protocol uses a tagged envelope with a bounded UTF-8 JSON object per line. Public types include:

- `Envelope<T>` with `protocol_version`, `message_id`, `run_id`, `timestamp`, and flattened payload;
- `DeploymentMessage` for untrusted deployment-to-runner messages;
- `RunnerMessage` for trusted runner-to-deployment observations and terminal notices;
- `ProtocolErrorMessage` with stable error code, safe message, correlation ID, and retryability;
- `JsonlDecoder` and `JsonlEncoder` with configurable maximum line size;
- `ProtocolSession` state that enforces handshake, registration, active-run, and terminal phases.

Proposed MVP handshake:

```text
runner -> run_started(public episode descriptor, limits, seed, supported protocol range)
deployment -> register_deployment(selected protocol version, deployment and agents)
runner -> registration_accepted(resolved public capabilities and remaining budgets)
deployment -> domain events or tool_request
runner -> accepted event, tool_result, protocol_error, budget_exhausted, or run_terminated
deployment -> final_submission
runner -> submission_accepted and run_terminated
```

Runner timestamps are authoritative for scored trajectory events. Deployment timestamps are retained only as untrusted metadata when supplied. Each runner response carries `caused_by_message_id`; tool results also carry `action_id`. Only one registration is accepted. Unknown message types produce a structured error; unknown optional fields are ignored within the same major version. EOF before a terminal submission produces a process-failure event.

### 3.3 Schemas

Versioned schemas live under `schemas/v0.3/`:

- `episode-manifest.schema.json`;
- `ground-truth.schema.json`;
- `deployment-registration.schema.json`;
- `protocol-message.schema.json`;
- `trajectory-event.schema.json`;
- `submission.schema.json`;
- `result.schema.json`;
- `scoring-profile.schema.json`;
- `benchmark-manifest.schema.json` after benchmark work begins.

Rust types are the implementation source of truth for the MVP, while checked-in JSON Schemas are compatibility artifacts tested against canonical examples. Schema changes require fixtures for the oldest supported minor version and rejection fixtures for unsupported major versions.

### 3.4 Trajectory, replay, and provenance

The runner is the only trajectory writer. It assigns a monotonic `sequence` beginning at one and appends one JSON object per line. Each event contains the common envelope, event kind, actor, optional task/action references, and a hash link to the previous canonical event. The final run artifact records the trajectory file SHA-256.

Replay performs no external calls. It:

1. validates JSONL framing, versions, unique IDs, sequence continuity, timestamps, and hash links;
2. applies domain state transitions in order;
3. rebuilds registered agents, tasks, actions, evidence, hypotheses, findings, budgets, and final submission;
4. rejects references to future, missing, cross-run, or wrong-owner objects;
5. re-runs deterministic evaluation from the reconstructed state and private ground truth;
6. compares the regenerated normalized result with the stored result.

An evidence item is valid only if every `source_action_id` belongs to a successful HuntEval-issued result in the same run and every referenced event ID was returned by at least one of those actions. Findings may reference only valid evidence. Final submission identifiers must be supported by accepted findings or explicitly marked as unsupported, which affects evidence metrics.

### 3.5 Hidden ground truth isolation

An authored episode package has separate roots:

```text
episode/
  public/manifest.yaml
  public/telemetry/*.parquet
  public/knowledge/*          # optional and absent in the first slice
  private/ground-truth.json
```

The trusted CLI resolves and hashes both roots before process launch. The deployment receives a generated public descriptor over stdin and a read-only public working directory containing only the public root. The private root path is never serialized into deployment messages, arguments, environment variables, logs, or resolved public artifacts. The evaluator receives a parsed `GroundTruth` value through a trusted in-process boundary after the deployment terminates.

The MVP supports Linux process isolation first and must fail closed if the configured isolation mechanism cannot guarantee the requested boundary. A test deployment attempts path traversal, environment discovery, and direct private-path access. Platform-portable sandboxing remains an open ADR and cannot be represented as complete until implemented and tested on that platform.

### 3.6 DuckDB worker and SQL policy

DuckDB executes in a child worker distinct from both the runner and deployment. The deployment never receives a database path. The runner sends the worker a typed request containing the query, bound parameters, exposed logical table names, and limits.

Policy validation has two layers:

1. parse the complete SQL input and require exactly one query statement whose root is `SELECT` or an allowed read-only relational expression;
2. walk the AST and reject mutation, DDL, `COPY`, `ATTACH`, `DETACH`, `PRAGMA`, extension operations, secrets, external scans, table functions, filesystem functions, network functions, unregistered catalogs/schemas/tables, and unapproved functions.

Execution adds DuckDB read-only mode, disabled external access, no extension autoload/install, a fixed temporary directory inside the worker sandbox, memory and thread limits, wall-clock timeout, maximum returned rows, and maximum serialized bytes. Timeouts kill the worker and return a typed `tool_timeout`; crashes return a typed worker failure without crashing the runner. Query results are converted into typed JSON values with an explicit schema and deterministic timestamp encoding.

The first slice permits parameterized projections, filters, joins over allowlisted tables, grouping, ordering, and bounded limits. Common table expressions, subqueries, and window functions require explicit AST tests before being enabled.

### 3.7 MVP metrics

The vertical slice implements only metrics whose contracts can be deterministic immediately:

| Metric | Range | Direction | Denominator and edge policy |
|---|---:|---|---|
| event precision | `[0,1]` | higher is better | submitted event IDs; empty submission with non-empty truth is `0`; both empty is `1` only for an explicitly benign episode |
| event recall | `[0,1]` | higher is better | ground-truth malicious event IDs; empty truth is not applicable unless the episode declares benign evaluation |
| entity precision | `[0,1]` | higher is better | submitted entity IDs; same empty-set policy as event precision |
| entity recall | `[0,1]` | higher is better | ground-truth malicious entity IDs; same empty-truth policy as event recall |
| evidence grounding rate | `[0,1]` | higher is better | submitted evidence items; no submitted evidence is `0` when findings exist and not applicable when no finding is required |
| provenance validity | `{0,1}` | higher is better | one run; any forged or cross-run reference makes it `0` |
| task completion rate | `[0,1]` | higher is better | created non-superseded tasks; zero created tasks is not applicable |
| tool-call utilization | `[0,1]` | lower raw usage is better | used calls / configured cap; zero cap with zero use is not applicable |

Raw counts and applicability are always preserved. A single run does not fabricate resilience or reproducibility values; those dimensions remain `null` until their required paired inputs exist. Attack-path, timeline, structured-conclusion, technique, evidence-completeness, canonical duplicate-work, causally useful communication, measured-duration, verified-cost, cross-run submission, and metric stability have separate deterministic contracts and tests. Registry-backed v0.4 scoring profiles enforce explicit missing-value behavior, registered direction, typed constraints, and verified resource provenance; v0.3 sources are normalized without rewriting them. Comparative confidence and reporting policy remain with their owning milestones.

## 4. Pull request sequence

### PR-00 — Close MVP contract decisions

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Record decisions that block implementation and align illustrative documentation with normative contracts.
2. **Files and crates affected:** `docs/ADR.md`, `docs/CONTRACTS.md`, `docs/METRICS_AND_RANKING.md`, `schemas/v0.3/` design notes; no Rust crates.
3. **Public types and contracts:** bidirectional handshake, runner-authoritative trajectory time, causal response IDs, metric applicability, seed/repetition semantics, and private/public episode layout.
4. **Tests:** schema examples are manually cross-checked in this documentation-only PR; executable tests start in PR-01.
5. **Acceptance criteria:** each open decision selected for the vertical slice has either a new accepted ADR or an explicit deferral; examples contain no contradictory field semantics.
6. **Dependencies:** approval of this execution plan.
7. **Risks and rollback:** changing protocol 0.3 before code exists is low risk; retain ADR IDs and use a new ADR instead of rewriting accepted historical decisions.

### PR-01 — Bootstrap the Rust workspace and contract primitives

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Create the smallest compiling workspace, CI, policy files, domain primitives, and CLI skeleton.
2. **Files and crates affected:** root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, CI configuration, contribution/security policy files, architecture and source-size check scripts, `crates/hunteval-domain`, `crates/hunteval-cli`, and `schemas/v0.3/`.
3. **Public types and contracts:** opaque IDs, UTC timestamp wrapper, schema/protocol versions, SHA-256 digest, `HuntEvalError`, and `hunteval --version`.
4. **Tests:** ID validation and serde round trips; RFC 3339 UTC rejection cases; version compatibility; hash formatting; schema example validation; dependency-direction check; source-size check; lint fixture proving first-party unsafe code is rejected.
5. **Acceptance criteria:** stable Rust builds offline after dependencies are fetched; CLI prints version; all crates inherit the workspace security lints; domain contains no prohibited infrastructure dependencies; no hand-written production Rust file exceeds 500 lines; dependency, advisory, license, rustdoc, formatting, lint, and test gates pass in CI and locally.
6. **Dependencies:** PR-00.
7. **Risks and rollback:** dependency sprawl and unstable APIs; pin a minimal dependency set and avoid empty future crates.

### PR-02 — Episode, deployment, and result contracts

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Implement versioned authored and normalized contracts before process execution.
2. **Files and crates affected:** domain modules `episode`, `deployment`, `evidence`, `result`, `metrics`; corresponding schemas; canonical examples and negative fixtures.
3. **Public types and contracts:** `EpisodeManifest`, `EpisodeLimits`, `GroundTruth`, `DeploymentRegistration`, `AgentRegistration`, `Evidence`, `Finding`, `FinalSubmission`, `RunResult`, and `MetricValue` with applicability.
4. **Tests:** YAML/JSON round trips; duplicate IDs; invalid confidence; non-UTC timestamps; missing limits; path traversal; public-manifest redaction; unknown optional fields; unsupported major versions.
5. **Acceptance criteria:** all current documentation examples have canonical valid fixtures; private ground truth cannot serialize as a public episode; schemas and Rust validation agree.
6. **Dependencies:** PR-01.
7. **Risks and rollback:** over-modeling illustrative fields; expose only fields required by the vertical slice and add optional fields compatibly.

### PR-03 — JSONL protocol session and deterministic replay state

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Implement bounded framing, handshake, registration, task delegation, evidence, finding, submission, and replay without launching a real deployment.
2. **Files and crates affected:** `hunteval-protocol`; domain `task` and `trajectory` modules; protocol and trajectory schemas; conformance fixtures.
3. **Public types and contracts:** `Envelope<T>`, `DeploymentMessage`, `RunnerMessage`, `ProtocolSession`, `TrajectoryEvent`, `ReplayState`, state-transition APIs, and stable protocol error codes.
4. **Tests:** valid transcript replay; oversized/malformed/non-UTF-8 lines; duplicate IDs; unknown agents/tasks/actions; invalid transitions; future and cross-run references; wrong action ownership; unsupported versions; early EOF; hash-link tampering.
5. **Acceptance criteria:** an in-memory canonical transcript reaches a valid final state; replay produces identical state; malformed input always returns a typed safe error without panic.
6. **Dependencies:** PR-02.
7. **Risks and rollback:** protocol ambiguity and event explosion; keep wire DTOs separate from domain commands and freeze only the vertical-slice subset.

### PR-04 — Deterministic AWS fixture and episode loader

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Generate and validate one synthetic AWS identity episode with benign noise and physically separate private truth.
2. **Files and crates affected:** `datasets/aws/aws-iam-001/`, a deterministic fixture generator under `tools/`, runner episode loader and hasher, fixture documentation.
3. **Public types and contracts:** `EpisodePackage`, `PublicEpisodePackage`, `ArtifactDigest`, `FixtureSeed`, and loader validation errors.
4. **Tests:** byte-identical regeneration; Parquet schema and stable IDs; reference query recovers truth; no malicious labels or private IDs in public files; symlink/path traversal rejection; modified-file hash detection.
5. **Acceptance criteria:** one command regenerates the fixture deterministically; public and private roots have disjoint resolved paths; all hashes are recorded; fixture contains a plausible benign alternative.
6. **Dependencies:** PR-02.
7. **Risks and rollback:** generator/library output may vary by version; pin writer versions and canonicalize ordering and metadata.

### PR-05 — Managed DuckDB worker

**Implementation status:** Completed on 2026-08-06.

1. **Objective:** Execute constrained read-only queries over the public fixture in a separate worker process.
2. **Files and crates affected:** `hunteval-duckdb` library and worker binary, runner worker adapter, SQL policy fixtures.
3. **Public types and contracts:** `SqlRequest`, `SqlParameters`, `SqlPolicy`, `QueryLimits`, `ToolResult`, `ToolError`, `DuckDbWorker`, and `ManagedTool` trait owned by the runner-facing boundary.
4. **Tests:** valid parameterized query; mutation and multi-statement rejection; `COPY`/`ATTACH`/`PRAGMA`/extension/file/network/table-function bypasses; unknown tables/functions; timeout; row/byte truncation; worker crash; deterministic result serialization.
5. **Acceptance criteria:** reference SQL returns expected public rows; forbidden SQL never reaches execution; timeout or crash cannot terminate the runner test process; deployment code has no DuckDB dependency.
6. **Dependencies:** PR-03 and PR-04.
7. **Risks and rollback:** parser and engine grammar mismatch; use a deny-by-default supported subset and add syntax only with positive and negative AST tests.

### PR-06 — Runner, process adapter, budgets, and artifact recorder

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Launch an untrusted deployment, mediate its complete protocol session, enforce budgets, and write reproducible run artifacts.
2. **Files and crates affected:** `hunteval-runner` modules `orchestrator`, `process`, `budget`, `policy`, `artifacts`, and `hashing`; CLI `run` command.
3. **Public types and contracts:** `RunConfig`, `RunOrchestrator`, `DeploymentProcess`, `BudgetLedger`, `ArtifactWriter`, `RunManifest`, terminal statuses, and process/tool failure events.
4. **Tests:** canonical child transcript; malformed output; stderr capture and redaction; process timeout/crash; message/tool/token budget exhaustion; atomic artifact finalization; interrupted partial run; environment allowlist; private-path and network-isolation probes.
5. **Acceptance criteria:** runner supplies only the public descriptor; every accepted action produces an append-only event; budget exhaustion is explicit; a failed deployment yields a normalized result shell and preserved trajectory.
6. **Dependencies:** PR-03, PR-04, and PR-05.
7. **Risks and rollback:** OS isolation differences and deadlocks; support one documented Linux isolation backend first, bound stdout/stderr, and fail closed when guarantees are unavailable.

### PR-07 — Deterministic evaluation and profile scoring subset

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Evaluate stored submissions and provenance against private truth without consulting the deployment.
2. **Files and crates affected:** `hunteval-evaluation` modules `sets`, `evidence`, `coordination`, `profile`, and `constraints`; metrics documentation and scoring-profile schema/example.
3. **Public types and contracts:** `Evaluator` trait, `EvaluationInput`, `MetricDefinition`, `MetricValue`, `MetricVector`, `ScoringProfile`, `AggregateScore`, and `ConstraintEvaluation`.
4. **Tests:** exact/partial/empty event and entity sets; benign episode policy; forged and unsupported evidence; zero denominators; non-applicable renormalization; invalid weights; hard constraint violations; deterministic normalized JSON.
5. **Acceptance criteria:** event/entity precision and recall, evidence grounding, provenance validity, task completion, and raw utilization follow documented contracts; unsupported dimensions remain `null`; weights are loaded only from the profile.
6. **Dependencies:** PR-02, PR-03, and PR-04.
7. **Risks and rollback:** misleading aggregate score with missing dimensions; require the profile to declare `reject`, `renormalize`, or `zero` for non-applicable dimensions.

### PR-08 — First end-to-end two-agent vertical slice

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Complete the required offline flow using a deterministic reference deployment.
2. **Files and crates affected:** reference deployment fixture under `deployments/two-agent-scripted`, runner orchestration integration, CLI, end-to-end tests, README quick start.
3. **Public types and contracts:** no new broad API; stabilize only the process command manifest and vertical-slice artifact layout.
4. **Tests:** full successful run; invalid evidence provenance; hidden-path probe; SQL policy violation recovery; process failure; replay and re-evaluation equality; golden `trajectory.jsonl` and normalized `result.json` with sanitized nondeterministic fields.
5. **Acceptance criteria:** one CLI command performs Parquet load, validation, two-agent registration, delegation, managed query, issued result, evidence, finding, submission, private evaluation, trajectory write, and result write; all required hashes are present; no private truth reaches deployment-visible artifacts.
6. **Dependencies:** PR-04 through PR-07.
7. **Risks and rollback:** a scripted deployment may overstate generality; label it a deterministic protocol fixture and make no model-quality claim.

### PR-09 — Protocol hardening and additional baseline topologies

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Generalize the proven protocol to single-agent, supervisor-investigator, and supervisor-specialist fixtures.
2. **Files and crates affected:** protocol/runner conformance suites and three reference deployments.
3. **Public types and contracts:** complete task lifecycle, operational messages, hypotheses, finding challenge/accept/reject, cancellation, and capability validation.
4. **Tests:** topology conformance; duplicate and orphan tasks; capability mismatch; cancellation/reassignment; message limits; scheduling permutations; property tests over malformed messages and transition sequences.
5. **Acceptance criteria:** all three topologies use the same public protocol with no framework SDK dependency; replay is deterministic under tested scheduling permutations.
6. **Dependencies:** PR-08.
7. **Risks and rollback:** premature topology-specific behavior in the runner; keep topology policy inside deployments and protocol validation topology-neutral.

### PR-10 — Benchmark matrix, statistics, and ranking

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Run multiple deployments, episodes, and paired seeds, then report defensible comparisons.
2. **Files and crates affected:** new `hunteval-statistics` crate, runner benchmark module, CLI benchmark/compare commands, benchmark and scoring schemas.
3. **Public types and contracts:** `BenchmarkManifest`, `RunCell`, `RunSet`, `StatisticalSummary`, `PairedDifference`, confidence interval, effect size, wins/ties/losses, and ranking groups.
4. **Tests:** seed-to-repetition mapping; resume after interruption; deterministic bootstrap seed; paired missing/failure cells; constraint-first ranking; inconclusive pairwise difference interval; invalid non-equivalent comparison labeling.
5. **Acceptance criteria:** matrix semantics are exactly deployment × episode × listed seed × declared configuration; `repetitions` is either derived from seeds or rejected when inconsistent; rankings retain raw metric vectors and never infer significance from marginal interval overlap alone.
6. **Dependencies:** PR-09 and at least two episodes.
7. **Risks and rollback:** pseudo-replication and overstated significance; pair by episode/seed, report sample counts, and label insufficient data.

### PR-11 — Complete cloud fixture MVP

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Add the remaining deterministic AWS, Azure, and Google Cloud episodes for the nine-episode benchmark.
2. **Files and crates affected:** provider fixture generators, nine episode packages, public schema documentation, benchmark manifest, integrity tests.
3. **Public types and contracts:** provider-native tables plus additive normalized semantic views; no lossy universal event replacement.
4. **Tests:** deterministic generation; provider schema validation; stable IDs; reference-query truth recovery; benign noise; public/private leakage scan; cross-provider normalized-view equivalence properties.
5. **Acceptance criteria:** three episodes per provider cover identity compromise, privilege escalation, and persistence/credential creation; every episode has an attack path and benign alternative; all regenerate byte-identically.
6. **Dependencies:** PR-04, PR-07, and PR-10.
7. **Risks and rollback:** unrealistic or label-leaking fixtures; require security review and keep generators as the auditable source.

### PR-12 — Resilience and deterministic fault injection

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Add paired baseline/fault runs and graceful degradation metrics.
2. **Files and crates affected:** new `hunteval-resilience` crate or runner module pending ADR, fault schemas, runner controller, fault fixtures.
3. **Public types and contracts:** `FaultProfile`, `FaultSchedule`, `FaultEvent`, `RecoveryOutcome`, and resilience metric definitions.
4. **Tests:** agent timeout, malformed response, worker failure, unavailable agent, noisy agent, deterministic task reassignment, retry budgets, paired degradation calculation.
5. **Acceptance criteria:** fault schedules reproduce from a seed; the runner survives all injected failures; resilience scores have documented ranges, directions, denominators, and edge cases.
6. **Dependencies:** PR-09 and PR-10.
7. **Risks and rollback:** nondeterministic process timing; inject at logical event boundaries rather than relying only on wall-clock races.

### PR-13 — Static reporting

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Produce normalized JSON and portable static HTML reports grounded in artifacts.
2. **Files and crates affected:** new `hunteval-reporting` crate, CLI report command, templates, snapshot tests.
3. **Public types and contracts:** `RunReport`, `BenchmarkReport`, `ArtifactLink`, and report rendering interface.
4. **Tests:** HTML escaping of untrusted text; artifact-link validation; incomplete/inconclusive labels; deterministic JSON; snapshot tests without embedded secrets or private paths.
5. **Acceptance criteria:** every displayed conclusion links to a metric or trajectory event; HTML requires no server or script execution; normalized JSON remains the source of truth.
6. **Dependencies:** PR-08 for run reports and PR-10 for benchmark reports.
7. **Risks and rollback:** injection through agent text and misleading summaries; escape by default and render only structured, cited claims.

### PR-14 — Optional local knowledge retrieval

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Add author-provided retrieval without making it a benchmark requirement.
2. **Files and crates affected:** new `hunteval-knowledge` crate, managed-tool adapter, document schema, injection fixtures.
3. **Public types and contracts:** `KnowledgeManifest`, `DocumentId`, `RetrievalRequest`, `RetrievalResult`, `Citation`, and local index adapter trait.
4. **Tests:** benchmark without retrieval; deterministic local retrieval; hidden-root denial; malicious instructions treated as data; metadata/path sanitization; exact citations; document and token budgets.
5. **Acceptance criteria:** retrieval is disabled by default, uses no network, cannot alter authorization/tool policy, and records queries, documents, citations, latency, and cost.
6. **Dependencies:** PR-08 and PR-13; not a dependency of PR-10 through PR-13.
7. **Risks and rollback:** accidental expansion into Self-RAG or policy injection; keep the tool read-only, optional, corpus-scoped, and independently authorized.

### PR-15 — Failure diagnosis and controlled improvement experiments

**Implementation status:** Completed on 2026-08-07.

1. **Objective:** Implement deterministic failure classification first, then controlled artifact comparisons under immutable safety constraints.
2. **Files and crates affected:** evaluation diagnosis modules, versioned taxonomy schema, experiment manifests, reporting extensions.
3. **Public types and contracts:** `FailureClassification`, `DiagnosticEvidence`, `Recommendation`, `ExperimentManifest`, `CandidateConstraint`, and `ValidationDecision`.
4. **Tests:** classifications cite observable events; unsupported diagnoses are omitted; immutable-section diffs rejected; hidden-test isolation; one-variable paired comparisons; regression and cost constraints.
5. **Acceptance criteria:** recommendations remain explicitly unvalidated until controlled validation passes; no private reasoning is requested; hidden test feedback is unavailable during selection; human review remains required.
6. **Dependencies:** PR-10, PR-12, and PR-13; assisted candidate generation is post-MVP.
7. **Risks and rollback:** causal overclaiming and benchmark overfitting; retain evidence citations, partitions, uncertainty, and manual approval gates.

## 5. Milestone dependency graph

```text
Plan approval
  -> PR-00 contract decisions
     -> PR-01 workspace
        -> PR-02 contracts
           -> PR-03 protocol/replay
           -> PR-04 AWS fixture
              -> PR-05 DuckDB worker
           -> PR-07 evaluation
        PR-03 + PR-04 + PR-05 -> PR-06 runner
        PR-04 + PR-05 + PR-06 + PR-07 -> PR-08 vertical slice
           -> PR-09 protocol hardening/topologies
              -> PR-10 benchmark/statistics
                 -> PR-11 nine cloud episodes
                 -> PR-12 resilience
              PR-08 + PR-10 -> PR-13 reporting
                 -> PR-14 optional retrieval
              PR-10 + PR-12 + PR-13 -> PR-15 diagnosis/experiments
```

The first usable vertical slice ends at PR-08. The cloud benchmark MVP ends at PR-13. PR-14 is optional and must never block the benchmark. PR-15 begins only after observable run artifacts and statistical comparisons are mature.

## 6. Open architectural questions

1. Which Linux sandbox backend provides the MVP deployment and worker filesystem/network guarantees without requiring privileged installation?
2. Which Rust SQL parser version most closely matches the embedded DuckDB grammar, and how will unsupported syntax fail closed?
3. Are runner-issued timestamps sufficient for deterministic replay, or must replay normalize timing fields separately from integrity hashes?
4. Should event hash chaining use canonical JSON, a binary canonical form, or hashes over exact JSONL bytes?
5. How are token counts and monetary cost verified for arbitrary external deployments rather than merely self-reported?
6. Should benign episodes be part of the first fixture set to lock empty-ground-truth metric semantics early?
7. Which JSON Schema draft and compatibility checker are normative for schema 0.3?
8. Should the DuckDB worker be the same binary in a dedicated subcommand or a separately packaged executable for stronger hashing and isolation?
9. What is the exact allowed SQL function set for each provider-native fixture and normalized view?
10. How are bootstrap confidence intervals seeded, and what minimum paired sample count prevents a ranking claim?
11. What build identifier is reproducible across local builds: executable hash, source revision plus lockfile hash, or both?
12. Which fields in model/deployment configuration may be redacted in public reports while retaining a stable configuration hash?

## 7. ADRs introduced by PR-00

- **ADR-033 — Define the bidirectional JSONL session and runner-authoritative event ordering.** Accepted; covers handshake, causality, timestamps, EOF, bounded lines, and terminal messages.
- **ADR-034 — Define public/private episode packaging and fail-closed process isolation.** Accepted; covers roots, mounts, environment, symlinks, platform scope, and leakage tests.
- **ADR-035 — Define canonical trajectory hashing and replay equivalence.** Accepted; covers exact-byte hashing, sequence, partial files, and deterministic result comparison.
- **ADR-036 — Define the supported SQL subset and dual-layer worker enforcement.** Accepted; covers parser mismatch, AST allowlists, engine settings, and resource termination.
- **ADR-037 — Define metric applicability and aggregate missing-value policy.** Accepted; covers benign episodes, zero denominators, `null` dimensions, and profile behavior.
- **ADR-038 — Define seed and repetition semantics for paired comparisons.** Accepted; listed seeds define paired run cells and inconsistent legacy repetition counts are rejected.
- **ADR-039 — Define reproducible fixture generation and Parquet canonicalization.** Accepted; covers pinned writers, ordering, metadata, hashes, and generator provenance.
- **ADR-040 — Define trusted versus self-reported resource usage.** Accepted; covers tokens, cost, duration, and report labeling.

## 8. Validation commands

Run after every Rust pull request:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
./scripts/check-dependency-direction.sh
./scripts/check-source-size.sh
```

The two repository scripts are introduced in PR-01. `check-dependency-direction.sh` rejects forbidden inward-to-outward crate dependencies and cycles. `check-source-size.sh` fails when a hand-written production `.rs` file exceeds 500 physical lines and reports files above the 300-line review threshold. CI also verifies that production sources contain no prohibited panic shortcuts or undocumented lint suppression. These mechanical checks complement, but do not replace, human review of cohesion, naming, error handling, and security assumptions.

Additional commands by milestone:

| Pull request | Exact additional commands |
|---|---|
| PR-00 | `rg -n "Status: Proposed|TODO|TBD" docs/ADR.md docs/CONTRACTS.md docs/METRICS_AND_RANKING.md` and review every match |
| PR-01 | `cargo metadata --no-deps --format-version 1`; `cargo run -p hunteval-cli -- --version`; `cargo test -p hunteval-domain` |
| PR-02 | `cargo test -p hunteval-domain contracts`; `cargo test -p hunteval-domain schema`; `cargo test -p hunteval-domain ground_truth` |
| PR-03 | `cargo test -p hunteval-protocol`; `cargo test -p hunteval-protocol --test conformance`; `cargo test -p hunteval-protocol --test malformed` |
| PR-04 | `cargo run -p hunteval-fixture-tool -- generate datasets/aws/aws-iam-001`; `cargo test -p hunteval-runner fixture`; `git diff --exit-code -- datasets/aws/aws-iam-001` |
| PR-05 | `cargo test -p hunteval-duckdb`; `cargo test -p hunteval-duckdb --test sql_policy`; `cargo test -p hunteval-duckdb --test worker_failures` |
| PR-06 | `cargo test -p hunteval-runner`; `cargo test -p hunteval-runner --test process_failures`; `cargo test -p hunteval-runner --test isolation` |
| PR-07 | `cargo test -p hunteval-evaluation`; `cargo test -p hunteval-evaluation --test metric_contracts`; `cargo test -p hunteval-evaluation --test scoring_profiles` |
| PR-08 | `cargo test --workspace --test vertical_slice`; `cargo run -p hunteval-cli -- run --episode datasets/aws/aws-iam-001 --deployment deployments/two-agent-scripted`; `cargo run -p hunteval-cli -- trajectory inspect runs/latest/trajectory.jsonl` |
| PR-09 | `cargo test -p hunteval-protocol --test topology_conformance`; `cargo test -p hunteval-runner --test scheduling`; `cargo test -p hunteval-protocol --test malformed` |
| PR-10 | `cargo test -p hunteval-statistics`; `cargo test -p hunteval-runner benchmark`; `cargo run -p hunteval-cli -- benchmark validate examples/cloud-mvp-benchmark.yaml` |
| PR-11 | `cargo run -p hunteval-fixture-tool -- generate-all`; `cargo test --workspace cloud_fixtures`; `git diff --exit-code -- datasets` |
| PR-12 | `cargo test -p hunteval-resilience`; `cargo test -p hunteval-runner --test fault_injection`; `cargo test -p hunteval-evaluation resilience` |
| PR-13 | `cargo test -p hunteval-reporting`; `cargo test -p hunteval-reporting --test untrusted_rendering`; `cargo run -p hunteval-cli -- report generate runs/latest --format json`; `cargo run -p hunteval-cli -- report generate runs/latest --format html` |
| PR-14 | `cargo test -p hunteval-knowledge`; `cargo test -p hunteval-knowledge --test injection`; `cargo test -p hunteval-runner --test no_retrieval` |
| PR-15 | `cargo test -p hunteval-evaluation diagnosis`; `cargo test -p hunteval-evaluation experiments`; `cargo test -p hunteval-reporting diagnostic_report` |

Security-sensitive PRs also run the relevant negative fixtures under constrained CI. Milestone completion reports must list exact commands, results, known limitations, deferred work, documentation changes, and any ADR status changes. A milestone is not complete when any required command fails.
