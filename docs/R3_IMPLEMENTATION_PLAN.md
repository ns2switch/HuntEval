# R3 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R3.1 through R3.3 into a reviewable pull-request sequence. R3 hardens the already completed local benchmark loop against untrusted deployment implementations; it does not redesign benchmark evaluation, reporting, scoring, or topology semantics.

R3 covers:

- a fail-closed Linux sandbox capability contract and operating-system resource enforcement;
- complete process-tree supervision for deployments and managed workers;
- adversarial, property, fuzz, compatibility, and third-party conformance testing for the JSONL protocol;
- standalone verification of completed and partial run artifacts;
- centralized bounded redaction and deterministic secret scanning.

R2 remains complete with its recorded R2.4 external-enforcement caveat. Existing R2 contracts, completion commits, benchmark evidence, report semantics, metric vectors, constraint-first ranking, and GitHub Actions evidence are not reopened. R4.4 topology benchmarking and R6.4 prompt improvement analysis remain future roadmap work and receive no implementation through this plan.

### Delivery status

Status values are evidence-based. `implemented` means the behavior and focused local tests exist but the release evidence is not yet complete. `complete` requires a dedicated completion commit, focused acceptance tests, all canonical repository gates, documentation evidence, and passing GitHub Actions on that revision. `planned` makes no implementation claim.

| Milestone | Status | Evidence or next dependency |
|---|---|---|
| R3-00 | implemented | schema 0.5 contracts and accepted ADR-047 through ADR-052 |
| R3-01 | implemented | shared bounded redactor and non-disclosure tests |
| R3-02 | implemented | executable capability probes and `system check` |
| R3-03 | implemented | validated content-addressed execution policy and run provenance |
| R3-04 | implemented | infrastructure-only shared sandbox adapter |
| R3-05 | implemented | supervised PID namespace, resource launcher, and process-tree tests |
| R3-06 | implemented | production protocol transport migrated with typed failure mapping |
| R3-07 | implemented | DuckDB worker migrated with public-file-only mounts and isolation tests |
| R3-08 | implemented | deterministic protocol property suites |
| R3-09 | implemented | isolated fuzz package, public corpus, pinned bounded CI smoke configuration |
| R3-10 | implemented | hostile live-process cases and partial-run verification |
| R3-11 | implemented | content-addressed protocol compatibility inventory |
| R3-12 | implemented | sandboxed public deployment conformance service and CLI |
| R3-13 | implemented | bounded standalone public run-verification service |
| R3-14 | implemented | compatible `run verify` CLI and end-to-end integration |
| R3-15 | implemented | safe secret scanner and Security, Adversarial, End-to-end, and Package integration |

The release name R3/v0.3 is independent from persisted schema and process-protocol version numbers. Existing schema versions 0.3 and 0.4 and protocol version 0.3 remain immutable. New R3 artifacts use an additive schema version selected in R3-00; protocol version 0.3 remains the supported wire contract unless an independently reviewed compatibility defect requires an additive protocol revision.

## 2. Baseline audit

The repository already provides strong R3 foundations:

1. `LinuxSandbox` and the production protocol transport use Bubblewrap with new user, PID, filesystem, and network namespaces, a read-only public episode mount, an isolated temporary directory, and no inherited environment.
2. The canonical Security and End-to-end jobs install Bubblewrap on a pinned Ubuntu 22.04 runner and fail when the required executable or isolation test is unavailable.
3. Deployment stdout, stderr, JSONL lines, run duration, tool responses, stored trajectory, submission, and reporting inputs already have byte or time bounds.
4. The JSONL decoder rejects non-UTF-8, oversized, unterminated, multi-line, and malformed values; the protocol session validates identities, ownership, phase, budgets, and provenance.
5. Trajectory replay validates contiguous sequence numbers, exact predecessor hashes, causal protocol state, terminal completion, and the complete trajectory digest.
6. Stored evaluation re-verifies trajectory and submission bytes before building a non-serializable trusted evaluation view.
7. The DuckDB worker is a separate short-lived process with a deny-by-default SQL policy, engine-level external-access denial, memory and row bounds, a timeout, and typed crash/protocol failures.
8. Report verification validates normalized reports and their declared artifact digests without reading evaluator-only ground truth.

The audit also identifies the exact R3 gaps:

1. Sandbox construction is duplicated between `process.rs` and `run/transport.rs`; there is no single capability or execution-policy contract.
2. A file-existence check for `/usr/bin/bwrap` does not prove namespaces, mounts, process-tree termination, or resource controls work on the host.
3. Timeout handling kills the immediate child, but no test proves that grandchildren, worker descendants, or pipe-holder processes terminate.
4. Deployment execution does not enforce memory, CPU time, output-file size, open-file, or process-count limits at the operating-system boundary.
5. The DuckDB worker is constrained by its protocol and DuckDB settings but does not yet share the deployment sandbox boundary or a verified public-root mount.
6. Current protocol tests are example-based; no property suite, fuzz package, retained regression corpus, or bounded adversarial CI entrypoint exists.
7. Only protocol version 0.3 has canonical fixtures, but the supported-version inventory and compatibility result are not machine-readable.
8. There is no public third-party deployment conformance command.
9. There is no standalone `run verify` command. Existing stored-evaluation and report verification cover subsets of a complete run and expose different result shapes.
10. Redaction is exact-value replacement in one process adapter. Production protocol stderr, configuration diagnostics, and report metadata do not share a single bounded redaction policy.
11. No deterministic secret scan covers tracked fixtures, snapshots, CI logs, generated reports, and release-candidate inputs.

These gaps define the delivery order below. R3 must consolidate existing behavior instead of creating a parallel runner path.

## 3. Mandatory delivery rules

Every R3 pull request must:

- preserve the domain crate's independence from process, filesystem, Bubblewrap, DuckDB, CLI, and fuzzing implementations;
- keep evaluator-only ground truth outside deployment and worker mounts, environment, diagnostics, verification output, and conformance fixtures;
- execute scored tools only through HuntEval and keep network access denied for R3 scored execution;
- use typed stable failure codes and never include raw untrusted input, environment values, secrets, or private paths in public errors;
- enforce all reads, writes, collections, processes, protocol frames, diagnostics, scans, and verification work with explicit bounds;
- terminate the complete process tree on timeout, protocol failure, output overflow, controller cancellation, or dropped supervision handle;
- use stable Rust and no first-party `unsafe`; any low-level dependency requires supply-chain review and a documented safe API boundary;
- preserve protocol 0.3 and schema 0.3/0.4 compatibility through immutable fixtures and explicit adapters;
- retain exact artifact hashes and add the sandbox launcher, execution policy, and verification-relevant binaries to resolved run identity;
- keep hand-written production Rust files below 500 lines and split cohesive modules before 300 lines where practical;
- add positive, negative, malformed-input, deterministic/replay, and resource-exhaustion tests for each changed boundary;
- update contracts, schemas, threat model, ADRs, CLI documentation, and status evidence in the same pull request as behavior;
- keep all repository artifacts in English.

The canonical completion gates remain:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/e2e.sh
git diff --check
```

R3 adds one bounded adversarial entrypoint, introduced by R3-09:

```bash
./scripts/ci/r3-adversarial.sh
```

No milestone advances while an earlier dependency has a failing gate.

## 4. Architecture decisions to close in R3-00

R3-00 is documentation and contract work only. It adds proposed ADR-047 through ADR-052 without rewriting accepted ADR-001 through ADR-046.

### ADR-047 — Define the supported Linux sandbox capability contract

- R3 scored execution supports one named Linux Bubblewrap backend and fails closed on unsupported platforms.
- Capability detection executes safe probes for required namespaces, read-only binds, network denial, process-tree termination, and the selected resource-limit mechanism; executable presence alone is insufficient.
- Machine-readable capability output contains backend/version and supported capability names, not host paths, environment values, kernel command lines, or unrestricted diagnostics.
- R3 implements only `network: denied`; future exceptions require a new versioned policy and separate threat-model review.

### ADR-048 — Use one sandbox adapter and a supervised process tree

- A new infrastructure-only `hunteval-sandbox` crate owns Linux process construction, the small resource-limit launcher, bounded pipes, lifecycle supervision, and capability probing.
- `hunteval-runner` and `hunteval-duckdb` consume this adapter; `hunteval-domain` and `hunteval-protocol` do not depend on it.
- The launcher sets resource limits through an audited safe dependency API and then starts the requested executable. No first-party unsafe hook is permitted.
- Bubblewrap's PID namespace and the supervisor lifecycle must prove that stopping the namespace leader removes every descendant.

### ADR-049 — Make operating-system execution policy explicit and hashed

- An additive schema 0.5 `execution-policy` artifact declares backend, denied network policy, wall time, CPU time, address-space or memory cap, output-file size, open files, process count, and stdout/stderr bounds.
- New hardened benchmark manifests reference exact policy bytes. The resolved benchmark definition and cell identity include the policy and sandbox-launcher digests.
- Schema 0.3 and 0.4 inputs remain readable. Scored execution of a legacy manifest requires an explicitly selected, versioned compatibility policy whose digest is recorded; there is no implicit default or silent downgrade.
- Unsupported or unenforceable limits fail before the deployment receives episode data.

### ADR-050 — Separate deterministic protocol regression from exploratory fuzzing

- Property tests run deterministic seeded cases in ordinary CI.
- Retained minimal regression inputs run as bounded corpus tests in ordinary CI.
- `cargo-fuzz` targets live outside the workspace, use a pinned tool version, and run bounded smoke iterations in the adversarial job; longer campaigns are manual or scheduled.
- Every discovered defect first receives a minimized non-secret regression input and stable expected reason code before its fix is considered complete.

### ADR-051 — Define standalone run-verification levels and reason codes

- `run verify` verifies public run integrity without requiring private ground truth or an original model provider.
- It validates safe paths, manifest/version/run identity, exact declared digests, required artifact inventory, trajectory chain and terminal state, submission equivalence, normalized result consistency, and schema compatibility.
- It does not claim to re-evaluate private metrics. Any evaluator-only consistency check remains a separate trusted mode and is not implied by a public verification result.
- Completed, partial, malformed, unsupported, and tampered runs produce a machine-readable schema 0.5 result with stable path-free reason codes.

### ADR-052 — Centralize bounded redaction and secret scanning

- A small infrastructure leaf module or crate owns deterministic redaction and high-signal secret detection for process diagnostics, configuration values, report metadata, fixtures, snapshots, CI artifacts, and release inputs.
- Structured values are redacted before serialization where possible. Text scanning is bounded, streaming, and reports only rule IDs, safe relative artifact labels, locations, and one-way fingerprints.
- Matches never include the candidate secret in stdout, stderr, JSON, panic text, or snapshots.
- Secret scanning is a release gate, not a substitute for external secret management or repository-host scanning.

## 5. Planned architecture and contracts

```text
hunteval-domain          infrastructure-independent contracts and identifiers
hunteval-protocol        pure framing, state machine, replay, compatibility model
hunteval-sandbox         Linux capability probe, launcher, resource limits, process tree
hunteval-duckdb          SQL/worker protocol using hunteval-sandbox
hunteval-runner          run use cases, sandbox policy selection, artifact verification
hunteval-reporting       normalized rendering using already-redacted DTOs
hunteval-cli             system check, deployment conformance, and run verify adapters
```

Allowed new dependency direction:

```text
hunteval-sandbox  -> no HuntEval crate
hunteval-duckdb   -> hunteval-sandbox
hunteval-runner   -> hunteval-sandbox + existing ports
hunteval-cli      -> hunteval-runner
```

Schema 0.5 is additive and is expected to contain:

- `execution-policy.schema.json`;
- `sandbox-capability-report.schema.json`;
- `protocol-conformance-result.schema.json`;
- `run-verification-result.schema.json`;
- `secret-scan-result.schema.json`;
- shared stable reason-code and bounded diagnostic primitives.

The exact fields, bounds, compatibility table, and canonical examples are frozen in R3-00 before Rust types are added. Existing schemas are never edited to simulate compatibility.

### Stable CLI additions

```text
hunteval system check --format text|json
hunteval deployment conformance <deployment> --format text|json
hunteval run verify <run-directory> --format text|json
```

The current `hunteval run --episode ... --deployment ...` syntax remains accepted. New commands return zero only when all required checks pass; unsupported, incomplete, malformed, or tampered inputs return nonzero with bounded diagnostics. JSON output is deterministic and path-safe.

## 6. R3.1 — Isolation backends and resource enforcement

### R3-01 — Central bounded redaction primitive

1. **Objective:** establish the safe diagnostic boundary needed before process adapters are consolidated.
2. **Files and crates:** a small infrastructure redaction module or leaf crate selected by ADR-052; runner process errors; reporting metadata adapters; focused tests.
3. **Contracts:** `RedactionPolicy`, `Redactor`, `RedactedText`, maximum input/output sizes, stable truncation marker, and non-secret match metadata. Public errors retain typed codes and never echo source values.
4. **Tests:** exact and overlapping values, empty-value rejection, UTF-8 and lossy byte input, truncation, repeated secrets, JSON-escaped values, environment/configuration fields, and proof that debug/error serialization contains no source secret.
5. **Acceptance:** both existing process paths use the same redactor; a seeded secret cannot appear in returned diagnostics, partial failure artifacts, reports, or test snapshots; output is deterministic and bounded.
6. **Dependencies:** R3-00 and ADR-052.
7. **Risks and rollback:** over-redaction may remove useful diagnostics; retain stable reason codes and safe fingerprints, and roll back consumers without restoring raw secret output.

### R3-02 — Fail-closed sandbox capability probe

1. **Objective:** replace executable-presence assumptions with an executable host capability check.
2. **Files and crates:** `hunteval-sandbox`; capability fixtures; CLI `system check`; Security job and operations documentation.
3. **Contracts:** `SandboxBackendId`, `SandboxRequirement`, `SandboxCapability`, `SandboxCapabilityReport`, stable unavailable reason codes, schema 0.5 report, text/JSON CLI projection.
4. **Tests:** missing Bubblewrap, non-executable backend, unsupported platform, namespace denial, read-only mount, hidden-path denial, network denial, process-tree probe, each resource-limit probe, malformed backend version output, and bounded diagnostics.
5. **Acceptance:** the exact CI host returns every required capability; overriding the backend with a missing or non-conforming executable fails; no scored process starts after a failed capability report.
6. **Dependencies:** R3-00, ADR-047, and R3-01 for safe diagnostics.
7. **Risks and rollback:** container hosts vary in namespace support; document the single supported baseline and fail closed rather than weakening requirements or skipping tests.

### R3-03 — Versioned operating-system execution policy

1. **Objective:** make every enforceable process limit explicit, comparable, and content-addressed.
2. **Files and crates:** domain-neutral authored/resolved configuration adapters, benchmark resolver, schemas/v0.5, canonical examples, contract and compatibility tests.
3. **Contracts:** `AuthoredExecutionPolicy`, `ResolvedExecutionPolicy`, typed positive limit wrappers, denied network mode, backend ID, exact policy digest, and comparison reason for policy mismatch. Existing episode budgets remain semantically distinct.
4. **Tests:** zero/overflow limits, unknown fields/backend/network modes, incompatible combinations, duplicate references, absolute/traversal/symlink paths, exact-byte digest changes, legacy schema behavior, canonical schema/Rust parity, and cell-identity change on policy or launcher change.
5. **Acceptance:** every hardened run binds an explicit policy and launcher digest before process start; comparison rejects policy drift; legacy execution cannot receive an implicit policy.
6. **Dependencies:** R3-00 and ADR-049.
7. **Risks and rollback:** a new policy can fragment comparison cells; preserve raw policy digests and reject non-equivalent cells rather than normalizing limits silently.

### R3-04 — Shared Linux sandbox adapter and launcher

1. **Objective:** create one infrastructure adapter for deployment and worker process construction without changing protocol behavior.
2. **Files and crates:** new `hunteval-sandbox` crate and launcher binary, workspace/dependency policy, release packaging, capability tests.
3. **Contracts:** `SandboxSpec`, `GuestMount`, `SandboxEnvironment`, `ResourceLimits`, `SandboxLauncherDigest`, and safe launch errors. Guest mounts are explicit, read-only by default, and cannot contain private roots.
4. **Tests:** mount traversal/symlink escape, duplicate guest paths, non-UTF-8 paths, executable replacement, environment-name/value bounds, missing system roots, read-only public root, isolated `/tmp`, denied network, and launcher hash stability.
5. **Acceptance:** one validated specification produces the Bubblewrap invocation and launcher configuration used by both future consumers; the dependency-direction and unsafe-code gates pass; release artifacts include and hash the launcher.
6. **Dependencies:** R3-01 through R3-03 and ADR-048.
7. **Risks and rollback:** extracting shared process code may destabilize working execution; land the adapter behind existing tests before migrating either production consumer, and retain the old adapter until parity is proven.

### R3-05 — Complete process-tree supervision and resource enforcement

1. **Objective:** enforce wall time, CPU time, memory/address space, file size, open files, process count, and bounded pipes for the full sandbox process tree.
2. **Files and crates:** `hunteval-sandbox` supervisor/launcher modules; adversarial helper modes; Linux integration tests.
3. **Contracts:** `SupervisedProcess`, terminal `ProcessOutcome`, typed `LimitKind`, measured termination cause, idempotent cancellation, and drop semantics. Limit values come only from `ResolvedExecutionPolicy`.
4. **Tests:** child and grandchild timeout, ignored signals, fork/pipe holders, CPU loop, memory allocation, file growth, descriptor exhaustion, process fan-out, stdout/stderr flood, simultaneous exit/timeout, repeated cancellation, supervisor drop, and no surviving PIDs.
5. **Acceptance:** every limit produces its typed terminal reason without terminating the controller; all descendants disappear within a bounded grace period; output remains redacted and bounded; probes run in Security CI rather than skip.
6. **Dependencies:** R3-04.
7. **Risks and rollback:** kernel limit semantics differ; support only the probed Linux baseline, distinguish configured from observed limits, and never fall back to runner-only accounting.

### R3-06 — Migrate deployment protocol transport

1. **Objective:** route the production bidirectional JSONL deployment session through the shared supervised sandbox.
2. **Files and crates:** runner `run/transport` and process modules, run error mapping, reference deployment helper modes, integration tests.
3. **Contracts:** transport consumes `ResolvedExecutionPolicy`; `RunFailureKind` gains stable sandbox/limit distinctions without exposing diagnostics; the run manifest records backend, policy, and launcher digests.
4. **Tests:** canonical three-topology sessions, slow lines, partial UTF-8, EOF, crash, timeout, output flood, message flood, grandchild survival attempt, executable swap, private-root/network probes, redacted stderr, and deterministic successful artifacts.
5. **Acceptance:** the existing 36-cell matrix remains byte-semantically equivalent except for declared new provenance fields; every process failure preserves a verifiable partial run; the old duplicated Bubblewrap builder is removed.
6. **Dependencies:** R3-05 and immutable protocol 0.3 fixtures.
7. **Risks and rollback:** transport changes can affect benchmark determinism; compare old/new semantic trajectories on canonical fixtures and roll back the migration without removing the shared adapter.

### R3-07 — Isolate the DuckDB worker and close R3.1

1. **Objective:** run each managed DuckDB worker through the same process-tree and resource boundary while exposing only verified public telemetry.
2. **Files and crates:** `hunteval-duckdb` process adapter, runner SQL router, guest-path mapper, worker tests, Security and End-to-end scripts.
3. **Contracts:** verified public table registrations map host files to safe read-only guest paths; worker policy/launcher digests join managed-tool provenance; SQL error codes remain backward compatible.
4. **Tests:** private-root and arbitrary-file probes, network probe, table symlink race, worker child/grandchild leak, memory/CPU/file/descriptor/process limits, timeout/crash/invalid output, valid query parity, and runner survival.
5. **Acceptance:** deployment and worker negative isolation tests run in CI; neither process can access evaluator roots; every resource breach is typed; reference queries remain deterministic; R3.1 roadmap exit conditions pass.
6. **Dependencies:** R3-05 and R3-06.
7. **Risks and rollback:** DuckDB native startup and memory mapping may conflict with limits; measure a conservative explicit policy, preserve SQL-level bounds as defense in depth, and never disable OS enforcement to recover performance.

## 7. R3.2 — Adversarial protocol testing

### R3-08 — Stateful protocol generators and property tests

1. **Objective:** test framing, session transitions, identities, budgets, provenance, and replay over generated bounded sequences.
2. **Files and crates:** protocol dev-dependencies, `tests/property_*`, reusable model/generator modules, deterministic regression seeds.
3. **Contracts:** a pure reference transition model maps generated actions to expected acceptance or stable `ProtocolErrorCode`; generators never call process, filesystem, provider, or evaluator code.
4. **Tests:** arbitrary byte framing, supported envelopes, invalid origins/phases, duplicate/unknown/future IDs, ownership, task/action/evidence/finding chains, budget edges, sequence overflow boundaries, predecessor hashes, truncation, and replay equivalence.
5. **Acceptance:** fixed seeds reproduce failures; accepted generated sessions replay to identical terminal projections and digests; rejected inputs never panic, allocate beyond bounds, or expose raw input in errors.
6. **Dependencies:** R3-00 and ADR-050.
7. **Risks and rollback:** unconstrained strategies can create slow or meaningless cases; use small composable generators, explicit case counts, shrinking bounds, and retained minimal examples.

### R3-09 — Fuzz targets and retained regression corpus

1. **Objective:** add bounded fuzz coverage without making ordinary builds depend on nightly Rust or nondeterministic campaigns.
2. **Files and crates:** `fuzz/` package, targets for decoder/session/replay/conformance input, minimized corpus directories, pinned tool documentation, `scripts/ci/r3-adversarial.sh`.
3. **Contracts:** fuzz input is untrusted bytes; targets have explicit input/operation limits, no external I/O, no provider calls, no ground truth, and no secret-bearing seed data.
4. **Tests:** bounded smoke runs for each target, corpus replay under stable Rust where possible, malformed UTF-8/JSON/hash/state seeds, crash minimization workflow, and seeded failure propagation for the CI script.
5. **Acceptance:** all targets complete the configured smoke budget without panic, hang, or unbounded allocation; a synthetic crashing target proves nonzero propagation; every corpus file has a documented public origin and size bound.
6. **Dependencies:** R3-08.
7. **Risks and rollback:** fuzz tooling can increase CI time or supply-chain surface; pin it, isolate it from the workspace and release binary, cap runtime, and keep deterministic corpus regression mandatory if exploratory jobs are disabled temporarily.

### R3-10 — Adversarial live-process protocol suite

1. **Objective:** verify runner survival and typed termination against hostile protocol peers, not only pure state-machine inputs.
2. **Files and crates:** reference deployment adversarial modes or dedicated test fixture binary, runner transport integration tests, adversarial CI entrypoint.
3. **Contracts:** fixture modes are test-only and identify slow writer, early EOF, invalid UTF-8, oversized frame, partial frame, duplicate/future reference, flood, stderr flood, crash, fork, and never-exit behavior.
4. **Tests:** each fixture mode, simultaneous protocol and process failures, deadline boundaries, backpressure, closed stdin/stdout, partial artifacts, redaction, process-tree cleanup, and controller continuation to later cells.
5. **Acceptance:** no fixture terminates or wedges the benchmark controller; every failure has one stable public category; later queued cells execute unless fail-fast was explicitly selected; partial artifacts verify consistently.
6. **Dependencies:** R3-06 and R3-09.
7. **Risks and rollback:** timing assertions can be flaky; assert bounded state transitions and cleanup rather than exact milliseconds, and retain pure deterministic equivalents for every transport case.

### R3-11 — Protocol compatibility fixtures

1. **Objective:** make every supported protocol minor and its rejection behavior independently testable.
2. **Files and crates:** `examples/contracts/protocol/`, compatibility manifest, protocol tests, schema fixtures, compatibility documentation.
3. **Contracts:** fixture index records protocol version, direction, expected terminal status or stable error code, exact transcript digest, and required capability set. Protocol 0.3 is initially the only accepted version.
4. **Tests:** canonical success for each topology, minimum/maximum negotiation, unknown newer minor, incompatible major, unknown type/field policy, malformed envelope, transcript tampering, deterministic replay, and fixture-index digest verification.
5. **Acceptance:** one command validates the complete supported-version inventory offline; adding or removing support requires fixture and compatibility-table changes; no adapter rewrites a stored transcript.
6. **Dependencies:** R3-08 and immutable existing protocol examples.
7. **Risks and rollback:** fixtures may accidentally freeze implementation details; assert public wire behavior and stable reason codes only, preserving runner-internal freedom.

### R3-12 — Third-party deployment conformance command and R3.2 closure

1. **Objective:** expose the bounded compatibility suite to external deployment executables without granting scored authority.
2. **Files and crates:** runner conformance application service, CLI deployment command, schema 0.5 result, canonical fake managed tool, operator documentation and CLI tests.
3. **Contracts:** `ConformanceCase`, `ConformanceCheck`, `ConformanceResult`, supported protocol inventory, safe deployment reference, deterministic text/JSON output, and stable exit codes. Conformance uses public synthetic inputs only.
4. **Tests:** conforming three reference topologies, unsupported version, bad registration, malformed frames, forbidden direct tool behavior, timeout/crash/flood, unsafe executable path, symlink replacement, unavailable sandbox, deterministic output, and no private fields.
5. **Acceptance:** `hunteval deployment conformance` runs offline through the production sandbox/transport, produces an auditable machine result, and cannot access ground truth or bypass managed tools; all R3.2 roadmap exit conditions pass.
6. **Dependencies:** R3-09 through R3-11 and R3-07 sandbox closure.
7. **Risks and rollback:** conformance may be mistaken for benchmark quality certification; label it protocol/safety compatibility only and make no investigation-performance claim.

## 8. R3.3 — Artifact verification and redaction

### R3-13 — Standalone run-verification contract and service

1. **Objective:** unify public integrity checks for completed and partial run directories behind one application service.
2. **Files and crates:** runner verification modules, safe bounded artifact reader, schema 0.5 result and examples, contract and integration tests.
3. **Contracts:** `RunVerificationResult`, `VerificationStatus`, ordered `VerificationCheck`, stable `VerificationReason`, checked artifact count/digests, supported schema/protocol versions, and explicit `private_evaluation: not_checked` in public mode.
4. **Tests:** safe regular directory, symlink/hard-link and traversal attacks where enforceable, missing/extra/oversized files, manifest mismatch, unsupported version, wrong run ID, digest changes, partial JSON/JSONL, broken hash chain, nonterminal trajectory, submission mismatch, metrics/result inconsistency, unsafe paths, race-resistant open behavior, and deterministic reason ordering.
5. **Acceptance:** valid completed runs pass; any mutated required artifact fails; partial runs return typed incomplete reasons; output contains no absolute/private paths or ground-truth values; verification never requires a provider or executes deployment content.
6. **Dependencies:** R3-00/ADR-051, R3-01 redaction, and existing replay/trusted-input/report verification behavior.
7. **Risks and rollback:** combining checks may overstate what is verified; expose check-level scope and never label private metric correctness as checked without a separate trusted evaluator input.

### R3-14 — `run verify` CLI and compatibility closure

1. **Objective:** make standalone verification scriptable and human-readable while preserving the existing run command syntax.
2. **Files and crates:** CLI parser/handler modules, runner exports, command documentation, JSON snapshots and exit-code tests.
3. **Contracts:** `hunteval run verify <directory> --format text|json`, deterministic JSON, concise text, zero for verified completed runs, nonzero for incomplete/invalid/unsupported runs, and no path disclosure in normalized output.
4. **Tests:** parser compatibility with `hunteval run --episode`, completed/partial/tampered directories, text/JSON parity, malformed options, inaccessible/symlink roots, stable exit codes, piped output, and no ANSI/host-dependent content in JSON.
5. **Acceptance:** the public 36-cell end-to-end job verifies selected run artifacts before benchmark reporting; changing any selected byte makes the command and job fail; existing CLI commands remain compatible.
6. **Dependencies:** R3-13.
7. **Risks and rollback:** restructuring Clap commands can break automation; add parser compatibility tests first and retain the old execution syntax throughout R3.

### R3-15 — Secret scanning, release integration, and R3 closure

1. **Objective:** apply centralized redaction and deterministic secret detection to repository and generated artifacts, then prove all R3 exit criteria end to end.
2. **Files and crates:** secret-scan module, schema 0.5 result, `scripts/ci/secret-scan.sh`, canonical CI Security/Package jobs, release checklist, README and R3 status evidence.
3. **Contracts:** versioned `SecretScanPolicy`, stable high-signal rule IDs, bounded `SecretScanFinding`, safe relative artifact labels, line/field locations, one-way fingerprints, allowlist entries with rationale and expiry, deterministic result ordering.
4. **Tests:** representative token/key/password formats, entropy false-positive fixtures, split/encoded/JSON values where declared supported, binary/oversized/symlink files, malicious filenames, redaction-before-reporting, generated run/report/log/RC scans, allowlist abuse, and proof that findings never print matched values.
5. **Acceptance:** tracked public artifacts and selected CI/release outputs scan clean; seeded secret fixtures fail locally and in CI; full quality, security, adversarial, end-to-end, clean-cache, package, and run-verification gates pass on the same revision; R3 completion evidence records exact commands and known limitations.
6. **Dependencies:** R3-01, R3-07, R3-12, and R3-14.
7. **Risks and rollback:** scanners can leak their own matches or block benign fixtures; hash matches before persistence, keep rules reviewable and high-signal, require bounded justified allowlists, and never disable scanning globally to unblock a release.

## 9. Delivery waves and dependency graph

Milestone numbers identify ownership. Implementation follows dependency order, and only one behavior-changing milestone is active at a time.

### Wave A — Freeze boundaries and make diagnostics safe

1. R3-00 contracts and ADRs.
2. R3-01 centralized redaction.
3. R3-02 capability probe and CLI.
4. R3-03 explicit execution policy.

### Wave B — Enforce the Linux process boundary

5. R3-04 shared sandbox adapter and launcher.
6. R3-05 full process-tree and resource enforcement.
7. R3-06 deployment transport migration.
8. R3-07 worker migration and R3.1 exit gate.

### Wave C — Make the protocol adversarially testable

9. R3-08 property model and generators.
10. R3-09 fuzz targets and corpus.
11. R3-10 hostile live-process suite.
12. R3-11 compatibility fixtures.
13. R3-12 conformance CLI and R3.2 exit gate.

### Wave D — Verify and scan every public artifact

14. R3-13 verification service.
15. R3-14 run-verification CLI.
16. R3-15 secret scanning and R3 release gate.

```text
R3-00 contracts and ADRs
  -> R3-01 redaction
  -> R3-02 capability probe
  -> R3-03 execution policy
       -> R3-04 sandbox adapter/launcher
            -> R3-05 process-tree/resource enforcement
                 -> R3-06 deployment transport
                      -> R3-07 DuckDB worker isolation

R3-00 protocol decisions
  -> R3-08 property model
       -> R3-09 fuzz/corpus
            -> R3-10 adversarial transport
       -> R3-11 compatibility fixtures
R3-07 + R3-09 + R3-10 + R3-11
  -> R3-12 deployment conformance

R3-00 verification decisions + R3-01 redaction
  -> R3-13 verification service
       -> R3-14 run verify CLI

R3-07 + R3-12 + R3-14
  -> R3-15 secret scanning and R3 closure
```

## 10. Milestone handoff checklist

Before completing any R3 milestone:

1. its objective and user-visible result are implemented without unrelated scope;
2. public contracts have schema, canonical example, validation, and compatibility coverage;
3. security and ground-truth-isolation impact is documented and negatively tested;
4. positive, negative, malformed-input, resource, and deterministic/replay tests pass;
5. first-party production code contains no unsafe, panic shortcuts, secret output, or unbounded input;
6. source files are split for cohesion before 300 lines where practical and remain below 500 lines;
7. exact focused commands and all canonical gates pass;
8. documentation, ADR status, migration behavior, rollback, and known limitations are current;
9. `git diff --check` passes and no private, generated, or unrelated file is tracked;
10. the milestone receives a descriptive commit and only then changes to `complete` with evidence.

Pushes do not combine unfinished milestones. A remote CI failure returns the milestone to active status until the same revision passes locally and remotely.

## 11. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| kernel or runner cannot enforce one required limit | scored isolation claim would be false | executable capability probes and fail-closed unsupported status; no skip or userspace-only fallback |
| process descendants survive controller timeout | host resource leak and cross-cell interference | PID namespace/process-tree tests, idempotent supervisor drop, bounded cleanup verification |
| sandbox extraction changes successful protocol bytes | comparison reproducibility regression | old/new semantic transcript parity before migration; preserve protocol 0.3 fixtures |
| low OS limits break DuckDB startup | managed tools become unavailable | explicit conservative policy measured in CI; keep SQL limits as defense in depth; never disable OS limits silently |
| property or fuzz tests become unbounded/flaky | unreliable CI and contributor friction | deterministic seeds, operation caps, bounded smoke budgets, minimized corpus regression |
| fuzz corpus contains secrets or private labels | repository disclosure | public synthetic corpus only, corpus scan, provenance metadata, code review |
| conformance is mistaken for quality certification | unsupported deployment claims | label result as protocol/safety compatibility only; exclude scoring and ground truth |
| verifier claims private metric correctness | false assurance | check-level scope and explicit `private_evaluation: not_checked` in public mode |
| artifact verification follows a changed symlink | integrity bypass | descriptor-based no-follow bounded reads and race tests; reject unsupported filesystem semantics |
| redaction misses an encoded secret | disclosure in diagnostics or reports | structured redaction first, documented supported encodings, secret scan defense in depth, no secret-bearing tests |
| scanner reports matched secret text | scanner becomes exfiltration path | persist only rule/location/fingerprint; adversarial output tests |
| additive schema breaks R2 artifacts | historical runs become unreadable | immutable 0.3/0.4 fixtures and explicit adapters; roll back new writer while retaining readers |

## 12. R3 completion definition

R3 is complete only when all of the following are true:

1. The supported Linux environment passes executable capability probes for namespace, filesystem, network, process-tree, and every declared resource guarantee.
2. Deployment and DuckDB worker processes share the hardened sandbox and cannot access private episode roots or network interfaces.
3. Timeout, cancellation, protocol failure, output overflow, and every resource breach remove the entire process tree without terminating the benchmark controller.
4. Execution policy and relevant launcher/binary hashes are explicit in resolved artifacts and comparison identity.
5. Property suites and retained fuzz regressions cover framing, state transitions, replay, identifiers, malformed input, and hash chains with deterministic seeds.
6. Slow writers, early EOF, duplicates, future references, floods, invalid UTF-8, crashes, and descendant processes have live transport tests with typed outcomes.
7. Every supported protocol minor has immutable positive and negative compatibility fixtures, and third-party deployments can run the public conformance command offline.
8. `run verify` detects malformed, partial, unsupported, missing, symlinked, digest-mismatched, chain-broken, and internally inconsistent public run artifacts.
9. Redaction and secret scanning cover process diagnostics, configuration fields, fixtures, snapshots, selected CI logs, generated reports, and RC inputs without emitting matched values.
10. The three reference deployment topologies and complete 36-cell benchmark still execute deterministically through HuntEval-managed tools.
11. No R3 result or diagnostic contains private ground truth, private chain of thought, raw secrets, private paths, or unsupported causal claims.
12. Local and GitHub Actions quality, security, adversarial, end-to-end, verification, and package gates pass on the same revision.

Completion evidence must record exact commands, supported host/backend versions, execution-policy hash, sandbox-launcher hash, compatibility-fixture index hash, fuzz corpus hashes, conformance result hash, benchmark manifest and input hashes, runner/worker hashes, normalized result digest, run-verification result hash, secret-scan policy/result hashes, known limitations, and ADR status changes.

## 13. Exact acceptance commands

Commands that exist before R3 remain mandatory:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/e2e.sh
git diff --check
```

Planned focused commands become mandatory when their owning milestone introduces them:

```bash
cargo test -p hunteval-sandbox
cargo test -p hunteval-runner --test isolation
cargo test -p hunteval-sandbox --test process_tree
cargo test -p hunteval-duckdb --test worker_failures
cargo test -p hunteval-duckdb --test worker_isolation

cargo test -p hunteval-protocol --test conformance
cargo test -p hunteval-protocol --test topology_conformance
cargo test -p hunteval-protocol --test property_framing
cargo test -p hunteval-protocol --test property_session
cargo test -p hunteval-protocol --test property_replay
./scripts/ci/r3-adversarial.sh

cargo test -p hunteval-runner --test run_verification
cargo test -p hunteval-cli --test vertical_slice
cargo test -p hunteval-reference-deployment --test conformance
./scripts/ci/secret-scan.sh
```

The bounded CI smoke uses `cargo-fuzz` 0.13.2 and `nightly-2026-02-12`. Longer campaigns may increase `-runs`, but the canonical adversarial gate executes 1,000 iterations per target:

```bash
cargo +nightly-2026-02-12 fuzz run jsonl_decoder -- -runs=1000
cargo +nightly-2026-02-12 fuzz run protocol_session -- -runs=1000
cargo +nightly-2026-02-12 fuzz run trajectory_replay -- -runs=1000
cargo +nightly-2026-02-12 fuzz run conformance_input -- -runs=1000
```

No command may silently skip because Bubblewrap, a resource capability, a corpus, or a required scanner is missing.
