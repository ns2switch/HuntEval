# HuntEval

HuntEval is an open-source framework for reproducibly evaluating multi-agent systems applied to threat-hunting scenarios in cloud environments.

The evaluated unit is a complete **deployment**, not an isolated model. A deployment may include one or more agents, prompts, models, tools, memory layers, coordination policies, and runtime configuration. HuntEval measures which implementation performs best and explains the trade-offs across investigation quality, evidence quality, coordination, resilience, efficiency, and reproducibility.

The diagnostic layer uses observable run traces to classify supported failures and propose concrete improvement hypotheses. Those recommendations remain unvalidated until controlled experiments satisfy regression and verified-cost constraints.

## MVP scope

- Rust core.
- CLI-first interface.
- DuckDB and Parquet as the canonical local analytics environment.
- Initial scenarios for AWS, Microsoft Azure, and Google Cloud.
- Evaluation of single-agent and multi-agent deployments.
- HuntEval-managed tool execution during scored runs.
- Ground truth hidden from the evaluated deployment.
- Structured recording of agents, tasks, messages, tool calls, evidence, hypotheses, and findings.
- Configurable scoring profiles and statistical deployment comparison.
- Optional RAG for knowledge supplied by the hunt author and, later, for querying HuntEval-generated reports.

## Implementation status

The executable PR-00 through PR-15 plan and R2 phases A through D (R2-00 through R2-18) are complete. Repository-owned quality, security, adversarial, end-to-end, benchmark-science, cache-parity, and release-candidate gates are enforced by protected GitHub settings. The external R2.4 caveat was closed on 2026-08-09 by the administrator attestation and protected `v0.4.0-rc.1` dry run. Authored v0.3 and v0.4 benchmark manifests resolve into infrastructure-independent definitions whose stable cell identities include configuration, episode, scoring profile, seed, optional fault profile, runtime binaries, and schema bytes. The three reference topologies are independently executable deterministic JSONL peers. The matrix service runs them through the networkless sandbox, mediates scored SQL through the isolated DuckDB worker, schedules bounded parallel work, records append-only attempts, resumes interruptions without overwriting history, and verifies normalized result digests before declaring a comparison eligible. Metric inputs are reduced from digest-verified stored trajectories and submissions with explicit action-to-evidence-to-finding provenance; evaluator-only ground truth never enters the serializable observation view. Investigation, evidence, coordination, efficiency, and cross-run stability metrics use deterministic structured contracts. Versioned v0.4 scoring profiles select registered metrics, obtain direction from the registry, preserve explicit missing-value policy, and require verified provenance for hard resource constraints. Immutable v0.3 profiles remain readable through an in-memory compatibility adapter.

Benchmark reporting now normalizes cells, metrics, scoring omissions, constraints, resource provenance, paired statistics, ranking groups, claims, limitations, and exact artifact digests into deterministic JSON. The same validated result renders as portable, script-free HTML. Report generation detects run and benchmark inputs from validated artifacts, writes atomically, and provides an offline verification command that fails when a referenced artifact changes.

Deterministic diagnosis emits only classifications supported by observable event or metric references. Improvement recommendations identify affected runs and remain unvalidated with mandatory human review. Controlled experiments change exactly one variable, isolate hidden-test feedback, preserve immutable authorization, tool-access, and data-handling sections, and enforce metric-regression and verified-cost constraints.

The original MVP sequence is recorded in `docs/EXECUTION_PLAN.md`. Current R2 delivery status, commit evidence, dependencies, and acceptance gates are maintained in `docs/R2_IMPLEMENTATION_PLAN.md`.

R3 hardening is complete. The implementation adds an explicit schema 0.5 execution policy, executable Linux capability probes, one supervised Bubblewrap boundary for deployments and managed DuckDB workers, protocol property/fuzz/conformance coverage, standalone run verification, centralized bounded redaction, and deterministic secret scanning. The implementation revision `2d34517` passed all eight GitHub Actions jobs in run `31305219082`; exact local and remote completion evidence is recorded in `docs/R3_COMPLETION_EVIDENCE.md`. R2 implementation evidence remains unchanged; its later external-enforcement closure is recorded separately in `docs/GITHUB_SETTINGS_ATTESTATION.md`.

R4 is complete. Schema 0.6 benchmark-science contracts, benign and extended cloud fixtures, nine content-addressed dataset approvals, policy-bound statistics, bounded contributor workflows, normative reference topology artifacts, controlled-variable equivalence, topology-aware metrics, and deterministic topology reports are implemented. The 108-cell E2E produces an auditable controlled topology experiment and JSON/HTML reports. Implementation revision `f9559a6` passed all nine GitHub Actions jobs in run `31321445726`; exact closure evidence is recorded in `docs/R4_COMPLETION_EVIDENCE.md`. The governed R5 implementation plan is defined; R5 remains the next implementation milestone and no R5 implementation is complete.

## Quick start

Build the workspace so the CLI and managed worker are sibling executables, then run the offline reference slice:

```bash
cargo build --workspace
cargo run -p hunteval-cli -- run \
  --episode datasets/aws/aws-iam-001 \
  --deployment deployments/two-agent-scripted
cargo run -p hunteval-cli -- trajectory inspect runs/latest/trajectory.jsonl
cargo run -p hunteval-cli -- report generate runs/latest --format html
cargo run -p hunteval-cli -- benchmark validate examples/cloud-mvp-benchmark.yaml
cargo run -p hunteval-cli -- benchmark run examples/cloud-mvp-benchmark.yaml \
  --output runs/cloud-mvp --jobs 2
cargo run -p hunteval-cli -- benchmark status runs/cloud-mvp --format json
cargo run -p hunteval-cli -- benchmark compare runs/cloud-mvp \
  --left single-agent-scripted --right two-agent-scripted
cargo run -p hunteval-cli -- report generate runs/cloud-mvp --format json
cargo run -p hunteval-cli -- report generate runs/cloud-mvp --format html
cargo run -p hunteval-cli -- report verify runs/cloud-mvp --format json
cargo run -p hunteval-cli -- system check --format json
cargo run -p hunteval-cli -- deployment conformance \
  target/debug/hunteval-reference-deployment --format json -- \
  --topology supervisor-worker
cargo run -p hunteval-cli -- run verify runs/latest --format json
cargo run -p hunteval-fixture-tool -- scaffold \
  --provider aws --episode-id aws-iam-900 contributions/aws-iam-900
# A contributed episode remains incomplete until an independent review record exists.
cargo run -p hunteval-fixture-tool -- validate datasets/aws/aws-iam-004
cargo run -p hunteval-fixture-tool -- document datasets/aws/aws-iam-004
cargo run -p hunteval-fixture-tool -- review-bundle datasets/aws/aws-iam-004
cargo run -p hunteval-cli -- dataset review-record datasets/aws/aws-iam-004 \
  --review-policy policies/dataset-review-v1.json \
  --review-id review-aws-iam-004-r4 --reviewer-id reviewer-001 \
  --reviewed-at 2026-08-09T00:00:00Z --confirm-independent-approval
./scripts/ci/secret-scan.sh
```

## Development

The workspace pins Rust `1.93.1`. Run the same authoritative gates used by GitHub Actions:

```bash
cargo run -p hunteval-cli -- --version
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/r3-adversarial.sh
./scripts/ci/e2e.sh
```

See `CONTRIBUTING.md` for security, Clean Architecture, readability, and review requirements.

Regenerate the canonical AWS fixture with:

```bash
cargo run -p hunteval-fixture-tool -- generate datasets/aws/aws-iam-001
```

## Explicitly out of scope for the initial release

- Self-RAG as the primary object of evaluation.
- Collection of private chain of thought.
- Direct execution against production SIEM platforms.
- Fully autonomous prompt optimization without experimental validation.
- A fixed universal score embedded in the codebase.
- A web dashboard, Kubernetes deployment, or distributed control plane.

## Documentation map

- `docs/SPECIFICATION.md`: functional and technical specification.
- `docs/ADR.md`: architecture decision records.
- `docs/CONTRACTS.md`: domain contracts and JSONL process protocol.
- `docs/METRICS_AND_RANKING.md`: metrics, scoring profiles, statistics, and ranking.
- `docs/PROMPT_OPTIMIZATION.md`: failure diagnosis and prompt improvement workflow.
- `docs/THREAT_MODEL.md`: threats against the framework and evaluated deployments.
- `docs/IMPLEMENTATION_PLAN.md`: milestones and acceptance criteria.
- `docs/EXECUTION_PLAN.md`: executable pull-request sequence, contracts, tests, and quality gates.
- `docs/ROADMAP.md`: prioritized post-MVP releases, dependencies, and exit criteria through v1.0.
- `docs/BENCHMARK_CLI.md`: benchmark execution, recovery, comparison, output, and exit-code reference.
- `docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md`: end-to-end example comparing three reference topologies over the current cloud benchmark matrix.
- `docs/R2_IMPLEMENTATION_PLAN.md`: canonical delivery status, dependency order, implementation steps, and acceptance gates through R2-18.
- `docs/R3_IMPLEMENTATION_PLAN.md`: delivery status, contracts, tests, risks, and acceptance gates for R3.1 through R3.3.
- `docs/R3_COMPLETION_EVIDENCE.md`: exact R3 revisions, gates, artifact hashes, limitations, and remote CI evidence.
- `docs/R4_IMPLEMENTATION_PLAN.md`: governed delivery sequence, contracts, tests, risks, and acceptance gates for R4.1 through R4.4.
- `docs/R4_COMPLETION_EVIDENCE.md`: exact R4 revision, gates, approvals, artifact hashes, limitations, and remote CI evidence.
- `docs/R5_IMPLEMENTATION_PLAN.md`: governed delivery sequence, contracts, tests, risks, and acceptance gates for R5.1 through R5.3.
- `docs/GITHUB_OPERATIONS.md`: required GitHub controls, verification, runner trust, retention, and rollback.
- `docs/GITHUB_SETTINGS_ATTESTATION.md`: administrator-owned evidence for live repository settings.
- `docs/RELEASE_CHECKLIST.md`: non-publishing release-candidate dry-run procedure and evidence requirements.
- `AGENTS.md`: permanent development-agent instructions.

## Short definition

> HuntEval is an open-source framework for evaluating, comparing, diagnosing, and improving multi-agent threat-hunting deployments against reproducible cloud-security scenarios.

## Project principles

1. **Evidence over narrative.** Findings must be traceable to observable telemetry and tool results.
2. **Deployment-level evaluation.** Architecture and coordination are part of the system being tested.
3. **Framework neutrality.** HuntEval must not require a particular agent SDK or LLM provider.
4. **Reproducibility.** Datasets, prompts, configurations, binaries, and random seeds are versioned or hashed.
5. **Safe evaluation.** Scored tools are mediated by HuntEval and ground truth is isolated.
6. **Transparent ranking.** Metric vectors remain available even when an aggregate score is calculated.
7. **Validated improvement.** Prompt recommendations are hypotheses until confirmed by controlled experiments.
