# Implementation plan

This document records the original MVP milestones. PR-00 through PR-15 implemented the executable MVP sequence. New work is prioritized in `ROADMAP.md` and must receive its own pull-request-level implementation plan before code changes begin. Release-specific preparation is maintained in `R2_IMPLEMENTATION_PLAN.md` through `R6_IMPLEMENTATION_PLAN.md`; completed-release evidence remains in the corresponding completion records.

## Milestone 0 — Repository bootstrap

Deliverables:

- Cargo workspace;
- CI workflow;
- Apache-2.0 license;
- contribution and security policy placeholders;
- basic identifier, timestamp, version, and error types;
- versioned schema directory;
- CLI skeleton.

Acceptance criteria:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Milestone 1 — End-to-end vertical slice

```text
synthetic Parquet fixture
-> episode validation
-> two-agent deployment registration
-> task creation and delegation
-> managed read-only DuckDB query
-> tool result
-> evidence with provenance
-> final finding and submission
-> hidden-ground-truth evaluation
-> trajectory.jsonl and result.json
```

Acceptance criteria:

- one offline integration test executes the complete flow;
- the deployment cannot read ground truth;
- invalid evidence provenance is rejected;
- the result contains event and entity precision/recall;
- replay reconstructs the run state.

## Milestone 2 — Multi-agent protocol

Deliverables:

- process handshake;
- protocol-version negotiation;
- deployment, agent, and capability registration;
- task lifecycle;
- operational messages;
- hypotheses, evidence, and findings;
- timeouts, cancellation, and malformed-line handling;
- protocol conformance tests.

Acceptance criteria:

- at least two reference deployments use the same protocol;
- unknown and duplicate IDs produce typed errors;
- task-state transitions are validated;
- protocol fuzz or property tests cover malformed messages.

## Milestone 3 — Managed DuckDB execution

Deliverables:

- separate worker process;
- Parquet table registration;
- normalized semantic views;
- SQL AST validation;
- read-only and allowlist policy;
- time, memory, row, and output limits;
- structured result and error contracts.

Acceptance criteria:

- mutation, file access, extension loading, and multi-statement bypasses are blocked;
- worker timeout and crash do not crash the runner;
- valid parameterized SELECT queries execute deterministically.

## Milestone 4 — Evaluation engine

Deliverables:

- event and entity precision/recall;
- attack-path and timeline evaluation;
- conclusion correctness;
- evidence grounding and completeness;
- basic coordination metrics;
- resource usage metrics;
- metric-vector output.

Acceptance criteria:

- every metric has unit tests and documented edge cases;
- evaluators operate from stored trajectory and submission artifacts;
- deterministic replay produces identical results.

## Milestone 5 — Benchmark runner and ranking

Deliverables:

- benchmark manifests;
- multiple episodes, deployments, repetitions, and seeds;
- scoring-profile loader;
- hard constraints;
- aggregate statistics;
- pairwise comparisons;
- CLI comparison output.

Acceptance criteria:

- a local command executes all matrix cells;
- interrupted runs can be resumed or reported cleanly;
- confidence intervals and stability metrics are generated;
- rankings preserve the underlying metric vectors.

## Milestone 6 — Resilience and fault injection

Deliverables:

- agent timeout;
- malformed agent response;
- worker failure;
- unavailable agent;
- task reassignment;
- noisy-agent fixture;
- paired baseline/fault comparison.

Acceptance criteria:

- fault events are deterministic for a seed;
- recovery and degradation metrics are produced;
- the runner remains stable when deployment processes fail.

## Milestone 7 — Cloud benchmark MVP

Create nine deterministic synthetic episodes:

- AWS: compromised identity, privilege escalation, persistence credential;
- Azure: anomalous sign-in, privileged role assignment, service-principal credential;
- Google Cloud: service-account impersonation, IAM policy change, service-account key creation.

Each episode must include:

- realistic benign background activity;
- stable event IDs;
- public schema documentation;
- hidden ground truth;
- an expected attack path;
- at least one plausible benign alternative;
- optional author-provided knowledge where useful.

Acceptance criteria:

- all episodes pass schema and integrity validation;
- reference SQL can recover the ground truth;
- no ground-truth labels appear in public files;
- fixtures are reproducible from documented generators.

## Milestone 8 — Reporting

Deliverables:

- normalized JSON report;
- static HTML report;
- run timeline;
- coordination graph or table;
- agent attribution;
- metric and ranking summaries;
- artifact links and hashes.

Acceptance criteria:

- HTML is generated without a server;
- every displayed claim links to a metric or trajectory event;
- reports clearly label incomplete and statistically inconclusive results.

## Milestone 9 — Optional knowledge retrieval

Deliverables:

- author-provided document manifest;
- local indexing adapter;
- retrieval tool contract;
- citations and provenance;
- prompt-injection test fixtures;
- retrieval usage metrics.

Acceptance criteria:

- the benchmark runs without retrieval;
- retrieval cannot access hidden ground truth;
- documents cannot change tool policies;
- citations identify exact document artifacts.

## Milestone 10 — Failure diagnosis

Deliverables:

- versioned failure taxonomy;
- deterministic failure classifiers;
- agent and prompt-version attribution;
- diagnostic report;
- confidence and evidence requirements.

Acceptance criteria:

- classifications reference affected runs and events;
- unsupported diagnoses are omitted rather than guessed;
- the same artifact set produces deterministic rule-based diagnoses.

## Milestone 11 — Prompt A/B experiments

Deliverables:

- prompt artifact registry;
- baseline/candidate experiment manifests;
- train, validation, and hidden-test partitions;
- candidate constraints;
- regression and trade-off report.

Acceptance criteria:

- only declared prompt artifacts differ between paired configurations;
- hidden tests are unavailable during candidate selection;
- security-policy sections are immutable;
- adoption recommendations include uncertainty and constraints.

## Milestone 12 — Assisted prompt recommendations

Deliverables:

- failure-to-prompt mapping rules;
- structured recommendation contract;
- candidate diff generation;
- human review workflow;
- validation trigger.

Acceptance criteria:

- every recommendation cites observable evidence;
- recommendations are labeled unvalidated until A/B tests pass;
- forbidden sections cannot be modified;
- benchmark-specific answer leakage is rejected.

## Cross-cutting requirements

Every milestone must:

1. update relevant documentation;
2. add positive and negative tests;
3. preserve protocol and schema versions;
4. avoid undocumented architectural changes;
5. report commands executed and limitations;
6. maintain English-only project artifacts.
