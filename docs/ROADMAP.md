# HuntEval roadmap

## 1. Purpose

This roadmap starts from the completed R2/v0.2 implementation baseline and describes the work required to turn the current MVP foundations into a complete, auditable, and stable evaluation platform. The existing R2.4 external-enforcement caveat remains as recorded below.

HuntEval evaluates a complete multi-agent threat-hunting deployment as its primary experimental unit. The evaluated system includes deployment topology, agent identities and roles, specialization, task delegation, coordination behavior, managed-tool usage, evidence production and propagation, resilience, resource consumption, reproducibility, and the final investigative outcome. A single-agent deployment remains a valid baseline. This definition does not require or privilege any agent framework, model provider, topology, or orchestration architecture.

It is outcome-based rather than date-based. A release is complete only when its exit criteria pass; version numbers do not imply a calendar commitment.

## 2. Current baseline

The repository currently provides:

- infrastructure-independent domain contracts;
- a versioned JSONL deployment protocol with deterministic replay;
- separate public and private episode loading;
- a constrained DuckDB worker over deterministic Parquet fixtures;
- a two-agent offline vertical slice;
- nine synthetic AWS, Azure, and Google Cloud episodes;
- scoring profiles, paired statistics, ranking primitives, and deterministic fault schedules;
- normalized run results plus static JSON and HTML run reports;
- optional corpus-scoped local knowledge retrieval;
- observable rule-based diagnosis and controlled experiment contracts.

Several of these are intentionally narrow foundations. The current delivery state is:

- the R2.1 CLI executes, resumes, inspects, and compares the complete local matrix; trusted-view replay, the R2-08 through R2-10 metric contracts, and registry-backed v0.4 scoring profiles with v0.3 compatibility are complete;
- event, entity, evidence completeness, task, utilization, graceful-degradation, attack-path, timeline, structured-conclusion, technique, duplicate-work, causally useful communication, measured-duration, verified-cost, and cross-run stability metrics exist;
- deterministic benchmark JSON and static HTML reports preserve comparisons, attribution, limitations, and exact artifact hashes;
- single-agent, two-agent, and supervisor-specialist reference deployments provide narrow topology foundations, but normative topology artifacts, controlled topology ablations, and topology-aware contribution analysis remain future R4.4 work;
- diagnosis and experiment safety contracts exist, but artifact registries, end-to-end validation workflows, broader evidence-backed classification, and the bounded prompt-analysis lifecycle remain future R5 and R6 work;
- canonical repository scripts and bounded least-privilege GitHub workflows are implemented; live merge and protected-tag enforcement awaits authorized administrator attestation.

## 3. Prioritization rules

Work is ordered by the following priorities:

1. **P0 — benchmark correctness:** complete the local benchmark loop before expanding integrations.
2. **P0 — security and reproducibility:** fail closed at every trust boundary and make every comparison independently verifiable.
3. **P1 — diagnostic usefulness:** add explanations only when they cite observable artifacts and do not imply hidden reasoning.
4. **P1 — contributor usability:** make datasets, deployments, and results straightforward to validate locally and in CI.
5. **P2 — extensibility:** add SDKs and adapters only after the stable core contracts and compatibility suite exist.

Every initiative must preserve Clean Architecture, typed errors, bounded untrusted input, explicit protocol and schema versions, and readable Rust source files. The existing quality gates remain mandatory for every merge.

Topology comparisons and future deployment rankings follow this hierarchy:

```text
objective measurements
  -> metric vector
       -> scoring profile
            -> optional aggregate score
                 -> ranking or comparison
```

The raw metric vector is authoritative. Any aggregate score is derived only from an explicit versioned scoring profile; no global score or implicit missing-value policy is introduced. Missing, unavailable, or unverifiable metrics are never silently converted to zero or otherwise imputed. Constraint-first ranking remains authoritative where defined. When deployments have materially different observable capabilities, reports state that limitation instead of inventing equivalent metrics.

## 4. Release sequence

### v0.2 — Complete local benchmark loop

**Objective:** turn the existing vertical slice and benchmark primitives into a complete offline workflow that can execute and compare deployments reproducibly.

The pull-request sequence, contracts, tests, and quality gates for all v0.2 initiatives are defined in `R2_IMPLEMENTATION_PLAN.md`.

#### R2.1 — Benchmark execution and resume (complete)

- add `benchmark run`, `benchmark resume`, `benchmark status`, and `benchmark compare` commands;
- execute the exact deployment × episode × seed × configuration matrix;
- persist cell state atomically and distinguish pending, running, completed, failed, and non-comparable cells;
- resume interrupted runs without silently replacing failed or missing cells;
- preserve per-cell configuration, dataset, protocol, schema, executable, and scoring-profile hashes.

**Exit criteria:**

- one command executes at least two deployments over all nine cloud episodes and multiple paired seeds;
- interruption and resume are covered by deterministic integration tests;
- a second execution with identical inputs produces equivalent normalized artifacts;
- comparison rejects unpaired or non-equivalent cells with typed errors.

Completion evidence: the public CLI completed both reference deployments over all nine cloud episodes and two paired seeds (36 cells), recovered a forcibly interrupted controller, reproduced equivalent definition, submission, metric, and aggregate-score artifacts, and rejected missing or digest-mismatched pairs.

#### R2.2 — Complete evaluation contracts (complete)

- add ordered attack-path accuracy;
- add timeline accuracy with versioned tolerances;
- score acceptable conclusions without requiring exact wording;
- separate evidence grounding from evidence completeness;
- add deterministic duplicate-work and useful-communication metrics;
- add cost-normalized efficiency only for measured or verified resource data;
- add cross-run reproducibility and stability metrics.

Each metric must specify range, direction, numerator, denominator, applicability, edge cases, normalization, and positive and negative fixtures before it can enter a scoring profile.

**Exit criteria:**

- every new metric has contract, serialization, edge-case, and deterministic replay tests;
- unsupported metrics remain `null` with an explicit applicability reason;
- scoring profiles cannot reference undefined or unverifiable metrics;
- no global score or implicit missing-value policy is introduced.

Completion evidence: R2-07 through R2-11 reduce verified run artifacts into trusted evaluation inputs; implement investigation, conclusion, evidence, coordination, efficiency, and stability contracts; aggregate exact seed sets with explicit unavailable repetitions; and normalize registry-backed v0.4 scoring profiles while preserving immutable v0.3 compatibility. The implementation is traceable through commits `0842918`, `2e10b1d`, `a730cd4`, and `c8c8c3b`. Comparative result normalization and rendering remain in R2.3.

#### R2.3 — Comparative reporting (complete)

- render normalized benchmark JSON as the source of truth;
- add portable static HTML benchmark reports;
- include metric vectors, constraints, sample counts, intervals, paired differences, wins/ties/losses, and inconclusive labels;
- add run timelines, task/agent attribution tables, and coordination summaries;
- link every conclusion to a metric, trajectory event, or comparison cell;
- display artifact hashes and provenance without exposing private paths or secrets.

**Exit criteria:**

- reports render incomplete and non-comparable benchmarks without overstating results;
- all untrusted fields are escaped and no active script is required;
- report generation is deterministic and snapshot-tested;
- artifact links and hashes are validated before rendering.

Completion evidence: R2-12 through R2-15 are implemented in `006519d`. The normalized v0.4 benchmark result retains cell status, raw metrics, score omissions, constraints, verified resource provenance, paired statistics, constraint-first ranking groups, typed claim sources, limitations, and exact artifact digests. Deterministic JSON is the machine-readable source of truth; portable HTML contains no active content; the unified CLI generates atomically and detects missing, stale, oversized, symlinked, or modified artifacts during verification.

#### R2.4 — GitHub delivery hardening (implementation complete; external enforcement pending)

- make GitHub Actions invoke the same repository-owned quality entrypoints used locally;
- cache Rust and DuckDB native build outputs safely;
- upload test and benchmark artifacts with bounded retention;
- document protected branches, required approvals, and release permissions.

**Exit criteria:** the same revision receives equivalent pass/fail results locally and in GitHub Actions, and required checks protect the default branch.

Implementation evidence: commits `5874792`, `23deef2`, and `5a38c63` provide pinned canonical local/CI gates, a pinned Bubblewrap-capable runner, runner-image-bound caches, fail-closed security capability checks, negative failure-propagation tests, bounded verification artifacts, clean-cache parity, CODEOWNERS, a live-settings verifier, and a non-publishing checksummed RC workflow. Local acceptance gates and RC package verification pass. GitHub Actions run `31255365813` passed all seven canonical jobs, including the uncached Package job. R2.4 remains open until an authorized administrator records that the required checks, default-branch protection, and protected RC tag rules are active and the protected RC tag dry run passes.

### v0.3 — Runner and protocol hardening

**Objective:** make process isolation, protocol handling, and artifact integrity robust enough for untrusted deployment implementations.

The pull-request sequence, contract decisions, tests, risks, and release gates for R3.1 through R3.3 are defined in `R3_IMPLEMENTATION_PLAN.md`.

Current state: the R3 implementation exists locally, but R3 remains the active milestone until a dedicated revision passes every local and GitHub Actions release gate. No completion evidence is claimed here.

#### R3.1 — Isolation backends and resource enforcement

- formalize the Linux sandbox backend and its required host capabilities;
- enforce process-tree termination, memory, CPU, file-size, descriptor, and wall-clock limits at the operating-system boundary;
- keep deployment network access denied by default and make any exception explicit and hashed;
- document unsupported platforms and provide a fail-closed capability check;
- verify that workers and deployment processes cannot access private episode roots.

#### R3.2 — Adversarial protocol testing

- add property tests and fuzz corpora for framing, state transitions, replay, identifiers, malformed JSON, oversized messages, and hash chains;
- test slow writers, early EOF, duplicate and future references, message floods, invalid UTF-8, and process crashes;
- define protocol compatibility fixtures for every supported minor version;
- add a conformance command for third-party deployments.

#### R3.3 — Artifact verification and redaction

- add a standalone `run verify` command;
- verify manifests, exact-byte hashes, trajectory chains, result consistency, and schema compatibility;
- centralize bounded redaction for environment values, stderr, configuration fields, and report metadata;
- add secret scanning to fixtures, snapshots, logs, and generated reports;
- produce a machine-readable verification result.

**Release exit criteria:**

- negative isolation tests run in CI on the supported Linux environment;
- fuzz/property suites retain regression seeds for every discovered defect;
- tampered or partially written runs fail verification with typed reason codes;
- no deployment failure can terminate the benchmark controller or expose ground truth.

### v0.4 — Benchmark science and dataset quality

**Objective:** improve validity, coverage, and reviewability of the benchmark itself.

#### R4.1 — Episode coverage expansion

- add explicitly benign scored episodes to lock empty-ground-truth semantics;
- add multi-stage attack paths and longer timelines for each provider;
- add cross-account/project/tenant cases and ambiguous benign alternatives;
- define difficulty and capability tags without exposing answers;
- require independent security review for new ground truth and reference queries.

#### R4.2 — Statistical policy

- define minimum paired sample requirements for ranking claims;
- add effect sizes, stability summaries, and multiple-comparison policy where applicable;
- distinguish exploratory, validation, and hidden-test comparisons;
- add calibration checks for confidence values and finding severity;
- document when results are descriptive rather than statistically conclusive.

#### R4.3 — Dataset contribution tooling

- add a contributor command to scaffold and validate an episode package;
- validate provider schemas, stable identifiers, deterministic generation, leakage, and reference-query recovery;
- generate public package documentation from validated metadata;
- produce review bundles that never include private ground truth in deployment-visible artifacts.

#### R4.4 — Multi-agent topology benchmarking

- represent deployment topology explicitly in normative versioned artifacts, including agent identities, roles, specialization, delegation and coordination relationships, model assignments, memory boundaries, task-allocation policy, execution pattern, and critic or reviewer roles;
- compare single-agent and multi-agent deployments, including supervisor/worker, hierarchical, peer-to-peer, centralized, decentralized, homogeneous-model, heterogeneous-model, shared-memory, isolated-memory, static-allocation, dynamic-allocation, sequential, parallel, specialist, generalist, critic-enabled, and critic-free configurations;
- execute controlled paired comparisons in which episode, seed, budgets, models, managed-tool policy, scoring profile, and other relevant variables remain equivalent unless the experiment manifest explicitly declares them as changed variables;
- analyze marginal benefit from additional agents, coordination overhead, redundant specialization, duplicate work, evidence propagation, task allocation, parallelism, agent utilization, role-specific contribution, topology-level resilience, and cost/quality trade-offs as separate observable dimensions;
- support controlled topology ablations such as removing an agent, replacing a specialist with a generalist, disabling a critic, changing shared memory to isolated memory, or switching static delegation to dynamic delegation;
- treat observational traces as insufficient for strong causal claims; label every contribution estimate from a controlled ablation as experimental and topology-dependent;
- preserve the authoritative raw metric vector, explicit applicability, constraint-first ranking, and versioned scoring-profile semantics for every topology comparison.

**Release exit criteria:**

- every episode class has deterministic reference recovery and leakage tests;
- benchmark versions identify exact episode membership and scoring profiles;
- result reports expose sample size and uncertainty for every comparative claim;
- fixture regeneration remains byte-identical under the pinned toolchain;
- deployment topology is explicit in normative versioned artifacts;
- single-agent, supervisor-worker, and supervisor-specialist reference deployments can be compared through the same paired benchmark matrix;
- topology comparisons preserve all declared control variables and record every changed variable;
- marginal-agent and controlled-ablation experiments produce auditable artifacts;
- coordination overhead and resource trade-offs are reported separately from investigation quality;
- unsupported topology metrics remain unavailable instead of being inferred;
- reports do not present role-specific or agent-specific performance as universally transferable across topologies.

### v0.5 — Evidence-backed diagnosis

**Objective:** expand diagnosis without requesting private reasoning or making unsupported causal claims.

#### R5.1 — Versioned taxonomy and attribution

- expand the taxonomy across investigation, evidence, tool use, coordination, resilience, and policy failures;
- attach classifications to affected runs, agents, actions, tasks, evidence, metrics, and artifact versions;
- define deterministic confidence levels based on evidence sufficiency rather than model speculation;
- omit classifications whose required observable evidence is absent.

#### R5.2 — Diagnostic reports

- add diagnostic sections to normalized and static reports;
- group recurrent failures across paired runs and deployments;
- distinguish observation, classification, recommendation hypothesis, experiment result, and approved change;
- link every recommendation to affected runs and exact observable sources.

#### R5.3 — Contribution and bottleneck analysis

- add controlled agent-ablation experiment manifests;
- report reassignment, queueing, duplicate work, idle time, and managed-tool bottlenecks;
- label contribution estimates as experimental and preserve changed topology information;
- avoid universal agent rankings that ignore assigned roles.

**Release exit criteria:** the same artifact set produces byte-equivalent rule-based diagnoses, unsupported claims are absent, and reports never contain or request private chain of thought.

### v0.6 — Controlled improvement workflow

**Objective:** make improvement hypotheses reproducible from artifact registration through human-approved validation.

#### R6.1 — Artifact registry and safe diffs

- register versioned deployment configuration and instruction artifacts by content hash;
- compare baseline and candidate configurations structurally;
- prove that exactly one declared experimental variable changed;
- reject changes to authorization, tool access, data handling, ground-truth isolation, or other immutable safety sections;
- detect benchmark-answer leakage in candidate artifacts.

#### R6.2 — Experiment orchestration

- execute paired baseline/candidate matrices over training and validation partitions;
- keep hidden-test results unavailable during candidate selection;
- evaluate quality, regression, resilience, resource, and verified-cost constraints;
- retain uncertainty and non-comparable cells in the decision artifact;
- require explicit human approval before adoption.

#### R6.3 — Recommendation lifecycle

- track recommendations as proposed, rejected, testing, validated, or adopted;
- preserve observable evidence, candidate diff, experiment manifest, results, and reviewer decision;
- never allow automated generation to alter immutable policies;
- require a new validation run when any candidate artifact changes.

#### R6.4 — Prompt improvement analysis

Prompt improvement analysis identifies observable failure patterns, attributes them only where evidence permits, inspects registered instruction and configuration artifacts, formulates a bounded improvement hypothesis, and may propose a candidate change for validation through R6.2. It never modifies or adopts deployment instructions autonomously.

```text
observable run failures
  -> failure classification
       -> agent or artifact attribution
            -> prompt inspection
                 -> candidate weakness
                      -> proposed prompt change
                           -> controlled A/B experiment
                                -> evidence-backed recommendation
                                     -> human review
```

- distinguish observation, classification, attribution, hypothesis, suggested change, experimentally supported change, and approved or adopted change in contracts and reports;
- require every recommendation to cite exact observable evidence, including affected run, trajectory event, task, tool action, finding, coordination event, and metric-delta references where applicable;
- bind recommendations to baseline and candidate artifact hashes and to the experiment manifest used for validation;
- define a versioned, reviewable prompt and configuration failure taxonomy covering at least role ambiguity, missing output contracts, missing evidence requirements, missing acceptance criteria, missing stopping conditions, unclear tool-use policy, insufficient error handling, insufficient delegation policy, duplicated responsibility, missing task ownership, missing conflict-resolution policy, excessive communication requirements, insufficient evidence-sharing rules, and overly broad specialist invocation criteria;
- represent future recommendations as structured auditable artifacts with an agent or deployment target, typed issue, evidence references, suspected weakness and bounded confidence, content-addressed target section, suggested operation and rationale, expected effects, validation requirement and experiment reference, and lifecycle status; the exact schema is deferred to the R6.4 implementation plan;
- never represent a change as causally validated from observational traces alone; validation requires a controlled baseline/candidate experiment with declared controls and a passing decision artifact;
- reject candidate changes to authorization, tool-access, filesystem, network, data-handling, ground-truth-isolation, benchmark-constraint, output-integrity, security-control, or other immutable safety and trust-boundary sections;
- scan every candidate for benchmark-answer leakage, keep hidden-test results unavailable during candidate generation and selection, and invalidate prior validation whenever candidate content changes;
- require explicit human approval before adoption; autonomous prompt adoption remains deferred until after v1.0 and is not part of the v1.0 commitment.

**Release exit criteria:**

- prompt recommendations cite exact observable artifact references;
- observational evidence alone cannot mark a recommendation as validated;
- candidate changes are content-addressed and structurally diffable;
- immutable policy sections cannot be modified through the improvement workflow;
- baseline/candidate experiments preserve and record declared control variables;
- hidden-test results cannot influence candidate generation or selection;
- recommendations distinguish proposed, testing, validated, rejected, and adopted states;
- no recommendation can be labeled validated without a passing controlled experiment;
- adoption requires explicit recorded human approval;
- changing candidate content invalidates every prior validation result.

### v0.7 — Knowledge and extension interfaces

**Objective:** add local analytical capabilities and external integrations without coupling them to the domain model or the scored MVP path.

#### R7.1 — Artifact-grounded local search

- index normalized reports and verified historical runs locally;
- answer structured questions with citations to run, event, metric, comparison, and document identifiers;
- preserve corpus authorization and distinguish benchmark knowledge from deployment-visible knowledge;
- record retrieval queries, citations, latency, and verified cost provenance.

#### R7.2 — Stable extension contracts

- define versioned managed-tool and deployment-adapter interfaces;
- add compatibility fixtures and conformance tests;
- keep external adapters outside the domain and evaluation cores;
- require explicit capability, network, filesystem, and budget declarations.

#### R7.3 — Python SDK

- provide typed builders and readers for manifests, protocol messages, run artifacts, and reports;
- generate models from normative schemas where practical;
- test Rust/Python serialization compatibility;
- keep orchestration authority and scored-tool execution in HuntEval.

**Release exit criteria:** third-party adapters and SDK clients pass the compatibility suite without introducing a dependency from the Rust core to a provider or agent framework.

### v0.8 — Release candidate

**Objective:** freeze and audit the interfaces intended for v1.0.

- publish a protocol and schema compatibility matrix;
- provide migrations or explicit rejection behavior for older artifacts;
- complete an independent security and reproducibility review;
- produce SBOMs, dependency audit results, checksums, and signed release artifacts;
- package the CLI and worker for supported platforms;
- document installation, operations, governance, disclosure, benchmark review, and release procedures;
- run the official benchmark pack from a clean environment using only published instructions.

**Exit criteria:** no unresolved P0 defect, no undocumented compatibility break, no failing quality or verification gate, and one reproducible release-candidate dry run from source checkout to signed reports.

### v1.0 — Stable evaluation platform

v1.0 declares stability for:

- domain, episode, deployment, run-result, and benchmark schemas;
- the process-neutral deployment protocol;
- managed DuckDB tool behavior and security policy;
- local benchmark execution and resume semantics;
- metric contracts, scoring profiles, statistics, and comparison labels;
- versioned multi-agent deployment topology representation and topology-aware comparative semantics;
- coordination metrics and topology-dependent attribution artifacts;
- controlled improvement experiment and recommendation artifacts;
- normalized reports and artifact verification;
- the documented compatibility and deprecation policy.

The v1.0 release must include an official versioned cloud benchmark pack, reproducible release artifacts, a security review, contributor governance, and complete operator documentation.

## 5. Dependency graph

```text
v0.2 complete benchmark loop
  -> v0.3 runner/protocol hardening
  -> v0.4 benchmark science
       -> R4.4 multi-agent topology benchmarking
            -> v0.5 evidence-backed diagnosis
                 -> v0.6 controlled improvements
                      -> R6.4 prompt improvement analysis

v0.3 stable contracts + v0.4 benchmark policy
  -> v0.7 knowledge and extension interfaces

v0.2 through v0.7 exit criteria
  -> v0.8 release candidate
       -> v1.0 stable platform
```

Security fixes, contract defects, leakage risks, and reproducibility failures preempt feature work at every stage.

## 6. Explicitly deferred until after v1.0

- production SIEM execution in scored mode;
- unrestricted network access for evaluated deployments;
- distributed benchmark scheduling and storage;
- a hosted leaderboard or multi-tenant control plane;
- a web dashboard;
- Kubernetes deployment;
- autonomous adoption of generated changes;
- causal claims based only on observational traces;
- collection or storage of private chain of thought.

Potential post-v1.0 work may include controlled SIEM-language adapters, signed public result registries, distributed execution, richer graph-based experimental attribution, and multimodal evidence. Each requires a separate threat-model and architecture review.

## 7. Roadmap governance

Every roadmap initiative must be delivered through a small reviewable pull request sequence. Each sequence must define:

1. objective and user-visible outcome;
2. affected contracts and compatibility;
3. security impact;
4. ground-truth-isolation impact;
5. positive tests;
6. negative tests;
7. malformed-input tests;
8. deterministic and replay tests;
9. exact quality gates and acceptance commands;
10. documentation and ADR changes;
11. migration behavior;
12. rollback behavior;
13. known limitations.

The future R4.4 and R6.4 implementation plans must address every item above before implementation begins; this roadmap does not substitute for those detailed plans.

An initiative is not complete while any required quality gate fails. Scope changes must update this roadmap and the corresponding implementation plan before implementation begins.
