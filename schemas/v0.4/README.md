# HuntEval schema contracts v0.4

Version 0.4 adds authored benchmark manifests, resolved cell identities, append-only benchmark events, deterministic benchmark state, comparison eligibility, executable deployment configuration, structured timelines, evaluator-only expected timeline windows, and typed report claims.

Version 0.3 files remain immutable and supported through explicit adapters. A 0.3 artifact is never rewritten or enriched by inference. Unknown fields and unsupported versions fail closed.

The public schemas are `benchmark-manifest.schema.json`, `benchmark-cell.schema.json`, `benchmark-event.schema.json`, `benchmark-state.schema.json`, `comparison-eligibility.schema.json`, `deployment-registration.schema.json`, `submission.schema.json`, and `report-claim.schema.json`. `ground-truth.schema.json` is evaluator-only and must never be mounted into or delivered to an evaluated deployment.

Canonical examples live under `examples/contracts/v0.4/` and are validated in the domain contract test suite. Schema references are resolved from the repository during tests; validation never fetches a network resource.
