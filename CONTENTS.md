# Package contents

All tracked source code, documentation, schemas, fixtures, and examples are written in English.

## Repository policy and workspace

- `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, and `LICENSE`
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and `deny.toml`
- `README.md` and this package index

## Rust crates

- `crates/hunteval-domain/`: infrastructure-independent identifiers, contracts, benchmark definitions, and results
- `crates/hunteval-protocol/`: bounded JSONL protocol sessions, framing, conformance, and replay
- `crates/hunteval-runner/`: mediated run orchestration, benchmark execution, journals, recovery, evaluation inputs, and stability aggregation
- `crates/hunteval-evaluation/`: trusted evaluation view, metric contracts, registry-backed scoring profiles, and constraints
- `crates/hunteval-statistics/`: paired comparison, ranking, and cross-run stability primitives
- `crates/hunteval-duckdb/`: isolated read-only DuckDB policy and worker
- `crates/hunteval-cli/`: public run, trajectory, report, and benchmark commands
- `crates/hunteval-reference-deployment/`: deterministic executable reference topologies
- `crates/hunteval-reporting/`: deterministic JSON and static HTML run reporting
- `crates/hunteval-fixture-tool/`: deterministic episode package generation and validation
- `crates/hunteval-knowledge/`: bounded optional local knowledge retrieval
- `crates/hunteval-resilience/`: deterministic fault scheduling and resilience evaluation

## Documentation

- `docs/SPECIFICATION.md`, `docs/ADR.md`, `docs/CONTRACTS.md`, and `docs/THREAT_MODEL.md`
- `docs/METRICS_AND_RANKING.md` and `docs/PROMPT_OPTIMIZATION.md`
- `docs/IMPLEMENTATION_PLAN.md` and `docs/EXECUTION_PLAN.md`: completed original MVP plans
- `docs/ROADMAP.md` and `docs/R2_IMPLEMENTATION_PLAN.md`: current roadmap and delivery evidence through R2-18
- `docs/BENCHMARK_CLI.md` and `docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md`: operational benchmark reference and end-to-end example

## Contracts, fixtures, and examples

- `schemas/v0.3/`: immutable original public contract schemas
- `schemas/v0.4/`: benchmark, report-claim, and scoring-profile schemas with explicit compatibility boundaries
- `datasets/`: deterministic AWS, Azure, and Google Cloud episode packages with physically separated public telemetry and private evaluator ground truth
- `deployments/`: single-agent, two-agent, and supervisor-specialist reference registrations
- `examples/`: benchmark manifests, versioned scoring profiles, and canonical contract fixtures
- `scripts/`: dependency-direction and source-size quality checks
