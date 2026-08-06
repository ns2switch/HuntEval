# Contributing to HuntEval

HuntEval welcomes focused changes that preserve its role as an evaluation framework for threat-hunting deployments.

## Development workflow

1. Read `AGENTS.md`, the accepted ADRs, and `docs/EXECUTION_PLAN.md`.
2. Work on one approved pull-request objective at a time.
3. Add positive, negative, and security tests for changed behavior.
4. Keep domain code independent from infrastructure adapters.
5. Update public contracts and documentation with the implementation.
6. Run every required quality gate before requesting review.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo deny check
./scripts/check-dependency-direction.sh
./scripts/check-source-size.sh
```

Hand-written production Rust files must not exceed 500 lines. Files above 300 lines require a documented cohesion review. Do not use unsafe first-party Rust or weaken authorization, provenance, ground-truth isolation, or managed-tool policies.

## Pull requests

Describe the objective, affected contracts, tests, security impact, commands executed, limitations, and rollback considerations. Keep mechanical refactoring separate from behavior changes and use English for project artifacts.
