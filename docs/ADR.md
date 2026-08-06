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
