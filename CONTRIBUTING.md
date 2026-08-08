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
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/e2e.sh
```

These repository-owned scripts are the authoritative local and GitHub Actions entrypoints. Rust `1.93.1`, the `rustfmt` and `clippy` components, cargo-deny `0.20.2`, and executable Bubblewrap at `/usr/bin/bwrap` are required. Missing capabilities fail closed. `./scripts/ci/test-failure-propagation.sh` demonstrates that seeded policy, security, and end-to-end failures return a nonzero status.

Hand-written production Rust files must not exceed 500 lines. Files above 300 lines require a documented cohesion review. Do not use unsafe first-party Rust or weaken authorization, provenance, ground-truth isolation, or managed-tool policies.

## Pull requests

Describe the objective, affected contracts, tests, security impact, commands executed, limitations, and rollback considerations. Keep mechanical refactoring separate from behavior changes and use English for project artifacts.

Changes to workflows, canonical CI scripts, schemas, datasets, security policy, or release controls require CODEOWNER review. Repository administrators must also follow `docs/GITHUB_OPERATIONS.md`; committed files do not replace live branch and tag protection.
