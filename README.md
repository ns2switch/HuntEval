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

The executable PR-00 through PR-15 plan and the operational R2.1 MVP through R2-06 are complete. Authored v0.3 and v0.4 benchmark manifests resolve into infrastructure-independent definitions whose stable cell identities include configuration, episode, scoring profile, seed, optional fault profile, runtime binaries, and schema bytes. The three reference topologies are independently executable deterministic JSONL peers. The matrix service runs them through the networkless sandbox, mediates scored SQL through the isolated DuckDB worker, schedules bounded parallel work, records append-only attempts, resumes interruptions without overwriting history, and verifies normalized result digests before declaring a comparison eligible.

Deterministic diagnosis emits only classifications supported by observable event or metric references. Improvement recommendations identify affected runs and remain unvalidated with mandatory human review. Controlled experiments change exactly one variable, isolate hidden-test feedback, preserve immutable authorization, tool-access, and data-handling sections, and enforce metric-regression and verified-cost constraints.

The implementation sequence and milestone evidence are maintained in `docs/EXECUTION_PLAN.md`.

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
```

## Development

The workspace uses stable Rust. Verify the bootstrap with:

```bash
cargo run -p hunteval-cli -- --version
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
./scripts/check-dependency-direction.sh
./scripts/check-source-size.sh
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
- `docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md`: end-to-end example comparing two deployments over the 36-cell cloud MVP matrix.
- `docs/R2_IMPLEMENTATION_PLAN.md`: canonical delivery status, dependency order, implementation steps, and acceptance gates through R2-18.
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
