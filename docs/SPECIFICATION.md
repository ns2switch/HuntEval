# HuntEval technical specification v0.3

## 1. Vision

HuntEval evaluates systems that use one or more agents to investigate threat-hunting scenarios. The evaluated unit is a **deployment**: the complete combination of agents, roles, models, prompts, tools, memory, coordination strategy, and runtime configuration.

The framework must answer:

1. Which deployment solves a benchmark most effectively?
2. With what precision, recall, evidence quality, cost, latency, and stability?
3. Which agents, tasks, messages, and tool actions contributed to success or failure?
4. How resilient is the deployment when agents or tools fail?
5. Which prompt changes are likely to improve behavior, and do controlled experiments confirm the improvement?

HuntEval is not tied to Self-RAG. Retrieval-augmented generation is an optional capability that deployments may use to consult hunt-author knowledge, and that HuntEval may later use to query generated reports.

## 2. Product goals

### 2.1 Primary goals

- Run cloud threat-hunting episodes in a deterministic local environment.
- Evaluate single-agent and multi-agent deployments through a neutral protocol.
- Compare deployments under equivalent budgets and datasets.
- Preserve complete observable provenance without collecting private reasoning.
- Attribute findings, errors, cost, and latency to agents and actions.
- Produce metric vectors and configurable aggregate rankings.
- Support repeated runs and statistically defensible comparisons.
- Diagnose recurring failure patterns.
- Generate evidence-backed prompt improvement hypotheses.

### 2.2 Secondary goals

- Support centralized, hierarchical, and decentralized coordination.
- Support homogeneous and heterogeneous model deployments.
- Provide a Python SDK after the core process protocol stabilizes.
- Permit optional local RAG over author-provided knowledge.
- Generate static HTML and normalized JSON reports.

### 2.3 Non-goals for the MVP

- Production incident response automation.
- Direct access to cloud accounts or production SIEMs.
- Autonomous modification of security policies.
- Private chain-of-thought inspection.
- A universal leaderboard score independent of benchmark context.
- Fully automated prompt search over hidden test episodes.

## 3. Target users

- Researchers evaluating multi-agent architectures.
- SOC and threat-hunting teams comparing assistant deployments.
- Developers building cloud-security agents.
- Model providers testing agentic investigation capabilities.
- Open-source contributors creating episodes, tools, and adapters.

## 4. Core concepts

### 4.1 Episode

A reproducible threat-hunting scenario containing:

- public objective and constraints;
- telemetry fixtures;
- exposed schemas and semantic views;
- optional author-provided knowledge;
- hidden ground truth;
- budgets and safety limits;
- optional fault-injection profile.

### 4.2 Deployment

The complete system under evaluation, including:

- one or more agents;
- agent identities and capabilities;
- prompt versions and hashes;
- model identifiers and generation parameters;
- memory and communication topology;
- coordination policy;
- optional retrieval components;
- runtime and tool-request adapter.

### 4.3 Run

One execution of a deployment against one episode with a fixed seed, budget, and configuration.

### 4.4 Benchmark

A versioned collection of episodes, repetitions, seeds, constraints, and a scoring profile.

### 4.5 Trajectory

An append-only sequence of observable events produced during a run. It includes coordination, task, message, hypothesis, tool, evidence, finding, failure, and submission events.

### 4.6 Evidence

A structured claim grounded in one or more HuntEval-issued tool results. Evidence is not a free-form opinion and must preserve provenance.

### 4.7 Finding

A threat-hunting conclusion that references evidence, affected entities, event identifiers, time ranges, confidence, and plausible benign alternatives.

## 5. High-level architecture

```text
HuntEval CLI
    |
    v
Benchmark Orchestrator
    |-- Episode Loader
    |-- Deployment Process Adapter
    |-- Budget and Policy Engine
    |-- Fault Injection Controller
    |-- Managed Tool Layer
    |     |-- DuckDB Worker
    |     |-- Knowledge Retrieval
    |     `-- Future Tool Plugins
    |-- Trajectory Recorder
    |-- Evaluation Engine
    |-- Statistical Comparison Engine
    |-- Failure Classification Engine
    |-- Prompt Improvement Analyzer
    `-- Report Generator
```

## 6. Repository layout

```text
hunteval/
├── crates/
│   ├── hunteval-domain
│   ├── hunteval-protocol
│   ├── hunteval-runner
│   ├── hunteval-duckdb
│   ├── hunteval-evaluation
│   ├── hunteval-statistics
│   ├── hunteval-resilience
│   ├── hunteval-knowledge
│   ├── hunteval-reporting
│   └── hunteval-cli
├── schemas/
├── sdk/python/
├── benchmarks/
├── datasets/
│   ├── aws/
│   ├── azure/
│   └── gcp/
├── deployments/
├── examples/
└── docs/
```

The MVP may initially combine small crates when this reduces unnecessary complexity, but dependency direction must preserve the domain boundary.

## 7. Initial cloud scope

The initial benchmark focuses on identity and control-plane activity because it supports comparable episodes across providers.

### 7.1 AWS

Initial telemetry:

- AWS CloudTrail management events;
- IAM inventory snapshots;
- optional GuardDuty-derived context.

Initial scenario families:

- compromised identity;
- suspicious role assumption;
- privilege escalation through policy changes;
- creation of access keys or persistence credentials;
- cross-account access.

### 7.2 Microsoft Azure

Initial telemetry:

- Microsoft Entra ID sign-in and audit logs;
- Azure Activity Logs;
- service principal and role inventory snapshots.

Initial scenario families:

- anomalous sign-in followed by administrative activity;
- privileged role assignment;
- service-principal credential creation;
- consent or application abuse;
- cross-tenant or subscription access.

### 7.3 Google Cloud

Initial telemetry:

- Google Cloud Audit Logs;
- IAM policy snapshots;
- service-account and project inventory.

Initial scenario families:

- service-account impersonation;
- IAM policy modification;
- service-account key creation;
- cross-project access;
- suspicious control-plane enumeration.

## 8. Data model

### 8.1 Raw and normalized layers

HuntEval must preserve provider-native fields while exposing optional normalized semantic views.

```text
Provider-native Parquet tables
          |
          v
Read-only DuckDB tables
          |
          +--> provider-specific queries
          `--> normalized semantic views
```

A universal schema must not remove provider-specific semantics.

### 8.2 Required event fields

Each scored event must have a stable event identifier. Provider-specific tables should expose or derive:

- `event_id`;
- event time;
- provider;
- tenant, account, subscription, or project;
- principal;
- action;
- resource;
- source address when available;
- raw or normalized attributes.

### 8.3 Hidden ground truth

Ground truth is stored outside the deployment-visible episode mount and may include:

- malicious event IDs;
- benign but suspicious event IDs;
- malicious entities;
- expected attack path;
- expected timeline boundaries;
- expected ATT&CK techniques;
- acceptable conclusion variants;
- minimum evidence requirements.

## 9. Deployment protocol

The MVP uses newline-delimited JSON over a child process's standard input and output.

### 9.1 Why process-based JSONL

- language-neutral;
- easy to test and replay;
- isolates crashes and malformed output;
- supports local Python and Rust deployments;
- avoids requiring an agent framework.

### 9.2 Registration

A deployment must register:

- deployment ID and architecture;
- agents and roles;
- capabilities;
- prompt versions and hashes;
- models and parameters;
- supported protocol version.

### 9.3 Managed actions

Agents request actions. HuntEval validates budgets and policies, executes the action, records provenance, and returns a structured observation.

Initial actions:

- execute read-only SQL;
- request a public schema description;
- retrieve optional author-provided knowledge;
- create or update a hypothesis;
- create, delegate, complete, fail, or reassign a task;
- share evidence;
- propose, challenge, accept, or reject a finding;
- submit the final hunt result.

## 10. Coordination model

HuntEval does not prescribe a topology. It supports:

- single generalist agent;
- supervisor-worker;
- supervisor with specialists;
- hierarchical teams;
- peer-to-peer agents;
- externally orchestrated deployments.

All coordination relevant to scoring must be observable through protocol events. Hidden internal communication is not scored and may be prohibited in strict benchmark mode.

### 10.1 Task lifecycle

```text
created -> delegated -> started -> completed
                      |          -> failed -> reassigned
                      `-> cancelled
```

### 10.2 Message lifecycle

Messages include source, target, task, concise purpose, and references to evidence or actions. Free-form messages are allowed, but structured metadata is mandatory for attribution.

### 10.3 Concurrency

The benchmark defines maximum registered agents, active agents, parallel tool calls, and total outstanding tasks. HuntEval records active time, idle time, queue time, and synchronization delay where measurable.

## 11. Tool execution

### 11.1 DuckDB worker

DuckDB runs in a separate constrained process. The SQL policy must enforce:

- read-only statements;
- an allowlist of exposed schemas and tables;
- query timeout;
- memory limit;
- row and result-size limit;
- no extension installation;
- no filesystem access outside mounted data;
- no network access;
- AST-based validation rather than string matching alone.

### 11.2 Optional knowledge retrieval

The episode author may provide a corpus containing:

- threat-intelligence reports;
- prior incident reports;
- cloud architecture descriptions;
- known service accounts and administrative ranges;
- hunting instructions;
- internal detection notes.

Retrieval is optional and not the primary benchmark target. HuntEval records queries, returned document IDs, citations, latency, and cost. Documents are treated as untrusted data and must not override tool or safety policies.

### 11.3 Future plugins

Future tools may include entity graphs, timeline builders, Sigma translators, or provider-specific query adapters. The plugin API must preserve managed execution and provenance.

## 12. Budgets

Each episode or benchmark may limit:

- wall-clock duration;
- deployment steps;
- tool calls;
- SQL queries;
- retrieved documents;
- inter-agent messages;
- generated tokens;
- monetary cost;
- registered and concurrently active agents;
- delegation depth.

Budget exhaustion produces a structured terminal event and does not silently discard the run.

## 13. Run artifacts

A run directory contains at minimum:

```text
run.json
resolved-config.json
trajectory.jsonl
submission.json
result.json
metrics.json
artifacts/
logs/
```

Artifacts must include hashes for:

- public episode manifest;
- hidden ground truth;
- telemetry files;
- deployment configuration;
- prompts;
- scoring profile;
- HuntEval binary or build identifier.

## 14. Evaluation dimensions

HuntEval preserves a metric vector containing:

1. investigation quality;
2. evidence quality;
3. coordination quality;
4. resilience;
5. efficiency;
6. reproducibility.

An optional aggregate score is calculated only through a named, versioned scoring profile.

## 15. Prompt diagnosis and improvement

Prompt improvement is a future diagnostic workflow:

```text
observable traces
-> failure classification
-> agent attribution
-> prompt inspection
-> improvement hypothesis
-> candidate patch
-> controlled A/B test
-> validation on unseen episodes
```

Recommendations must not be presented as proven improvements until validated.

## 16. CLI requirements

Initial command surface:

```bash
hunteval episode validate <path>
hunteval deployment validate <path>
hunteval run --episode <id-or-path> --deployment <path>
hunteval benchmark run <path>
hunteval compare <run-set-or-deployments...>
hunteval trajectory inspect <run-id>
hunteval coordination analyze <run-id>
hunteval report generate <run-id> --format json|html
```

Future commands:

```bash
hunteval resilience run ...
hunteval prompt compare ...
hunteval diagnose ...
hunteval knowledge index ...
hunteval ask ...
```

## 17. Reproducibility requirements

- Every stochastic run records a seed.
- Every comparison uses equivalent budgets unless explicitly marked otherwise.
- Deployment prompts and model settings are immutable within a run.
- Benchmark manifests reference exact episode and scoring-profile versions.
- Repeated executions produce aggregate statistics rather than relying on one run.
- Hidden test episodes must remain unavailable to prompt optimization.

## 18. MVP baseline deployments

The repository should include three minimal reference deployments:

### A. Single generalist

One agent performs planning, querying, analysis, and final synthesis.

### B. Supervisor and investigator

A supervisor creates and delegates tasks to one investigator, then produces the final submission.

### C. Supervisor and specialists

A supervisor coordinates a query specialist, an identity specialist, and an evidence critic.

These are reference baselines, not preferred architectures.

## 19. MVP benchmark

Nine synthetic episodes:

- three AWS episodes;
- three Azure episodes;
- three Google Cloud episodes.

Each provider should include identity compromise, privilege escalation, and persistence or credential creation. Fixtures must contain realistic benign background activity and stable ground truth.

## 20. Acceptance criteria for the first usable release

- One command executes a full local benchmark offline.
- At least three deployment topologies use the same protocol.
- Ground truth is inaccessible to deployment processes.
- DuckDB queries are read-only and constrained.
- Every final finding can be traced to tool results and agents.
- Metric vectors and aggregate profile scores are produced.
- Repeated runs produce confidence intervals and stability measures.
- A failed agent can be detected and represented in the result.
- Documentation, CLI output, prompts, schemas, examples, and code comments are in English.
