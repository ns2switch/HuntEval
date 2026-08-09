# HuntEval schema contracts v0.6

Version 0.6 adds benchmark-science contracts for public episode classification, private dataset review, statistical policy and calibration, contributor validation, deployment topology, controlled topology experiments, equivalence, paired ablation observations, topology analysis, and controlled comparison reports.

Versions 0.3 through 0.5 remain immutable. A 0.6 reader does not rewrite older artifacts or infer unavailable fields. Unknown fields and unsupported versions fail closed.

Public artifacts cannot contain evaluator-only ground truth, reference answers, hidden partition membership, private paths, raw secrets, authorization changes, or private chain of thought. Canonical examples live under `examples/contracts/v0.6/` and validate offline.

`policies/dataset-review-v1.json` is the versioned human-review policy bound by every canonical R4 dataset approval. Any change to a public package, private ground truth, reference query, or the policy makes its approval stale.
