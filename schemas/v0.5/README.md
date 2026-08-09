# HuntEval schema contracts v0.5

Version 0.5 adds runner-hardening artifacts for explicit operating-system execution policy, fail-closed sandbox capability reporting, deployment protocol conformance, standalone run verification, and bounded secret scanning.

Versions 0.3 and 0.4 remain immutable and readable through explicit compatibility behavior. A 0.5 artifact never upgrades or rewrites an older stored artifact. Unknown fields and unsupported versions fail closed.

The public schemas are `execution-policy.schema.json`, `sandbox-capability-report.schema.json`, `protocol-conformance-result.schema.json`, `run-verification-result.schema.json`, and `secret-scan-result.schema.json`. None of these contracts may contain evaluator-only ground truth, raw secret values, private host paths, or private chain of thought.

Canonical examples live under `examples/contracts/v0.5/` and are validated offline. Schema references never require network access.
