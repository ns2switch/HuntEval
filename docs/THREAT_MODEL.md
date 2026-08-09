# Threat model

## 1. Assets

HuntEval must protect:

- hidden ground truth;
- benchmark integrity;
- telemetry fixtures;
- deployment credentials and API keys;
- host filesystem and network;
- scoring profiles and evaluator logic;
- prompt and configuration artifacts;
- run trajectories and reports;
- contributor-supplied datasets and knowledge documents.

## 2. Trust boundaries

```text
User / CI
   |
Trusted HuntEval runner
   |-- trusted evaluator and ground truth
   |-- constrained DuckDB worker
   |-- optional knowledge index
   `-- untrusted deployment process
          |-- untrusted agents
          `-- untrusted model output
```

The evaluated deployment, model output, retrieved documents, episode-authored knowledge, and generated SQL are untrusted.

## 3. Adversaries and failure sources

- a malicious evaluated deployment attempting to read ground truth;
- a buggy deployment emitting malformed messages or unbounded work;
- prompt injection embedded in author-provided knowledge;
- a compromised or noisy worker agent;
- a benchmark contributor supplying malicious files;
- accidental secret leakage in logs or artifacts;
- optimization logic overfitting or gaming known episodes.

## 4. Threats and mitigations

### 4.1 Ground-truth leakage

**Threat:** The deployment reads hidden event IDs, labels, or expected conclusions.

**Mitigations:**

- separate filesystem paths and mounts;
- sandboxed deployment process;
- no ground-truth fields in resolved public manifests;
- canary values and leakage detection;
- artifact and environment review;
- restricted process permissions and network access.

### 4.2 Agent impersonation

**Threat:** One process submits actions as another registered agent.

**Mitigations:**

- registered identities;
- process/session-bound credentials or channels;
- message correlation and ownership validation;
- rejection of unknown or inactive agent IDs.

### 4.3 Forged provenance

**Threat:** An agent claims evidence from a tool call that never occurred.

**Mitigations:**

- tool results are generated and signed or correlated by the runner;
- evidence may reference only existing action IDs and returned event IDs;
- replay validates every causal reference.

### 4.4 Hidden communication channels

**Threat:** Agents bypass observable coordination to exchange information or access external resources.

**Mitigations:**

- strict benchmark mode with mediated communication;
- network disabled by default;
- constrained IPC;
- process and filesystem isolation;
- benchmark report marks non-mediated deployments.

### 4.5 Message loops and denial of service

**Threat:** Agents generate unbounded tasks, messages, or retries.

**Mitigations:**

- message, task, and delegation-depth limits;
- timeouts and watchdogs;
- cancellation and terminal budget events;
- per-agent and deployment-level rate limits.

### 4.6 Dangerous SQL

**Threat:** Queries modify data, read files, install extensions, or exhaust resources.

**Mitigations:**

- AST parsing;
- read-only policy;
- table and function allowlists;
- isolated worker process;
- memory, CPU, time, and result limits;
- no extensions, external files, or network.

### 4.7 Prompt injection through optional RAG

**Threat:** A knowledge document instructs an agent to bypass policies or reveal secrets.

**Mitigations:**

- label documents as untrusted data;
- separate instructions from retrieved content;
- keep authorization and tool policies outside the corpus;
- sanitize metadata and paths;
- require document citations;
- restrict tools independently from prompts.

### 4.8 Shared-memory poisoning

**Threat:** A worker introduces false evidence or malicious instructions into shared state.

**Mitigations:**

- provenance and versioning;
- scoped memory;
- evidence validation;
- challenge and conflict-resolution events;
- ability to quarantine or reject agent output.

### 4.9 Collusion and benchmark gaming

**Threat:** A deployment memorizes public episodes or exploits evaluator behavior instead of hunting.

**Mitigations:**

- hidden tests;
- rotating episodes and seeds;
- withheld ground truth;
- benchmark constraints;
- leakage and anomaly detection;
- no evaluator feedback during scored runs.

### 4.10 Secret exfiltration

**Threat:** API keys or host secrets appear in messages or reports.

**Mitigations:**

- no real secrets in datasets;
- external secret management;
- environment allowlist;
- log redaction;
- network blocked by default;
- avoid storing provider request bodies unless required.

### 4.11 Unsafe prompt optimization

**Threat:** An optimizer weakens authorization or inserts benchmark-specific answers.

**Mitigations:**

- immutable prompt sections;
- human review;
- sandboxed A/B tests;
- hidden test separation;
- no ground-truth references;
- security-policy diff checks.

### 4.12 Malicious dataset files

**Threat:** Crafted Parquet, YAML, Markdown, or archive files exploit parsers or consume excessive resources.

**Mitigations:**

- schema validation;
- file-size and decompression limits;
- dependency updates;
- isolated parsing where practical;
- no execution of embedded content;
- trusted release signing for official benchmark packs.

### 4.13 Host process escape and descendant survival

**Threat:** An untrusted deployment or managed worker reads evaluator files, reaches the network, spawns surviving descendants, or exhausts host resources after its immediate process exits.

**Mitigations:**

- one fail-closed Bubblewrap adapter for scored deployments and managed DuckDB workers;
- executable probes for namespace, read-only mount, network-denial, process-tree, and resource-limit behavior;
- a new PID namespace, denied network namespace, read-only explicit mounts, isolated temporary storage, cleared environment, and termination tied to the namespace leader;
- explicit versioned limits for wall time, CPU time, address space, file size, descriptors, processes, and bounded output;
- complete-tree timeout and drop cleanup tests using descendant pipe holders;
- no fallback to unsandboxed scored execution when a required capability is unavailable.

### 4.14 Artifact tampering and diagnostic disclosure

**Threat:** A modified, partial, linked, or oversized run is treated as authentic, or hostile stderr and configuration values disclose secrets through diagnostics and CI artifacts.

**Mitigations:**

- bounded no-follow artifact reads and exact manifest digest verification;
- trajectory replay, submission equivalence, execution-policy validation, and normalized-result consistency checks;
- explicit incomplete and unsupported states without private re-evaluation claims;
- centralized bounded redaction before diagnostics are serialized;
- deterministic scanning of tracked public inputs, generated reports, CI evidence, and release-candidate contents;
- secret findings retain only a rule, safe relative location, and one-way fingerprint.

## 5. MVP security requirements

- No deployment network access by default.
- Hidden ground truth is not mounted into deployment processes.
- DuckDB executes in a separate constrained process.
- Hard limits exist for time, memory, messages, tasks, and queries.
- Artifacts include integrity hashes.
- Logs are redacted and do not contain secrets.
- Retrieved documents cannot modify tool or authorization policies.
- Protocol and provenance violations are represented in final results.

## 6. Security testing

The test suite must include:

- attempts to read hidden paths;
- SQL policy bypass cases;
- malformed and oversized JSONL messages;
- unknown-agent impersonation;
- forged action and evidence references;
- unbounded task and message loops;
- prompt-injection fixtures;
- archive and path traversal tests;
- cancellation and timeout behavior.
- descendant-process cleanup and operating-system limit enforcement;
- capability-probe failure and unsupported-host behavior;
- completed, partial, tampered, oversized, symlinked, and hard-linked run verification;
- redaction and secret-scanner non-disclosure tests;
- protocol property, retained corpus, bounded fuzz, and hostile live-process tests.
