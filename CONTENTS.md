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
- `crates/hunteval-sandbox/`: fail-closed Linux capability probes, process supervision, resource policies, redaction, and secret scanning
- `crates/hunteval-commercial/`: finite read-only commercial connector catalogs, network policy, secret references, and transport boundary
- `crates/hunteval-release/`: versioned R8 interface inventory, fail-closed freeze eligibility, and deterministic freeze-manifest derivation

## Documentation

- `docs/SPECIFICATION.md`, `docs/ADR.md`, `docs/CONTRACTS.md`, and `docs/THREAT_MODEL.md`
- `docs/METRICS_AND_RANKING.md` and `docs/PROMPT_OPTIMIZATION.md`
- `docs/IMPLEMENTATION_PLAN.md` and `docs/EXECUTION_PLAN.md`: completed original MVP plans
- `docs/ROADMAP.md`, `docs/R2_IMPLEMENTATION_PLAN.md`, `docs/R3_IMPLEMENTATION_PLAN.md`, and `docs/R3_COMPLETION_EVIDENCE.md`: current roadmap and completed R2/R3 plans and evidence
- `docs/R4_IMPLEMENTATION_PLAN.md` through `docs/R7_IMPLEMENTATION_PLAN.md`: completed milestone plans and their associated evidence
- `docs/PRE_R8_CONNECTOR_IMPLEMENTATION_PLAN.md`, `docs/V071_FRAMEWORK_CONNECTOR_PLAN.md`, and `docs/V072_COMMERCIAL_CONNECTOR_PLAN.md`: in-progress framework, MCP, and commercial connector delivery sequence
- `docs/R8_IMPLEMENTATION_PLAN.md`, `docs/R8_00_IMPLEMENTATION_EVIDENCE.md`, `docs/R8_WAVE_A_IMPLEMENTATION_EVIDENCE.md`, and `docs/R8_LOCAL_IMPLEMENTATION_EVIDENCE.md`: governed R8 delivery and exact local implementation state
- `docs/R8_COMPATIBILITY.md`, `docs/R8_OPERATIONS.md`, `docs/OFFICIAL_BENCHMARK_CARD.md`, and the independent-review templates: compatibility, candidate operation, benchmark, and external review procedures
- `docs/FRAMEWORK_CONNECTORS.md`, `docs/COMMERCIAL_CONNECTORS.md`, and `docs/CONNECTOR_SUPPORT_MATRIX.md`: connector behavior, security boundaries, and exact support status
- `docs/BENCHMARK_CLI.md` and `docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md`: operational benchmark reference and end-to-end example
- `docs/GITHUB_OPERATIONS.md`, `docs/GITHUB_SETTINGS_ATTESTATION.md`, and `docs/RELEASE_CHECKLIST.md`: delivery controls, administrator evidence, and non-publishing release-candidate procedure

## Contracts, fixtures, and examples

- `schemas/v0.3/`: immutable original public contract schemas
- `schemas/v0.4/`: benchmark, report-claim, and scoring-profile schemas with explicit compatibility boundaries
- `schemas/v0.5/`: execution-policy, capability, conformance, run-verification, and secret-scan schemas
- `schemas/v1.0/`: R8 freeze, compatibility, migration, native-platform, supply-chain, signature, benchmark-pack, and closure-evidence schemas
- `datasets/`: deterministic AWS, Azure, and Google Cloud episode packages with physically separated public telemetry and private evaluator ground truth
- `deployments/`: single-agent, two-agent, and supervisor-specialist reference registrations
- `examples/`: benchmark manifests, versioned scoring profiles, and canonical contract fixtures
- `fuzz/`: isolated bounded protocol fuzz targets and synthetic public regression corpora
- `scripts/`: dependency-direction, source-size, canonical CI, security, adversarial, end-to-end, compatibility, native-platform packaging, supply-chain, signature, safe-install, closure, and non-publishing release-candidate checks
