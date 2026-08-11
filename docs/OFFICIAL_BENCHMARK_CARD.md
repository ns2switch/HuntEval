# HuntEval cloud v1 candidate benchmark card

## Intended use

The candidate pack evaluates complete single-agent and multi-agent threat-hunting deployments over deterministic synthetic AWS, Azure, and Google Cloud IAM investigations. It supports controlled comparisons of investigation quality, evidence, coordination, resilience, resource use, reproducibility, and topology-dependent effects.

The machine-readable root is `examples/contracts/v1.0/official-benchmark-pack.json`. The benchmark manifest, scoring profile, deployments, episode packages, policies, and resolved cell inputs remain content-addressed by the existing benchmark resolver.

## Composition

- 18 episodes: six per cloud provider;
- three reference deployments: single-agent, two-agent, and supervisor-specialist;
- seeds 11 and 29 with two declared repetitions;
- one explicit versioned scoring profile; raw metric vectors remain authoritative;
- deterministic public telemetry with physically separate evaluator ground truth.

## Security and isolation

Public observations never contain ground truth. Evaluator-private files are not included in public release packages, deployment mounts, public reports, or uploaded CI artifacts. Dataset review, secret scanning, hash verification, managed-tool execution, sandboxing, and report verification remain mandatory.

## Limitations

The pack is synthetic and IAM-focused. It does not represent production SIEM execution, authorize network access, or establish universal performance for agents, models, frameworks, topologies, vendors, or cloud providers. Missing and unverifiable metrics remain unavailable and are never imputed. Topology attribution remains experimental and topology-dependent.

## Version and rollback

The candidate identity is immutable. Any changed episode, deployment, policy, seed, budget, tool, scoring profile, schema, or manifest creates a new pack identity. A rejected candidate is retained as evidence and replaced by a new version; files are never replaced under the same identity.
