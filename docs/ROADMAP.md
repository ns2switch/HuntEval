# HuntEval roadmap

## 1. Purpose

This roadmap starts from the implemented PR-00 through PR-15 baseline and describes the work required to turn the current MVP foundations into a complete, auditable, and stable evaluation platform.

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

Several of these are intentionally narrow foundations. The most important completeness gaps are:

- the R2.1 CLI executes, resumes, inspects, and compares the complete local matrix; trusted-view replay and the remaining evaluation contracts are now the next benchmark-completeness dependency;
- event, entity, evidence, task, utilization, and graceful-degradation metrics exist, while attack-path, timeline, conclusion, semantic coordination, efficiency, and cross-run reproducibility metrics remain incomplete;
- static run reports exist, but benchmark reports, timelines, coordination views, agent attribution, and artifact hashes are not yet complete;
- diagnosis and experiment safety contracts exist, but artifact registries, end-to-end validation workflows, and broader evidence-backed classification remain incomplete;
- the GitHub workflow runs the quality gates, but it still needs a shared local/CI entrypoint, hardened merge policy, bounded artifacts, and release automation.

## 3. Prioritization rules

Work is ordered by the following priorities:

1. **P0 — benchmark correctness:** complete the local benchmark loop before expanding integrations.
2. **P0 — security and reproducibility:** fail closed at every trust boundary and make every comparison independently verifiable.
3. **P1 — diagnostic usefulness:** add explanations only when they cite observable artifacts and do not imply hidden reasoning.
4. **P1 — contributor usability:** make datasets, deployments, and results straightforward to validate locally and in CI.
5. **P2 — extensibility:** add SDKs and adapters only after the stable core contracts and compatibility suite exist.

Every initiative must preserve Clean Architecture, typed errors, bounded untrusted input, explicit protocol and schema versions, and readable Rust source files. The existing quality gates remain mandatory for every merge.

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

#### R2.2 — Complete evaluation contracts

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

#### R2.3 — Comparative reporting

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

#### R2.4 — GitHub delivery hardening

- make GitHub Actions invoke the same repository-owned quality entrypoints used locally;
- cache Rust and DuckDB native build outputs safely;
- upload test and benchmark artifacts with bounded retention;
- document protected branches, required approvals, and release permissions.

**Exit criteria:** the same revision receives equivalent pass/fail results locally and in GitHub Actions, and required checks protect the default branch.

### v0.3 — Runner and protocol hardening

**Objective:** make process isolation, protocol handling, and artifact integrity robust enough for untrusted deployment implementations.

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

**Release exit criteria:**

- every episode class has deterministic reference recovery and leakage tests;
- benchmark versions identify exact episode membership and scoring profiles;
- result reports expose sample size and uncertainty for every comparative claim;
- fixture regeneration remains byte-identical under the pinned toolchain.

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

**Release exit criteria:** no recommendation can be labeled validated without a passing controlled experiment and recorded human review, and hidden-test feedback cannot influence candidate selection.

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
- normalized reports and artifact verification;
- the documented compatibility and deprecation policy.

The v1.0 release must include an official versioned cloud benchmark pack, reproducible release artifacts, a security review, contributor governance, and complete operator documentation.

## 5. Dependency graph

```text
v0.2 complete benchmark loop
  -> v0.3 runner/protocol hardening
  -> v0.4 benchmark science
       -> v0.5 evidence-backed diagnosis
            -> v0.6 controlled improvements

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
2. affected contracts and schema/protocol compatibility;
3. security and ground-truth-isolation impact;
4. positive, negative, malformed-input, and deterministic tests;
5. exact quality and acceptance commands;
6. documentation and ADR changes;
7. migration, rollback, and known limitations.

An initiative is not complete while any required quality gate fails. Scope changes must update this roadmap and the corresponding implementation plan before implementation begins.
