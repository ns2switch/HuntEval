# Architecture decision records

Each decision is accepted unless its status states otherwise.

## ADR-001 — Use Rust for the core

**Status:** Accepted.

Rust provides strong typing, safe concurrency, portable binaries, predictable resource control, and a suitable foundation for a CLI benchmark runner.

## ADR-002 — Use DuckDB and Parquet as the canonical local environment

**Status:** Accepted.

They support reproducible offline analytics without requiring a production SIEM or cloud account.

## ADR-003 — Evaluate the deployment rather than the model

**Status:** Accepted.

The deployment includes agents, prompts, models, memory, tools, and coordination. Evaluating only a model would hide architectural effects.

## ADR-004 — Use a process-neutral JSONL protocol for the MVP

**Status:** Accepted.

A line-oriented process protocol permits integrations in Python, Rust, and other languages while isolating deployment failures.

## ADR-005 — Execute scored tools through HuntEval

**Status:** Accepted.

During scored runs, agents request actions and HuntEval validates and executes them. This protects budgets, telemetry, and hidden ground truth.

## ADR-006 — Physically separate hidden ground truth

**Status:** Accepted.

The runner exposes only the public episode package. The evaluator reads ground truth from a path that is not mounted into the deployment process.

## ADR-007 — Do not collect private chain of thought

**Status:** Accepted.

HuntEval records observable actions, operational messages, reason codes, hypotheses, evidence, and conclusions. It does not request hidden reasoning traces.

## ADR-008 — Use event sourcing for run trajectories

**Status:** Accepted.

Append-only events support replay, attribution, audit, evaluator evolution, and post-run analysis.

## ADR-009 — Require agent identity and capability registration

**Status:** Accepted.

Each agent declares a stable ID, role, capabilities, model, and prompt version so actions can be attributed.

## ADR-010 — Support centralized and decentralized coordination

**Status:** Accepted.

The protocol does not require a supervisor. Events preserve source, target, task, and causal relationships.

## ADR-011 — Require structured provenance

**Status:** Accepted.

Evidence and findings must link agents, actions, and HuntEval-issued tool results.

## ADR-012 — Treat RAG as an optional auxiliary capability

**Status:** Accepted.

RAG may expose hunt-author knowledge or historical reports, but it does not define HuntEval's primary evaluation objective.

## ADR-013 — Preserve metric vectors before calculating aggregate scores

**Status:** Accepted.

Independent dimensions remain visible. Aggregate scores are contextual and are calculated by versioned scoring profiles.

## ADR-014 — Base rankings on repeated runs

**Status:** Accepted.

Stochastic deployments require repetitions, confidence intervals, stability analysis, and explicit seeds.

## ADR-015 — Version scoring profiles outside the binary

**Status:** Accepted.

Weights and minimum constraints belong to benchmark configuration, not hard-coded application logic.

## ADR-016 — Make fault injection a first-class benchmark feature

**Status:** Accepted for post-MVP implementation.

Agent timeouts, malformed messages, tool failures, and missing agents are required to evaluate graceful degradation and recovery.

## ADR-017 — Use raw provider tables plus normalized semantic views

**Status:** Accepted.

Provider-specific semantics must remain available while normalized views enable cross-cloud comparisons.

## ADR-018 — Parse SQL structurally and execute it in a constrained worker

**Status:** Accepted.

String matching is insufficient. SQL is parsed, checked against an allowlist, and executed in a separate resource-limited process.

## ADR-019 — Use YAML for authored manifests and JSON for results

**Status:** Accepted.

YAML improves maintainability for episodes and benchmarks. JSON simplifies machine processing and schema validation of outputs.

## ADR-020 — Keep the domain crate infrastructure-independent

**Status:** Accepted.

Domain types cannot depend on DuckDB, CLI frameworks, process management, LLM SDKs, or storage adapters.

## ADR-021 — Treat deployment output and retrieved content as untrusted

**Status:** Accepted.

All messages, SQL, citations, paths, and text are validated. Retrieved documents cannot change tool or authorization policies.

## ADR-022 — Require prompt and configuration hashing

**Status:** Accepted.

Every run records prompt hashes, versions, model settings, dataset hashes, and benchmark configuration.

## ADR-023 — Attribute performance through observable provenance

**Status:** Accepted.

Agent contribution is derived from actions, evidence, findings, and tool results, not inferred from private reasoning.

## ADR-024 — Keep prompt recommendations separate from validated improvements

**Status:** Accepted.

A recommendation is a hypothesis until a controlled experiment demonstrates improvement on validation episodes.

## ADR-025 — Use train, validation, and hidden test partitions for prompt experiments

**Status:** Accepted for the prompt-optimization phase.

This reduces benchmark overfitting and prevents candidate prompts from being selected on hidden test results.

## ADR-026 — Keep authorization and safety prompt sections immutable

**Status:** Accepted.

Automated or assisted prompt changes cannot modify authorization, data handling, tool access, or benchmark-integrity policies.

## ADR-027 — Measure coordination overhead explicitly

**Status:** Accepted.

Inter-agent messages, duplicate work, waiting, delegation depth, and evidence propagation are part of deployment performance.

## ADR-028 — Require deployment-level budgets

**Status:** Accepted.

Agent count, concurrency, tool calls, messages, tokens, duration, and cost are bounded at the deployment level.

## ADR-029 — Provide deterministic synthetic fixtures before public real-world datasets

**Status:** Accepted.

Synthetic cloud episodes provide legal clarity, stable IDs, controlled background noise, and exact ground truth for the MVP.

## ADR-030 — Generate normalized JSON and static HTML reports

**Status:** Accepted.

JSON is the source of truth. Static HTML provides a portable human-readable report without introducing a web service.

## ADR-031 — Reject a single universal leaderboard profile

**Status:** Accepted.

Accuracy-first, cost-aware, and resilience-first use cases require different weights and constraints.

## ADR-032 — Require English throughout project artifacts

**Status:** Accepted.

Documentation, source comments, prompts, examples, schemas, CLI messages, and reports must use English to support an international open-source community.

## ADR-033 — Define a bidirectional JSONL session with runner-authoritative event ordering

**Status:** Accepted.

The runner starts each session with `run_started`, including the public episode descriptor, resolved limits, seed, and supported protocol range. The deployment selects one supported version in `register_deployment`; the runner replies with `registration_accepted` before accepting scored actions. Runner responses identify the deployment message that caused them. The runner assigns authoritative trajectory timestamps and sequence numbers. Deployment timestamps are untrusted metadata and do not determine ordering or budgets. Input is bounded to one UTF-8 JSON object per line, and premature EOF produces a structured process failure.

## ADR-034 — Use separate public and private episode roots and fail closed on isolation

**Status:** Accepted.

An episode package has physically distinct `public` and `private` roots. Only the public root may be exposed read-only to a deployment process. Private paths, ground-truth references, and private hashes are never serialized into deployment-visible messages, environment variables, logs, or resolved public artifacts. The trusted evaluator loads ground truth after the deployment boundary has been established. The MVP supports a documented Linux isolation backend and refuses scored execution when its filesystem or network guarantees cannot be established.

## ADR-035 — Hash exact trajectory bytes and require deterministic replay equivalence

**Status:** Accepted.

The runner serializes each trajectory event once as compact JSONL. Every event after the first contains the SHA-256 of the exact preceding line bytes, including its newline. The run manifest records the SHA-256 of the complete trajectory file. Replay validates framing, sequence continuity, hash links, causal references, and state transitions without external providers, then re-runs deterministic evaluators. Fields declared operationally nondeterministic are excluded from semantic result equivalence but remain protected by artifact hashes.

## ADR-036 — Use a deny-by-default SQL subset with parser and engine enforcement

**Status:** Accepted.

Managed SQL must contain exactly one parsed read-only query. An AST allowlist admits only the explicitly tested relational subset and registered tables/functions. Mutation, DDL, multi-statement input, external scans, filesystem or network functions, extension operations, secrets, and unregistered catalogs are rejected before execution. DuckDB also runs in a separate resource-limited worker with external access disabled. Parser/engine grammar mismatches fail closed; syntax is enabled only with positive and bypass tests.

## ADR-037 — Represent metric applicability explicitly and make missing-value scoring a profile policy

**Status:** Accepted.

Every raw metric records its value or `null`, applicability reason, direction, range, numerator, and denominator where applicable. Episodes explicitly declare whether an empty ground-truth set represents a benign scored case. Scoring profiles choose `reject`, `renormalize`, or `zero` for non-applicable dimensions; the policy is versioned with the weights. A result must never silently coerce missing resilience or reproducibility into a successful score.

## ADR-038 — Treat listed seeds as repetitions and require paired comparison cells

**Status:** Accepted.

A benchmark run cell is deployment × episode × listed seed × declared configuration. The seed list defines the repetitions. If a legacy `repetitions` field is present, it must equal the number of unique listed seeds or validation fails. Pairwise comparisons match deployments by episode, seed, and configuration; missing or failed cells are reported and are not silently replaced.

## ADR-039 — Generate fixtures reproducibly with pinned Parquet canonicalization

**Status:** Accepted.

Synthetic fixtures are generated from versioned source definitions and explicit seeds. Generators pin relevant dependencies, sort rows and stable identifiers deterministically, fix schemas and timestamp encodings, and suppress or normalize writer metadata that would vary between runs. Regeneration must produce byte-identical public and private artifacts, and official fixtures record generator, source, and output hashes.

## ADR-040 — Distinguish trusted, measured, and self-reported resource usage

**Status:** Accepted.

Duration, process status, managed tool calls, rows, messages, and artifact sizes are measured by HuntEval. Token counts and provider cost are trusted only when obtained from a configured verifiable adapter; otherwise they are labeled self-reported or unavailable. Reports and scoring profiles preserve this provenance and cannot apply a hard cost or token constraint to an unverifiable value without marking the run non-comparable.

## ADR-041 — Add schema version 0.4 without rewriting 0.3 artifacts

**Status:** Accepted.

Schema version 0.4 adds benchmark execution, timeline, comparison, and report-source contracts. Version 0.3 artifacts remain immutable and readable through explicit compatibility adapters. Readers reject unknown newer or incompatible versions with typed errors. Adaptation normalizes an older source into current in-memory types and never rewrites the stored source. Fields unavailable in 0.3 remain absent or explicitly not applicable; they are never inferred from free-form text.

## ADR-042 — Derive stable benchmark cell identity from resolved inputs

**Status:** Accepted.

A benchmark cell key contains the benchmark identifier, deployment identifier and configuration digest, episode identifier and package digest, seed, scoring-profile identifier and digest, and an optional fault-profile identifier and digest. Canonical field order and exact lowercase SHA-256 text define the identity input. Paths, timestamps, host names, and attempt numbers are excluded. The cell identifier is the SHA-256 digest of the canonical JSON key prefixed with `cell:`. Attempt identifiers are independent opaque identifiers, so retries never overwrite history.

## ADR-043 — Use an append-only benchmark journal and deterministic projection

**Status:** Accepted.

`benchmark-events.jsonl` is the authoritative append-only transition journal. `benchmark-state.json` is an atomically replaced deterministic projection. Cells move through `pending`, `running`, `completed`, `failed`, or `non_comparable`; terminal attempt events are never edited. Resume records an interrupted attempt before starting a new attempt. One local controller owns a benchmark journal through a bounded lock acquisition that fails with a typed error. Replaying the same valid journal must reproduce byte-equivalent normalized state.

## ADR-044 — Evaluate only trusted normalized artifacts

**Status:** Accepted.

Metric code consumes a trusted evaluation view reduced from validated trajectory events, the structured final submission, HuntEval-observed resource measurements, and evaluator-only ground truth. Filesystem parsing, protocol replay, and infrastructure adapters remain outside metric modules. Run metrics and benchmark metrics use separate contracts. Every metric retains applicability, range, direction, numerator, denominator, edge behavior, and provenance.

## ADR-045 — Require typed sources for report claims

**Status:** Accepted.

Every normalized or rendered report conclusion references at least one validated source: a metric JSON pointer, trajectory sequence, run or benchmark cell identifier, constraint result, statistical comparison identifier, or verified artifact digest. Reporting receives validated DTOs, never private ground truth, and does not reinterpret free-form deployment text as an evaluator conclusion. Invalid, private, missing, or cross-run references fail closed.

## ADR-046 — Use one canonical quality entrypoint locally and in GitHub Actions

**Status:** Accepted.

Repository-owned scripts define formatting, linting, testing, documentation, dependency, architecture, and source-size checks. Local development and GitHub Actions invoke those scripts rather than maintaining separate command lists. Workflow configuration may select jobs, caching, and bounded artifacts, but cannot weaken or redefine a quality check.

## ADR-047 — Define the supported Linux sandbox capability contract

**Status:** Accepted.

Scored execution supports a named Linux Bubblewrap backend only after executable probes establish the required user, PID, mount, IPC, UTS, cgroup, and network namespaces, read-only mounts, process-tree cleanup, and resource-limit support. Executable presence alone is not sufficient. Unsupported hosts fail before episode data is delivered. R3 permits only denied deployment network access; any future exception requires a new versioned policy and threat-model review.

## ADR-048 — Use one sandbox adapter and a supervised process tree

**Status:** Accepted.

An infrastructure-only `hunteval-sandbox` crate owns Linux process construction, bounded diagnostics, lifecycle supervision, capability probing, and operating-system resource limits. Runner and managed-worker adapters depend on it; domain and protocol crates do not. The implementation uses audited safe APIs and external sandbox primitives without first-party `unsafe`. Termination must remove the complete sandbox process tree.

## ADR-049 — Make operating-system execution policy explicit and hashed

**Status:** Accepted.

Schema version 0.5 defines an execution policy containing the sandbox backend, denied network mode, wall time, CPU time, address-space cap, file-size cap, open-file cap, process cap, and stdout/stderr bounds. Hardened runs bind exact policy bytes and the sandbox launcher digest into resolved provenance. Older schemas remain readable, but scored execution requires an explicitly selected and hashed compatibility policy; no implicit default or silent downgrade is permitted.

## ADR-050 — Separate deterministic protocol regression from exploratory fuzzing

**Status:** Accepted.

Property tests and retained minimized corpora run deterministically in ordinary CI. `cargo-fuzz` targets remain outside the release workspace, use a pinned tool version, bounded inputs, and bounded smoke campaigns. Longer exploratory campaigns are scheduled or manual. Every discovered failure receives a minimized public regression input and stable expected outcome before closure.

## ADR-051 — Define standalone run-verification levels and reason codes

**Status:** Accepted.

Public run verification checks safe paths, supported versions, run identity, exact declared hashes, required artifacts, trajectory replay, terminal state, submission equivalence, and normalized-result consistency without a provider or private ground truth. It reports completed, incomplete, malformed, unsupported, and tampered states with stable path-safe reason codes. Public verification never claims to have recomputed private evaluation metrics.

## ADR-052 — Centralize bounded redaction and secret scanning

**Status:** Accepted.

One infrastructure boundary performs deterministic bounded redaction for process diagnostics, configuration values, report metadata, fixtures, snapshots, CI artifacts, and release inputs. Structured values are removed before serialization where possible. Secret findings contain only stable rule identifiers, safe relative locations, and one-way fingerprints; matched values are never emitted. Secret scanning is a release gate and does not replace external secret management.

## ADR-053 — Add immutable benchmark-science contracts

**Status:** Accepted.

Schema version 0.6 adds benchmark-science artifacts for episode classification and review, statistical policy and calibration, contribution validation, deployment topology, and controlled topology experiments. Schemas 0.3 through 0.5 remain immutable. Readers use explicit compatibility behavior, writers emit only their current authored form, and unknown versions or fields fail closed.

## ADR-054 — Separate public episode classification from private review evidence

**Status:** Accepted.

Public episode metadata uses bounded taxonomies for difficulty, required capabilities, and investigation shape. Ground truth, reference queries, expected recovery details, reviewer notes, and hidden partition membership remain private. A review record binds exact public, private, and reference-query hashes. Any bound-byte change invalidates approval.

## ADR-055 — Make statistical claim policy explicit and versioned

**Status:** Accepted.

A content-addressed statistical policy defines comparison class, minimum paired samples, interval and effect-size methods, multiplicity handling, calibration policy, and permitted claim strength. Results below the declared threshold remain descriptive and cannot become conclusive. Missing pairs and unavailable metrics are never imputed.

## ADR-056 — Use one bounded contributor service behind the CLI

**Status:** Accepted.

Episode scaffolding, validation, documentation, and review-bundle generation use application services behind filesystem ports. Scaffolding writes only beneath a validated new root and never overwrites. Validation is read-only. Public results contain safe reason codes and hashes but no ground truth, private paths, reference answers, or reviewer notes.

## ADR-057 — Represent deployment topology as a normative artifact

**Status:** Accepted.

A framework-neutral schema 0.6 topology artifact declares agents, roles, specialization, model assignments, delegation and coordination relationships, memory boundaries, task allocation, execution pattern, and critic/reviewer roles. Protocol registration must conform to the artifact but does not replace it. Single-agent topology is a first-class baseline.

## ADR-058 — Prove control-variable equivalence for topology experiments

**Status:** Accepted.

A controlled topology experiment binds baseline and candidate topology hashes, paired cells, exact changed variables, and hashes for every declared control. Episode, seed, budgets, models, managed-tool policy, scoring profile, execution policy, schemas, and relevant binaries remain equal unless explicitly experimental. Undeclared drift makes the comparison ineligible.

## ADR-059 — Separate topology observation from experimental contribution

**Status:** Accepted.

Investigation quality, coordination overhead, duplicate work, evidence propagation, task allocation, parallelism, utilization, resilience, and verified resources remain separate observable dimensions. Observational traces do not establish contribution. Marginal-agent and role contribution require a controlled ablation and are always labeled experimental and topology-dependent.

## ADR-060 — Add immutable evidence-backed diagnosis contracts

**Status:** Accepted.

Schema version 0.7 adds bounded artifacts for diagnostic taxonomies, typed source references, failure classifications, run diagnoses, recurrence, bottleneck observations and analysis, controlled contribution analysis, normalized diagnostic reports, and diagnostic bundle manifests. Schemas 0.3 through 0.6 remain immutable. Readers use explicit compatibility behavior, writers emit only the current authored form, and unknown versions, fields, variants, references, confidence levels, and applicability states fail closed.

## ADR-061 — Separate reviewable taxonomy metadata from executable classifiers

**Status:** Accepted.

A content-addressed taxonomy defines stable codes, categories, safe descriptions, required source kinds, and evidence-sufficiency requirements. Executable classification remains typed Rust in the evaluation boundary; taxonomy data cannot contain expressions or scripts. The compiled registry and taxonomy must agree exactly on identifiers and versions. Changing taxonomy bytes or classifier semantics changes artifact identity and invalidates prior reproduction claims.

## ADR-062 — Make confidence an evidence-sufficiency label

**Status:** Accepted.

Diagnostic confidence is the bounded evidence-sufficiency level `direct`, `corroborated`, or `controlled`, never a model-generated probability. A level cannot exceed the strongest complete rule-specific evidence set. Controlled confidence additionally requires an eligible controlled experiment. Missing required evidence omits the classification rather than producing a speculative low-confidence result.

## ADR-063 — Resolve every diagnostic attribution against verified artifacts

**Status:** Accepted.

Diagnostic sources are tagged, typed, bounded references to exact verified run, event, agent, action, task, evidence, finding, metric, benchmark cell, comparison, topology experiment, or artifact identities. Resolution verifies digests, ownership, same-run scope, ordering, and referential integrity. A generic safe artifact reference is permitted only when no stronger typed reference exists. Public diagnosis cannot contain ground-truth identifiers, private paths, private hashes, or inferred private reasoning.

## ADR-064 — Treat diagnostic reports as deterministic projections

**Status:** Accepted.

Normalized JSON is the machine-readable diagnostic source of truth and static HTML is a safe deterministic projection. Reports keep observations, classifications, unvalidated hypotheses, controlled experiment results, and approved changes in explicit stages. R5 writers cannot emit an approved change; that state requires a future R6 approval artifact. Every displayed conclusion resolves to an included source, untrusted text is escaped, and active content is prohibited.

## ADR-065 — Separate recurrence from causal contribution

**Status:** Accepted.

Repeated classifications across comparable cells establish recurrence only. Agent or role contribution requires an eligible R4 controlled topology experiment, an exact changed-variable inventory, paired observations, and the versioned statistical policy. Contribution remains experimental and topology-dependent. Observational recurrence, proximity, message wording, or role labels cannot create a causal contribution claim.

## ADR-066 — Derive bottlenecks from runner-authoritative intervals

**Status:** Accepted.

Queueing, task execution, managed-tool wait, active-agent, and idle intervals derive from runner-authoritative trajectory order and timestamps. Overlaps are unioned deterministically; negative, reversed, missing, cross-run, or ambiguous intervals fail or become explicitly unavailable according to the metric contract. Agent-reported timing is never upgraded to measured timing, and bottlenecks remain separate from investigation quality and verified resource costs.

## ADR-067 — Add immutable controlled-improvement contracts

**Status:** Accepted.

Schema version 0.8 adds bounded artifacts for registration, structural diff, improvement policy, controlled experiment, equivalence, validation, recommendation lifecycle, human decision, external adoption record, prompt/configuration weakness taxonomy, normalized report, and bundle verification. Schemas 0.3 through 0.7 remain immutable. Unknown versions, fields, variants, states, references, policies, and applicability values fail closed.

## ADR-068 — Register exact bytes and compare only declared structure

**Status:** Accepted.

Every experimental artifact is identified by exact bytes, kind, media type, and SHA-256 digest. A bounded versioned section inventory is required for structural comparison. Opaque legacy artifacts remain valid provenance but are ineligible for a structural experiment. Diff operations use typed section identifiers; inferred Markdown structure and arbitrary filesystem patches are not normative evidence.

## ADR-069 — Put immutable safety policy outside candidate authority

**Status:** Accepted.

The versioned improvement policy owns the complete immutable class inventory: authorization, tool access, filesystem, network, data handling, ground-truth isolation, benchmark constraints, output integrity, and security controls. A candidate cannot redefine mutability. Missing, changed, removed, renamed, reclassified, or ambiguously parsed immutable content makes the candidate ineligible before scored execution.

## ADR-070 — Separate candidate selection from hidden-test evaluation

**Status:** Accepted.

An evaluator-only content-addressed partition policy controls training, validation, and hidden-test membership. Candidate generation and selection cannot receive hidden membership, metrics, failures, or episode-level feedback. A frozen candidate may receive one sealed final assessment for release or adoption review; that result cannot feed another candidate in the same lineage.

## ADR-071 — Reuse benchmark journals for controlled paired experiments

**Status:** Accepted.

R6 resolves baseline and candidate matrices into the existing benchmark cell and attempt model. Experiment journals add scoped transitions and references without replacing or modifying benchmark journals. Existing pairing, resume, failure, non-comparability, statistics, sandbox, and verification semantics remain authoritative.

## ADR-072 — Make recommendation state append-only and digest-bound

**Status:** Accepted.

Recommendation lifecycle events form an append-only hash chain and project deterministically to current state. Only explicit transitions are valid. A change to candidate or controlling bytes invalidates prior validation and downstream approval or adoption eligibility without rewriting history.

## ADR-073 — Separate experimental validation, human approval, and adoption

**Status:** Accepted.

A passing controlled decision can support `validated` but cannot approve or adopt a candidate. A human decision binds exact recommendation, candidate, experiment, validation, policy, reviewer identifier, and UTC time. HuntEval never edits the active deployment; `adopted` records a separately confirmed external action against the approved digest.

## ADR-074 — Treat prompt recommendations as bounded hypotheses until controlled support

**Status:** Accepted.

A content-addressed reviewable taxonomy maps exact R5 classifications and observable source families to candidate prompt/configuration weaknesses. Executable mapping remains compiled typed Rust. Suggested changes cite exact evidence and sections. Observational traces alone cannot produce validation.

## ADR-075 — Keep suggested patches separate and non-authoritative

**Status:** Accepted.

A suggested-change artifact may describe bounded typed operations against mutable sections, but generation never writes into a registered baseline or deployment tree. A suggestion becomes testable only after an explicit materialization step creates new bytes, registration assigns a new digest, safety validation passes, and a new controlled experiment binds the candidate.

## ADR-076 — Add immutable knowledge and extension contracts

**Status:** Accepted.

Schema version 0.9 adds bounded analytical corpus, index, query, result, retrieval-audit, extension manifest, capability policy, resolution, conformance, and SDK compatibility artifacts. Schemas 0.3 through 0.8 remain immutable. Unknown versions, fields, variants, source classes, capabilities, and states fail closed.

## ADR-077 — Separate evaluator analytics from deployment-visible knowledge

**Status:** Accepted.

Every R7 corpus has exactly one authorization scope. Evaluator analytical artifacts are never exposed through deployment sessions or managed retrieval tools. Deployment-visible corpora contain only explicitly authored verified documents and retain the existing untrusted-input and disabled-by-default controls. Mixed scopes fail validation.

## ADR-078 — Index only verified content-addressed public artifacts

**Status:** Accepted.

Analytical corpus membership binds a stable source identity, kind, exact SHA-256 digest, and successful public verification. Index construction requires exact inventory agreement, bounded normalized fields, deterministic ordering, and a content-addressed manifest. Digest drift, unsupported artifacts, private material, symlinks, and incomplete verification are rejected before indexing.

## ADR-079 — Make analytical answers typed deterministic projections

**Status:** Accepted.

Analytical queries use a bounded versioned vocabulary over declared source kinds and verified fields. Results retain source identity, artifact digest, field, and bounded excerpt. They cannot create metrics, causal claims, experimental validation, approval, or adoption absent from source artifacts. Query/result use is recorded through hash-linked retrieval audit events with measured latency and explicit cost availability.

## ADR-080 — Use out-of-process versioned extension contracts

**Status:** Accepted.

Managed-tool and deployment adapters declare exact executable identity, supported versions, requested capabilities, denied network, tool inventory, and resource limits. Third-party execution remains out of process behind the existing sandbox and supervisor. HuntEval does not expose a stable Rust ABI or load third-party libraries into its process.

## ADR-081 — Keep capability policy and scored-tool authority in HuntEval

**Status:** Accepted.

An extension manifest requests capabilities but never grants them. Runner-owned deny-by-default policy resolves requests against explicit limits and rejects every undeclared or excessive capability. Scored tools remain runner-mediated; neither deployment adapters nor SDK clients may execute them directly. Scored network access remains denied before v1.0.

## ADR-082 — Build the Python SDK from normative contracts

**Status:** Accepted.

The pure Python SDK provides strict public contract models, bounded local readers, authored builders, and a deployment-side JSONL peer. It does not implement evaluation, scoring, authoritative verification, runner orchestration, provider access, or direct scored tools. The Rust core has no dependency on Python or Python packaging.

## ADR-083 — Prove cross-language compatibility with canonical fixtures

**Status:** Accepted.

Rust and Python validate the same immutable schema and protocol fixture inventory. Compatibility follows normative schema/protocol semantics and canonical vectors rather than implementation object layout. Python supports version 3.11 or newer for the initial R7 package; changing language or contract support requires an updated compatibility index and package version.

## ADR-084 — Keep framework connectors as optional protocol peers

**Status:** Accepted.

Framework packages remain outside the Rust core and base Python dependency set. Connectors depend only on a small structural interface and translate observable activity into the existing deployment protocol. Framework-native tools cannot replace HuntEval-managed scored tools.

## ADR-085 — Use one bounded framework lifecycle vocabulary

**Status:** Accepted.

CrewAI, LangGraph, AutoGen, Google ADK, Semantic Kernel, and generic MCP clients share task, delegation, tool, evidence, finding, resource, and terminal operations. Framework-only state is not promoted into a normative universal model.

## ADR-086 — Preserve unavailable framework observations

**Status:** Accepted.

Connectors emit a topology or resource observation only when the framework supplies an authoritative observable event. Display text, hidden state, scheduling assumptions, and private reasoning never create inferred metrics.

## ADR-087 — Isolate framework dependencies and support claims

**Status:** Accepted.

The SDK uses dependency-free structural adapters. Exact framework packages are installed separately and become supported only after their exact versions pass the published conformance matrix. Fixture conformance alone is labeled implemented, not release-complete.

## ADR-088 — Require deterministic doubles before provider smoke tests

**Status:** Accepted.

Every connector first passes provider-free lifecycle, malformed-input, process-failure, replay, and package-isolation fixtures. Provider-backed smoke tests are optional, non-scored, secret-safe, and cannot replace deterministic evidence.

## ADR-089 — Make MCP a bounded adapter, never an authority

**Status:** Accepted.

The MCP surface is one run-bound local session with a fixed `hunteval.*` tool catalog. Sampling, elicitation, roots, server-provided prompts, arbitrary resources, dynamic tools, and remote transports remain unavailable. MCP calls retain the deployment protocol, runner policy, budgets, and managed-tool authority.

## ADR-090 — Keep commercial transport out of the evaluation core

**Status:** Accepted.

Commercial transport policy and adapters live in the infrastructure-only `hunteval-commercial` crate. Domain, evaluation, scoring, and reporting crates have no dependency on vendor schemas, HTTP clients, authentication libraries, or credentials.

## ADR-091 — Represent credentials only by opaque references

**Status:** Accepted.

Policies and public audit artifacts contain a validated secret-reference identity and its one-way hash, never a token, cookie, password, client secret, or authorization header. Runtime resolution requires a future supervised worker channel.

## ADR-092 — Expose finite read-only operation catalogs

**Status:** Accepted.

Each platform has a typed allowlist. URL, method, origin, headers, credentials, and mutation operations are not agent-controlled request fields. Cross-platform and undeclared operations fail before transport execution.

## ADR-093 — Make deterministic replay the required offline baseline

**Status:** Accepted.

Synthetic fixtures bind exact request and response hashes. Changed request arguments require a new reviewed fixture. Offline replay performs no DNS, socket, credential, provider, or tenant access.

## ADR-094 — Separate live conformance from ordinary CI

**Status:** Accepted.

Live-read-only conformance requires protected environments, explicit approval, least-privilege non-production credentials, and bounded attestations. Forks and untrusted branches receive neither credentials nor live execution.

## ADR-095 — Keep remote observations distinct from asserted evidence

**Status:** Accepted.

Vendor records and classifications remain untrusted source-provenanced observations. An evaluated deployment must explicitly cite them before they become submission evidence. Vendor output is never HuntEval ground truth.

## ADR-096 — Evaluate native agents only through documented exports

**Status:** Accepted.

Platform-native agent attribution requires stable public identifiers for actions, evidence, results, and terminal state. UI scraping, browser automation, private endpoints, and inferred hidden activity are prohibited; missing observations remain unavailable.

## ADR-097 — Defer production scored SIEM execution and mutation

**Status:** Accepted.

Pre-v1.0 commercial connectors permit deterministic replay and, after external enforcement exists, authorized non-scored live-read-only conformance. Production scored execution, response actions, remediation, containment, and every remote mutation remain unavailable.
