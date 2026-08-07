# R2 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R2.1 through R2.4 into a reviewable pull-request sequence. It covers:

- complete benchmark execution, resume, status, and comparison;
- the remaining evaluation contracts required for a defensible local benchmark;
- normalized comparative JSON and portable static HTML reporting;
- equivalent local and GitHub Actions quality gates.

This is a planning artifact. It does not authorize work outside R2 or relax any existing security, architecture, compatibility, or maintainability rule.

### Delivery status

Status values are evidence-based: `complete` means the milestone has a dedicated commit, its acceptance tests pass, and the complete workspace gate has passed. `planned` means no completion claim is made, even when reusable foundations already exist.

| Milestone | Status | Evidence or next dependency |
|---|---|---|
| R2-00 | complete | `d51d6fb` — v0.4 contracts, schemas, and accepted ADRs |
| R2-01 | complete | `46c056e` — stable resolved benchmark identities and v0.3 compatibility |
| R2-02 | complete | `1685b69` — independently executable reference deployment |
| R2-03 | complete | `cf789a1` — isolated generic mediated run engine |
| R2-04 | complete | `41108b6` — append-only benchmark journal and deterministic projection |
| R2-05 | complete | `7f1f076` — bounded matrix service, production DuckDB mediation, resume, and verified comparisons |
| R2-06 | complete | public benchmark CLI, documented exit codes, 36-cell smoke matrix, and interrupted recovery |
| R2-07 | planned | requires trusted artifacts from R2-03 |
| R2-08 | planned | requires R2-07 |
| R2-09 | planned | requires R2-07 |
| R2-10 | planned | requires R2-07 and benchmark aggregation |
| R2-11 | planned | requires R2-08 through R2-10 |
| R2-12 | planned | requires R2-06 and R2-11 |
| R2-13 | planned | requires R2-12 |
| R2-14 | planned | requires R2-13 |
| R2-15 | planned | requires R2-13 and R2-14 |
| R2-16 | planned | may evolve alongside remaining milestones; final acceptance follows R2-15 |
| R2-17 | planned | requires canonical scripts from R2-16 |
| R2-18 | planned | requires R2-15 and R2-17 |

The operational MVP cut line is the R2.1 exit gate after R2-06. It provides complete local benchmark execution, resume, status, and comparison. The full auditable v0.2/R2 release is not complete until R2-18 and the completion definition in section 12 pass.

## 2. Current baseline findings

The implementation audit after R2-06 established the following current starting points:

1. The public CLI validates, executes, resumes, inspects, and compares the complete deployment × episode × seed matrix.
2. Stable cell identities bind authored configuration, episode package, scoring profile, optional fault profile, seed, runtime binaries, and schema bytes.
3. The runner schedules deterministic batches with bounded concurrency, records every attempt in the hash-chained journal, retains failed and interrupted history, and rejects configuration drift.
4. Three external reference topologies execute inside the networkless Linux sandbox and request scored SQL only through the production HuntEval-owned DuckDB worker adapter.
5. Comparison eligibility requires complete pairs and re-verifies normalized result bytes against their journaled digests. Missing, failed, or tampered pairs remain explicitly ineligible.
6. The compatibility `run` command remains available for the original vertical-slice workflow; benchmark commands use the generic engine.
7. The evaluator still lacks a complete normalized trusted view reduced from verified artifacts. This is the next dependency at R2-07.
8. Attack-path, timeline, conclusion, evidence-completeness, deterministic coordination, verified efficiency, and cross-run stability contracts remain incomplete.
9. `BenchmarkReport` remains a serialization foundation. Static rendering supports run reports, not complete comparative benchmark reports.
10. GitHub Actions runs the mandatory gates, but no shared local/CI entrypoint, fail-closed sandbox capability job, hardened release workflow, or repository-settings checklist exists.

These gaps define the order below. Reporting must not invent data that benchmark execution and evaluation do not yet produce.

## 3. Mandatory delivery rules

Every R2 pull request must:

- preserve physical separation of private ground truth;
- execute scored tools only through HuntEval;
- treat deployment output, retrieved documents, report fields, manifests, and stored artifacts as untrusted input;
- record observable actions and concise reason codes, never private chain of thought;
- preserve provenance from agent through action, result, evidence, finding, submission, metric, and report claim;
- use typed errors and bounded reads, writes, messages, collections, and process output;
- avoid `unwrap()`, `expect()`, undocumented lint suppression, and production panics;
- keep the domain independent from CLI, filesystem, DuckDB, process, reporting, and provider implementations;
- split production Rust files before 300 lines where practical and never exceed the enforced 500-line limit;
- update schemas, contracts, examples, ADRs, tests, and user documentation in the same pull request as a behavioral change;
- remain English-only in project artifacts.

The following commands are mandatory before every R2 pull request is complete:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
./scripts/check-dependency-direction.sh
./scripts/check-source-size.sh
```

Additional acceptance commands are listed per pull request. A later pull request must not begin while an earlier dependency has a failing gate.

## 4. Architecture decisions to close first

R2 begins with a contract-only pull request that records these decisions as ADRs:

### ADR-041 — Additive schema v0.4

- Existing v0.3 artifacts remain readable and immutable.
- New benchmark execution, timeline, comparison, and report contracts use v0.4.
- Readers accept explicitly supported older minor schemas and reject unknown newer or incompatible schemas with typed errors.
- Migration is normalization into current in-memory types; stored source artifacts are never rewritten silently.
- Canonical examples and JSON Schemas are added under `schemas/v0.4/`.

### ADR-042 — Stable benchmark cell identity

A benchmark cell key is the canonical tuple:

```text
benchmark ID
+ deployment ID and configuration hash
+ episode ID and package hash
+ seed
+ scoring-profile ID and hash
+ optional declared fault-profile ID and hash
```

The stable cell identifier is derived from the canonical tuple. Paths and timestamps are not identity inputs. An attempt identifier is separate so retries never overwrite history.

### ADR-043 — Append-only benchmark state

- `benchmark-events.jsonl` is the authoritative append-only state transition journal.
- `benchmark-state.json` is an atomically replaced deterministic projection.
- Cell states are `pending`, `running`, `completed`, `failed`, and `non_comparable`.
- Resume records an interrupted attempt and starts a new attempt; it never changes a previous terminal event.
- Only one local benchmark controller may own the journal. Lock acquisition is bounded and fails with a typed error.

### ADR-044 — Evaluation from trusted normalized artifacts

- Metric code consumes a trusted evaluation view reduced from stored trajectory, final submission, resource measurements, and evaluator-only ground truth.
- Infrastructure parsing and replay remain outside metric modules.
- Run-level and benchmark-level metrics are separate contracts.
- Every metric retains applicability, range, direction, numerator, denominator, and edge behavior.

### ADR-045 — Typed report claim references

Every displayed conclusion references one or more typed sources:

- a normalized metric JSON pointer;
- a trajectory sequence number;
- a run or benchmark cell identifier;
- a constraint result;
- a statistical comparison identifier;
- a verified artifact digest.

Reporting receives validated DTOs and never reads private ground truth or interprets free-form deployment text as a conclusion.

### ADR-046 — One canonical quality entrypoint

Local development and GitHub Actions invoke the same repository-owned quality scripts. Workflow configuration selects jobs and artifacts but does not redefine the checks.

## 5. Target dependency direction

R2 keeps the existing Clean Architecture direction:

```text
hunteval-domain
  <- hunteval-protocol
  <- hunteval-evaluation
  <- hunteval-statistics
  <- hunteval-reporting
  <- hunteval-runner
  <- hunteval-cli
```

The actual graph remains non-linear: evaluation and statistics depend only on domain; reporting depends on domain and statistics; runner composes all outbound adapters; CLI depends only on runner.

One new binary crate is permitted for an independently executable reference deployment:

```text
hunteval-reference-deployment -> hunteval-domain + protocol message contracts
```

It must not depend on the runner, evaluator, private fixture loader, ground truth, or DuckDB implementation. The dependency checker must enforce this rule.

No additional crate is introduced unless a pull request demonstrates a measured cohesion or dependency-direction problem.

## 6. R2.1 — Benchmark execution and resume

### R2-00 — Freeze R2 contracts and ADRs

**Objective:** accept ADR-041 through ADR-046 and publish v0.4 schemas before implementation.

**Primary files:** `docs/ADR.md`, `docs/CONTRACTS.md`, `docs/METRICS_AND_RANKING.md`, `schemas/v0.4/*`, contract examples.

**Contracts:** `BenchmarkManifest`, `BenchmarkCellKey`, `BenchmarkCellId`, `BenchmarkAttemptId`, `BenchmarkEvent`, `BenchmarkState`, `CellStatus`, `ComparisonEligibility`, `DeploymentProcessConfig`, `TimelineEntry`, and typed report source references.

The v0.4 private ground-truth contract adds structured acceptable submission statuses and expected timeline windows. The deployment-visible submission contract adds optional structured timeline entries. A v0.3 run adapted into v0.4 has no timeline value, so timeline metrics remain explicitly not applicable rather than being inferred. Deployment process configuration contains an executable reference, fixed arguments, and allowlisted environment variable names; it cannot embed environment values or expand scored-tool authority.

**Tests and acceptance:**

- canonical v0.4 examples validate against schemas;
- v0.3 examples remain unchanged and continue to validate;
- unknown fields and unsupported versions fail closed;
- public episode and deployment schemas cannot represent ground truth.

```bash
cargo test -p hunteval-domain contracts
rg -n "Status: Proposed|TODO|TBD" docs/ADR.md docs/CONTRACTS.md docs/METRICS_AND_RANKING.md
```

### R2-01 — Domain benchmark identities and compatibility reader

**Objective:** separate the human-authored filesystem manifest from the resolved infrastructure-independent benchmark definition while preserving the existing runner API through an explicit compatibility re-export.

**Primary files:** `hunteval-domain/src/benchmark/*`, `hunteval-domain/src/id.rs`, `hunteval-runner/src/benchmark/manifest.rs`, schema compatibility tests.

**Implementation notes:**

- keep `AuthoredBenchmarkManifest` and its safe relative artifact references in the runner boundary;
- resolve it into a domain `BenchmarkDefinition` containing typed IDs, hashes, seeds, and policy values but no filesystem paths;
- replace free-form schema strings with `SchemaVersion`;
- resolve and validate manifest paths in the runner, never in domain code;
- reject duplicate deployments, episodes, and seeds;
- canonicalize ordering before deriving cell identifiers;
- include configuration and artifact digests in cell identity;
- keep v0.3 manifests readable through a documented adapter.

**Tests:** identity stability, ordering independence, duplicate rejection, unsafe paths, digest changes, v0.3 adaptation, v0.4 round trips, unknown-version rejection.

```bash
cargo test -p hunteval-domain benchmark
cargo test -p hunteval-runner benchmark_manifest
```

### R2-02 — Independently executable reference deployment

**Objective:** replace embedded reference transcripts with a real JSONL protocol peer suitable for local conformance and benchmark tests.

**Primary files:** new `hunteval-reference-deployment` binary, deployment manifests, protocol fixtures, dependency policy.

**Implementation notes:**

- one binary may implement the three reference topologies through immutable manifest configuration;
- it receives only runner-visible episode data and managed tool results;
- it never links to fixture generators, package loaders, evaluator code, or private types;
- provider-specific investigation logic uses documented public tables and normalized views;
- stdout is protocol-only; bounded operational diagnostics use stderr;
- deterministic fixtures use the runner-provided seed.

**Tests:** handshake and terminal submission for all three topologies, managed-tool-only behavior, no network request, malformed runner input, early EOF, deterministic seed behavior, dependency-direction rejection.

```bash
cargo test -p hunteval-reference-deployment
cargo test -p hunteval-protocol --test topology_conformance
./scripts/check-dependency-direction.sh
```

### R2-03 — Generic mediated run engine

**Objective:** extract a provider-neutral, deployment-neutral run application service from the hard-coded vertical slice.

**Primary files:** `hunteval-runner/src/run/*`, process adapter, managed-tool router, artifact reducer, existing vertical-slice wrapper.

**Public application types:** `RunRequest`, `ResolvedRunInputs`, `RunExecutor`, `RunExecution`, `RunFailure`, and `RunArtifacts`.

**Implementation notes:**

- resolve and hash episode, deployment, scoring, executable, schema, protocol, and optional fault inputs before process start;
- start the external deployment through the isolation policy;
- mediate the bidirectional bounded JSONL session;
- dispatch every scored tool request through the managed-tool port;
- assign authoritative event order and preserve causal references;
- write partial artifacts for every terminal failure;
- evaluate only after replay and provenance validation succeed;
- retain the existing quick-start behavior as a thin compatibility wrapper until the generic CLI replaces it.

**Tests:** AWS/Azure/GCP execution, every reference topology, ground-truth denial, tool mediation, timeout, malformed messages, process crash, budget exhaustion, replay equivalence, deterministic hashes, partial artifact preservation.

```bash
cargo test -p hunteval-runner --test generic_run
cargo test -p hunteval-runner --test run_failures
cargo test -p hunteval-cli --test vertical_slice
```

### R2-04 — Benchmark journal and deterministic projection

**Objective:** persist benchmark progress so interruption is observable and resumable without losing attempt history.

**Primary files:** `hunteval-runner/src/benchmark/journal.rs`, `state.rs`, `projection.rs`, locking and atomic-write adapter tests.

**State rules:**

- only valid state transitions are appended;
- an attempt may enter one terminal state exactly once;
- projection order is by stable cell ID and attempt number;
- stale `running` attempts become explicitly `interrupted` events during resume;
- result artifact existence alone never marks a cell completed;
- completion requires artifact verification and matching cell identity;
- journal lines, snapshots, and error messages are bounded and untrusted on reload.

**Tests:** valid transition matrix, duplicate terminal event, truncated final line, tampered identity, stale lock, crash between journal sync and snapshot replace, replay determinism, symlink/path escape, interrupted attempt recovery.

```bash
cargo test -p hunteval-runner benchmark_journal
cargo test -p hunteval-runner --test benchmark_resume
```

### R2-05 — Matrix execution service

**Objective:** execute the exact benchmark matrix using the generic run engine and journal.

**Primary files:** `hunteval-runner/src/benchmark/service.rs`, scheduler integration, comparison eligibility reducer.

**Public application types:** `BenchmarkRunRequest`, `BenchmarkRunSummary`, `ResumePolicy`, and `BenchmarkProgress`.

**Implementation notes:**

- deterministic scheduling uses stable cell identity, not filesystem enumeration;
- concurrency is bounded by manifest and host policy;
- one cell failure does not stop unrelated cells unless fail-fast is explicitly requested;
- failed cells remain failed unless resume policy authorizes a new attempt;
- configuration drift creates a new identity rather than reusing an old result;
- comparison eligibility reports exact missing, failed, mismatched, or unverifiable pairs.

**Tests:** exact Cartesian product, bounded parallelism, all terminal cell outcomes, fail-fast behavior, resume policies, configuration drift, missing pair, fault-paired run, deterministic fake executor, real reference-deployment smoke matrix.

```bash
cargo test -p hunteval-runner --test benchmark_execution
cargo test -p hunteval-runner --test benchmark_resume
```

### R2-06 — Benchmark CLI

**Objective:** expose stable local commands without moving application logic into Clap handlers.

**Commands:**

```text
hunteval benchmark validate <manifest>
hunteval benchmark run <manifest> --output <directory> [--jobs N] [--fail-fast]
hunteval benchmark resume <benchmark-directory> [--retry failed|interrupted|none]
hunteval benchmark status <benchmark-directory> [--format text|json]
hunteval benchmark compare <benchmark-directory> --left <deployment> --right <deployment>
```

**Rules:** machine-readable output goes to stdout, diagnostics to stderr, non-success and non-comparable outcomes use documented exit codes, and CLI output never contains private paths or secrets.

**Tests:** argument conflicts, invalid paths, JSON output stability, exit codes, interrupted resume, non-comparable comparison, complete nine-episode smoke matrix.

```bash
cargo test -p hunteval-cli --test benchmark_commands
cargo run -p hunteval-cli -- benchmark validate examples/cloud-mvp-benchmark.yaml
```

**R2.1 exit gate:** one command executes at least two external reference deployments over all nine episodes and at least two paired seeds; a forced interruption resumes without overwriting attempts; a repeated identical benchmark yields equivalent normalized artifacts.

## 7. R2.2 — Complete evaluation contracts

### Normative metric decisions

R2 uses deterministic structured metrics. It does not introduce semantic model grading.

| Metric | Range and direction | Numerator / denominator | Required edge behavior |
|---|---|---|---|
| attack-path precision | `[0,1]`, higher | longest common subsequence length / submitted path length | empty submission with non-empty truth is `0`; both empty follows benign applicability |
| attack-path recall | `[0,1]`, higher | longest common subsequence length / expected path length | empty truth is not applicable unless benign semantics explicitly apply |
| timeline precision | `[0,1]`, higher | one-to-one submitted timeline entries matching event and versioned time tolerance / submitted entries | requires structured submitted timeline and private expected windows; duplicates cannot match twice |
| timeline recall | `[0,1]`, higher | one-to-one matched expected timeline entries / expected entries | zero expected entries is explicitly not applicable unless declared benign |
| conclusion correctness | `{0,1}`, higher | acceptable structured outcome match / one scored episode | never compares free-form summary wording |
| technique precision/recall | `[0,1]`, higher | exact versioned technique identifiers | empty-set behavior mirrors event metrics |
| evidence event coverage | `[0,1]`, higher | truth events cited by valid grounded evidence / truth events | submitted findings with no valid evidence produce `0` |
| evidence entity coverage | `[0,1]`, higher | truth entities cited by valid grounded evidence / truth entities | submitted findings with no valid evidence produce `0` |
| evidence sufficiency | `[0,1]`, higher | valid distinct evidence count capped at minimum required / minimum required | zero required evidence is not applicable |
| duplicate tool work | `[0,1]`, lower | repeated canonical tool fingerprints that add no new evidence / completed tool calls | zero calls is not applicable |
| useful communication | `[0,1]`, higher | operational messages with a causally linked target action or state transition / operational messages | zero messages is not applicable |
| measured duration utilization | `[0,1]`, lower | measured duration / configured duration cap, capped at `1` | a ratio at or above `1` carries a separate budget-exceeded status or constraint and is never reported as an ordinary success |
| verified cost utilization | `[0,1]`, lower | verified cost / configured cost cap, capped at `1` | unavailable or self-reported cost is not applicable and cannot satisfy a hard constraint; cap excess is a separate violation |
| submission stability | `[0,1]`, higher | mean paired similarity of structured submissions across listed seeds | fewer than two comparable runs is not applicable |
| metric stability | `[0,1]`, higher | one minus mean absolute paired difference for bounded metrics | missing or non-comparable cells remain explicit |

Exact names and serialization fields are frozen in R2-00. Any formula change after release requires a new metric contract version.

### R2-07 — Trusted evaluation view reducer

**Objective:** derive all evaluator inputs from verified stored artifacts while keeping ground truth in the trusted boundary.

**Primary files:** `hunteval-evaluation/src/input/*`, runner evaluation adapter, replay projection.

**Types:** `TrustedRunView`, `ObservedAction`, `ObservedEvidence`, `ObservedTask`, `ObservedMessage`, `ObservedFinding`, `SubmittedTimelineEntry`, and `EvaluationProvenance`.

**Tests:** forged/future/cross-run references, wrong agent ownership, incomplete runs, duplicate identifiers, missing artifacts, deterministic reduction, proof that the deployment-visible view cannot serialize ground truth.

```bash
cargo test -p hunteval-evaluation trusted_view
cargo test -p hunteval-runner --test evaluation_input
```

### R2-08 — Investigation and conclusion metrics

**Objective:** implement attack-path, timeline, structured conclusion, and technique contracts in small independent metric modules.

**Primary files:** `hunteval-evaluation/src/metrics/path.rs`, `timeline.rs`, `conclusion.rs`, `techniques.rs`, v0.4 episode and submission adapters.

**Tests:** exact, partial, reordered, duplicate, empty, benign, tolerance boundary, invalid time, unsupported technique version, and property tests for range and determinism.

```bash
cargo test -p hunteval-evaluation attack_path
cargo test -p hunteval-evaluation timeline
cargo test -p hunteval-evaluation conclusion
```

### R2-09 — Evidence and coordination metrics

**Objective:** add evidence completeness and deterministic observable coordination measures.

**Primary files:** `hunteval-evaluation/src/metrics/evidence.rs`, `coordination.rs`, canonical tool fingerprint helper.

**Rules:** canonical fingerprints use validated tool name plus canonical structured arguments; useful communication requires observable causality and cannot be inferred from message prose; raw duplicate counts remain diagnostic artifacts.

**Tests:** grounded and forged coverage, minimum evidence, equivalent JSON argument ordering, repeated call with new evidence, repeated call without evidence, causal and uncaused messages, reassignment, cancellation, zero denominators.

```bash
cargo test -p hunteval-evaluation evidence
cargo test -p hunteval-evaluation coordination
```

### R2-10 — Efficiency and cross-run stability

**Objective:** implement provenance-aware resource metrics and benchmark-level stability without fabricating values from a single run.

**Primary files:** evaluation resource metrics, `hunteval-statistics` stability module, runner benchmark aggregation.

**Rules:** runner-observed duration and counts are measured; tokens and monetary cost require a verified adapter for hard constraints; listed seeds define the stability sample; failures and missing cells are reported rather than imputed.

**Tests:** measured, verified, self-reported, unavailable, cap exceeded, zero cap, one sample, paired missing cells, deterministic seed order, identical and divergent submissions.

```bash
cargo test -p hunteval-evaluation efficiency
cargo test -p hunteval-statistics stability
cargo test -p hunteval-runner --test benchmark_metrics
```

### R2-11 — Scoring profile v0.4 and compatibility

**Objective:** allow profiles to select the completed metric set without embedding a universal score or unsafe missing-value behavior.

**Primary files:** scoring profile contracts, schema, examples, profile loader, constraint evaluation.

**Rules:** weights still sum to one; metric names and versions must exist; direction comes from the metric registry rather than authored input; constraints declare required resource provenance; v0.3 profiles remain readable through a compatibility adapter; profiles cannot treat missing resilience, reproducibility, or cost as success.

**Tests:** every missing-value policy, unknown metric/version, invalid weights, unverifiable cost constraint, disqualifying constraint, v0.3 compatibility, deterministic normalized JSON.

```bash
cargo test -p hunteval-evaluation scoring_profiles
cargo test -p hunteval-runner --test profile_compatibility
```

**R2.2 exit gate:** every roadmap metric has a normative contract and edge-case tests; evaluators reproduce identical vectors from the same verified artifacts; unsupported inputs remain explicitly not applicable; no metric reads files, starts processes, or accesses deployment adapters.

## 8. R2.3 — Comparative reporting

### R2-12 — Normalized benchmark result and claim graph

**Objective:** produce one validated reporting input from benchmark state, run results, statistics, constraints, and verified hashes.

**Primary files:** domain benchmark-result contracts, `hunteval-reporting/src/benchmark/*`, runner reporting adapter, v0.4 schemas.

**Types:** `BenchmarkResult`, `DeploymentSummary`, `MetricSummary`, `PairwiseComparison`, `RankingGroup`, `CellReference`, `ArtifactDigestReference`, `ReportClaim`, and `ClaimSource`.

**Rules:** rankings retain raw vectors; constraints order ranking before aggregates; inconclusive intervals are labeled; missing, failed, and non-comparable cells are counted; every claim has at least one valid source.

**Tests:** deterministic ordering, incomplete matrix, constraint-first ranking, missing pair, inconclusive comparison, invalid source, tampered digest, v0.4 schema round trip.

```bash
cargo test -p hunteval-reporting benchmark_result
cargo test -p hunteval-runner --test benchmark_reporting_input
```

### R2-13 — Deterministic JSON benchmark reports

**Objective:** make normalized JSON the complete machine-readable source of truth for comparisons.

**Primary files:** JSON renderer modules, normalized snapshots, report-generation application service.

**Content:** benchmark identity and hashes, cell inventory, deployment summaries, raw metrics, profile scores and omissions, constraints, resource provenance, statistical summaries, pairwise comparisons, ranking groups, claim sources, and known limitations.

**Tests:** byte determinism, incomplete/failed/non-comparable cells, untrusted strings, no private paths, no secrets, invalid numbers, exact source references.

```bash
cargo test -p hunteval-reporting benchmark_json
cargo run -p hunteval-cli -- report generate runs/benchmark-latest --format json
```

### R2-14 — Static HTML timeline and comparative views

**Objective:** render the normalized report as accessible, portable HTML without server or script execution.

**Primary files:** separate HTML page/component modules for overview, metrics, comparisons, timeline, coordination, attribution, artifacts, and limitations.

**Rules:** escape by default; use semantic HTML and inline static CSS only; display provenance labels; link run claims to trajectory sequences and benchmark claims to cells or metrics; avoid causal wording for observational attribution.

**Tests:** escaping in every component, no `<script>`, no event-handler attributes, safe relative links, accessibility landmarks, incomplete labels, attribution wording, deterministic snapshots.

```bash
cargo test -p hunteval-reporting benchmark_html
cargo test -p hunteval-reporting --test untrusted_rendering
cargo run -p hunteval-cli -- report generate runs/benchmark-latest --format html
```

### R2-15 — Report CLI and artifact verification

**Objective:** support run and benchmark report generation through one safe application boundary.

**Commands:**

```text
hunteval report generate <run-or-benchmark-directory> --format json|html
hunteval report verify <report-or-artifact-directory> --format text|json
```

**Rules:** input kind is detected from validated manifests, not directory names; reads are bounded; symlinks and traversal are rejected; outputs use atomic replacement; verification checks referenced artifacts and exact digests.

**Tests:** run and benchmark detection, malformed manifests, oversized input, symlink output attack, stale digest, partial report, JSON exit codes, no private path disclosure.

```bash
cargo test -p hunteval-cli --test report_commands
cargo test -p hunteval-runner --test report_verification
```

**R2.3 exit gate:** normalized JSON and static HTML represent complete, incomplete, and non-comparable benchmarks; every conclusion has a valid source; rendering is deterministic and safe for untrusted text; report verification detects any referenced artifact change.

## 9. R2.4 — GitHub delivery hardening

### R2-16 — Reproducible toolchain and canonical quality scripts

**Objective:** eliminate drift between developer and GitHub Actions commands.

**Primary files:** `rust-toolchain.toml`, `scripts/ci/quality.sh`, `scripts/ci/security.sh`, `scripts/ci/e2e.sh`, contributor documentation.

**Rules:** pin the supported Rust toolchain and components; keep cargo-deny version explicit; fail on missing required host capabilities rather than silently skipping security tests; avoid CI-only code paths; key caches by toolchain, target, features, and `Cargo.lock`.

**Tests:** execute each script locally from a clean checkout; verify shell syntax and executable bits; demonstrate that a deliberately failing fixture makes the corresponding script fail.

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/e2e.sh
```

### R2-17 — GitHub Actions hardening

**Objective:** make GitHub Actions the authoritative merge pipeline using the canonical repository-owned scripts.

**Stages:** policy, quality, test, security, end-to-end, documentation, package.

**Primary files:** `.github/workflows/ci.yml`, reusable workflow files if justified, CI operations documentation.

**Rules:** least-privilege tokens; no deployment secrets in ordinary jobs; interruptible branch pipelines; bounded job timeouts and artifact retention; safe DuckDB native cache; protected release jobs; merge requests require the canonical gates.

**Artifacts:** test logs, schema validation results, end-to-end normalized artifacts, cargo-deny result, documentation build, and verification summary. Private ground truth must never be uploaded separately or included in deployment-visible bundles.

**Acceptance:** branch and pull-request workflows pass; a seeded negative test proves gate failure propagation; every job invokes canonical scripts; cache restoration does not change results.

### R2-18 — GitHub governance and release dry run

**Objective:** document and verify the external GitHub settings that cannot be enforced only by committed files.

**Primary files:** `docs/GITHUB_OPERATIONS.md`, `CODEOWNERS`, release checklist, ownership and approval rules, security disclosure references.

**Checklist:** protected `main`, prohibited force push, required successful pipeline, minimum approvals, CODEOWNERS for schemas/security/fixtures, protected tags, release permissions, artifact retention, runner trust, dependency update policy, rollback.

**Acceptance:** an authorized maintainer records completion of the settings checklist; a release-candidate tag dry run produces checksums and verified artifacts without publishing a production release.

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/e2e.sh
git diff --check
```

**R2.4 exit gate:** the same commit and feature set receive equivalent outcomes locally and in GitHub Actions; all mandatory jobs are required for merge; CI artifacts are bounded, non-secret, and independently verifiable.

## 10. Dependency and delivery order

### Executable delivery waves from the current baseline

Milestone numbers describe contract ownership; implementation follows dependency order. Each milestone is one reviewable behavior change and one dedicated commit after its focused tests and the complete workspace gate pass.

#### Wave A — Close the operational MVP

1. **R2-05 matrix service**
   - define a small cell-executor port so scheduling and resume tests use a deterministic fake while production uses `RunExecutor`;
   - implement the production DuckDB managed-tool router and derive event provenance from validated worker output;
   - generate a normalized per-cell result containing matching benchmark cell and run identities before journal completion;
   - drive queue, attempt, completion, failure, interruption, and non-comparable events through `BenchmarkJournal`;
   - enforce deterministic cell order, bounded jobs, fail-fast semantics, and explicit retry policy without overwriting prior attempts;
   - reduce typed comparison eligibility from verified cells and reject configuration, pairing, fault, budget, protocol, schema, or digest drift;
   - finish with a fake-executor matrix suite and a real sandboxed reference-deployment smoke matrix.
2. **R2-06 benchmark CLI**
   - expose `run`, `resume`, `status`, and `compare` as thin handlers over the R2-05 service;
   - stabilize JSON output, diagnostics, documented exit codes, argument conflicts, and safe path handling;
   - execute the nine-episode, two-deployment, paired-seed benchmark from one command;
   - force an interruption, resume it, and prove equivalent eligible cells and normalized hashes.

**Wave A release checkpoint:** satisfy the R2.1 exit gate. At this point HuntEval may be described as an operational local MVP, but not as the complete auditable R2 release.

#### Wave B — Complete deterministic evaluation

3. **R2-07 trusted evaluation view:** replay and verify stored artifacts into typed observations; reject cross-run, future, forged, duplicate, or wrongly owned references before metric code runs.
4. **R2-08 investigation metrics:** implement path, timeline, structured conclusion, and technique modules independently, including benign and not-applicable behavior.
5. **R2-09 evidence and coordination metrics:** implement evidence coverage and sufficiency, canonical duplicate-work fingerprints, and causally useful communication without interpreting prose.
6. **R2-10 efficiency and stability:** use runner-measured duration and counts, verified-only cost, paired seeds, explicit missing cells, and deterministic cross-run aggregation.
7. **R2-11 scoring v0.4:** register the complete versioned metric set, preserve v0.3 reading, enforce explicit missing-value policies, and prevent unverified resource values from satisfying constraints.

**Wave B release checkpoint:** satisfy the R2.2 exit gate using only verified artifacts and deterministic structured calculations.

#### Wave C — Produce independently verifiable comparisons

8. **R2-12 benchmark result and claim graph:** normalize benchmark state, metrics, statistics, constraints, provenance, and hashes into one validated reporting input.
9. **R2-13 benchmark JSON:** make deterministic normalized JSON the complete machine-readable source of truth.
10. **R2-14 benchmark HTML:** render accessible static comparative views, timelines, attribution, limitations, and artifact provenance with no active content.
11. **R2-15 report CLI and verification:** detect validated input kinds, generate reports atomically, and verify every referenced artifact digest.

**Wave C release checkpoint:** satisfy the R2.3 exit gate for complete, incomplete, failed, and non-comparable benchmarks.

#### Wave D — Make delivery reproducible and governed

12. **R2-16 canonical toolchain and scripts:** pin the supported toolchain and make repository-owned quality, security, and end-to-end scripts authoritative. Missing Bubblewrap or another mandatory security capability must fail rather than skip.
13. **R2-17 GitHub Actions hardening:** invoke only canonical scripts, use least privilege, bound time and retention, preserve clean-cache parity, and publish only non-secret verification artifacts.
14. **R2-18 governance and release dry run:** add ownership and operations documentation, verify protected-branch and tag settings with an authorized maintainer, and produce a checksummed release candidate without publishing a production release.

**Wave D release checkpoint:** satisfy the R2.4 gate and then the complete R2 definition in section 12.

### Milestone handoff checklist

Before committing each remaining milestone:

1. focused positive, negative, malformed-input, and determinism tests pass;
2. every changed public contract has schema, compatibility, and canonical-example coverage;
3. security-sensitive behavior includes a fail-closed test that actually executes on the required host capability;
4. source files have been split for cohesion before 300 lines where practical;
5. README, contracts, ADRs, operator documentation, and this status table reflect the implemented behavior;
6. all mandatory workspace commands in section 3 pass;
7. `git diff --check` passes and the diff contains no unrelated or private files;
8. the milestone receives one descriptive commit; its status changes to `complete` only after recording the commit and gate evidence.

Pushes do not combine unfinished milestones. If a remote CI gate fails, the milestone returns to active status until the same revision passes locally and in GitHub Actions.

```text
R2-00 contracts and ADRs
  -> R2-01 benchmark identities
     -> R2-02 reference deployment
     -> R2-03 generic run engine
        -> R2-04 journal/projection
           -> R2-05 matrix service
              -> R2-06 benchmark CLI

R2-03 trusted run artifacts
  -> R2-07 evaluation view
     -> R2-08 investigation metrics
     -> R2-09 evidence/coordination metrics
     -> R2-10 efficiency/stability
        -> R2-11 scoring profiles

R2-06 benchmark outputs + R2-11 complete metrics
  -> R2-12 benchmark result/claims
     -> R2-13 JSON reporting
     -> R2-14 HTML reporting
        -> R2-15 report CLI/verification

R2-16 canonical CI scripts may begin after R2-00
R2-17 GitHub Actions workflow tracks the evolving scripts
R2-18 governance dry run begins only after R2-15 and R2-17
```

Only one behavior-changing pull request is active at a time. Documentation-only contract preparation may proceed separately, but contracts must merge before dependent implementation.

## 11. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| v0.4 breaks existing v0.3 artifacts | historical runs become unreadable | immutable v0.3 fixtures, compatibility reader, explicit schema tests; roll back new writer while retaining reader |
| generic execution accidentally exposes ground truth | benchmark invalidation and security breach | public package type at process boundary, sandbox negative tests, no private types in reference deployment |
| resume duplicates or overwrites results | biased statistics and lost provenance | append-only attempts, stable cell identity, atomic projection, transition property tests |
| concurrency makes artifacts nondeterministic | comparisons cannot reproduce | deterministic cell ordering and identity; authoritative per-run ordering; aggregate sorting independent of completion order |
| advanced metrics encode ambiguous semantics | misleading scores | deterministic structured inputs only, versioned formulas, explicit applicability, no semantic model grader |
| verified cost is unavailable | invalid cost ranking | provenance-aware null metric and non-comparable hard constraint; never trust unverified self-report |
| reporting creates injection or causal overclaim | unsafe or misleading output | typed claim sources, escaping, no scripts, controlled wording, untrusted rendering tests |
| CI cache changes behavior or leaks data | unreliable or unsafe builds | content-keyed caches, no private artifacts in cache, clean-cache parity job, bounded retention |
| DuckDB native build exceeds CI budget | slow or timed-out pipelines | safe cache, job separation, explicit timeout, measure before changing dependency/build strategy |
| PRs grow beyond human reviewability | maintainability regression | file-size gate, PR sequence above, contract-first changes, split modules before 300 lines |

## 12. R2 completion definition

R2 is complete only when all of the following are true:

1. Two or more independently executed reference deployments complete the nine-episode paired-seed benchmark through the public CLI.
2. Forced interruption and resume preserve attempt history and yield the same eligible comparison set as an uninterrupted execution.
3. Every R2.2 metric has a versioned contract, edge-case tests, and deterministic artifact-based evaluation.
4. Rankings preserve raw vectors, constraints, provenance, sample counts, uncertainty, and non-comparable cells.
5. JSON and static HTML reports cite every conclusion and verify every referenced digest.
6. Ground truth remains physically unavailable to deployment and reporting processes.
7. Local and GitHub Actions quality gates are equivalent and required.
8. All mandatory commands pass with no undocumented limitation or compatibility break.

Completion evidence must list exact commands, benchmark manifest hash, dataset hashes, deployment hashes, scoring-profile hash, runner and worker hashes, result digest, known limitations, and ADR status changes.
