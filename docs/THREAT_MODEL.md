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
- verified analytical corpora, indexes, queries, and citations;
- third-party extension manifests, executables, and capability policies;
- Python SDK packages and cross-language fixtures.

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
Third-party adapters, analytical source text, query input, Python-authored artifacts, and package contents are also untrusted. Evaluator analytical corpora are a separate trusted-use scope and are never deployment-visible.

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

### 4.15 Topology and dataset-science metadata abuse

**Threat:** A contributor uses topology labels, classification metadata, review files, or generated documentation to disclose answers, inject active content, forge approval, or support an undeclared causal comparison.

**Mitigations:**

- schema 0.6 uses bounded enums, identifiers, relationship counts, reason codes, and deny-unknown-field parsing;
- public episode classification contains no benign or malicious answer label and is loaded separately from evaluator-only truth;
- approvals bind exact public-package, private-truth, reference-query, and review-policy hashes; missing, rejected, malformed, or stale records never approve an episode;
- contributor scaffolding refuses overwrite and unsafe targets, while validation is read-only and rejects symlinks and oversized files;
- reviewer inventories contain hashes and visibility labels rather than private file contents;
- topology registration must match authored agent identities, roles, and architecture;
- controlled topology analysis is unavailable until exact artifact hashes and every declared variable match;
- observational metrics cannot produce role-contribution or topology-resilience claims;
- static topology HTML escapes untrusted text and always labels controlled results experimental and topology-dependent.

### 4.16 Diagnostic overclaim and attribution disclosure

**Threat:** Untrusted agent text, incomplete traces, or mismatched artifacts are used to infer private reasoning, expose evaluator-only data, fabricate attribution, silently impute missing observations, or present correlation as causal contribution.

**Mitigations:**

- schema 0.7 permits only bounded typed references to exact observable artifacts and rejects unknown fields and variants;
- every attribution must resolve against digest-verified artifacts with matching ownership, run scope, event order, and referential integrity before serialization;
- public diagnosis cannot contain ground-truth identifiers, private paths or hashes, hidden-test material, secrets, or private chain of thought;
- taxonomy data is non-executable, while classifier behavior remains compiled and reviewable;
- unsupported classifications and metrics remain unavailable instead of being guessed, assigned low confidence, imputed, or converted to zero;
- recurrence is observational and cannot produce a causal contribution claim;
- agent or role contribution requires an eligible R4 controlled topology experiment and remains experimental and topology-dependent;
- R5 hypotheses remain unvalidated and cannot be represented as approved changes without a future R6 approval artifact;
- normalized diagnostic JSON is authoritative and static HTML must escape untrusted text and prohibit active content.

### 4.17 Controlled-improvement policy bypass and hidden-test oracle

**Threat:** A candidate reclassifies or removes safety controls, changes an undeclared variable, embeds benchmark answers, extracts hidden-test feedback through repeated selection attempts, retains stale validation after mutation, forges human approval, or causes HuntEval to modify an active deployment.

**Mitigations:**

- schema 0.8 registers exact bounded regular bytes and requires explicit structure before a candidate can be compared;
- the improvement policy owns a fixed immutable inventory covering authorization, tool access, filesystem, network, data handling, ground-truth isolation, benchmark constraints, output integrity, and security controls;
- typed diff operations can target only mutable classes, and any second or undeclared experimental variable makes equivalence ineligible;
- answer-leakage checks return bounded safe status without matched private values, and hidden-test membership and feedback remain unavailable during generation and selection;
- the candidate is frozen before a sealed final assessment, which cannot feed another candidate in the same lineage;
- baseline, candidate, policies, experiment, schemas, and relevant binaries are content-addressed, so any changed byte invalidates prior validation;
- lifecycle events are append-only and cannot emit `validated`, `approved`, or `adopted` without the exact preceding decision artifacts;
- human approval requires explicit confirmation over exact hashes, while adoption records only a separately confirmed external action;
- HuntEval exposes no write port that modifies a registered baseline or active deployment as a side effect of recommendation, validation, review, or adoption recording;
- normalized reports and bundles reject private paths, hidden feedback, secrets, active content, and uncited stage claims.

### 4.12 Analytical-corpus scope confusion

**Threat:** Evaluator history, diagnostic evidence, or hidden benchmark context becomes available to an evaluated deployment through local search.

**Mitigations:**

- one mandatory corpus scope with no mixed inventories;
- evaluator analytics and deployment-visible knowledge use separate composition paths;
- deployment-visible corpora accept authored public documents only;
- exact source hashes and successful public verification are required before indexing;
- query authorization binds the exact corpus scope and index digest;
- cross-scope requests fail without revealing rejected matches.

### 4.13 Malicious or over-privileged extensions

**Threat:** A third-party adapter requests undeclared access, bypasses managed tools, changes executable bytes, or escapes resource boundaries.

**Mitigations:**

- exact executable, manifest, capability-policy, and resolution hashes;
- out-of-process execution through the existing sandbox and supervisor;
- deny-by-default capability intersection with no manifest self-authorization;
- scored network access remains denied;
- scored tools remain runner-mediated;
- executable and policy preflight covers digest drift and excess capability; supervised deployment and managed-tool conformance cover bounded protocol flows, malformed output, timeout, crash, transcript identity, and process cleanup;
- Python SDK helpers contain no runner, evaluator, provider, or direct-tool authority.

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
- immutable-section removal, rename, reclassification, and encoded answer-leakage tests;
- hidden-test membership, feedback, and repeated-oracle request rejection;
- undeclared-variable, stale-candidate, forged-validation, forged-approval, and unconfirmed-adoption tests;
- deterministic recommendation lifecycle replay and changed-candidate invalidation.
- evaluator/deployment corpus separation, digest drift, dangling citation, and cross-scope query rejection;
- extension capability escalation, executable mismatch, malformed manifest, process failure, and direct-tool bypass;
- Python/Rust malformed-fixture agreement, path confinement, package-content, and protocol-state tests.
