# HuntEval cloud v1 expanded candidate benchmark card

## Intended use

The expanded candidate corpus evaluates complete single-agent and multi-agent threat-hunting deployments over deterministic synthetic AWS, Azure, and Google Cloud control-plane investigations. It supports controlled comparisons of investigation quality, evidence, coordination, resilience, resource use, reproducibility, and topology-dependent effects.

The expanded benchmark manifest is `examples/cloud-expanded-benchmark.yaml`. Promotion to a new machine-readable official-pack root is pending independent review of the 36 new episodes and an additive resolution for the frozen pack schema, which currently requires exactly 18 episodes. The previous machine-readable root remains historical evidence and must not be relabeled as the expanded pack. Benchmark manifests, scoring profiles, deployments, episode packages, policies, and resolved cell inputs remain content-addressed by the existing benchmark resolver.

## Composition

- 54 episodes: 18 each for AWS, Azure, and Google Cloud;
- nine explicitly benign investigations containing plausible administrative or automation alternatives;
- 30 multi-stage investigations and 16 investigations spanning more than one account, subscription, tenant, or project scope;
- classified difficulty distribution of 12 introductory, 21 intermediate, and 12 advanced episodes; difficulty remains unavailable for nine preserved pre-R4 episodes rather than being inferred;
- investigation coverage spanning identity and credentials, permission change and persistence, cross-boundary movement, secrets and key management, storage and data access, serverless control planes, and managed container control planes;
- bounded event volumes: 30 small, 12 medium, and 12 large fixtures;
- three reference deployments: single-agent, two-agent, and supervisor-specialist;
- seeds 11 and 29 with two declared repetitions;
- one explicit versioned scoring profile; raw metric vectors remain authoritative;
- deterministic public telemetry with physically separate evaluator ground truth.

## Security and isolation

Public observations never contain ground truth. Evaluator-private files are not included in public release packages, deployment mounts, public reports, or uploaded CI artifacts. Dataset review, answer-leakage scanning, secret scanning, hash verification, deterministic reference recovery, managed-tool execution, sandboxing, and report verification remain mandatory. The 36 new episodes have content-addressed review bundles but remain ineligible until independent human approvals bind the exact public package, private truth, reference query, and review policy.

## Limitations

The corpus is synthetic, offline, and bounded; it does not establish universal real-world SOC performance or represent production SIEM execution. Provider and service coverage remains incomplete. It does not authorize network access or establish universal performance for agents, models, frameworks, topologies, vendors, commercial products, or cloud providers. The reference deployments are not commercial-product benchmarks. Missing and unverifiable metrics remain unavailable and are never imputed. Topology attribution remains experimental and topology-dependent. Performance on this public corpus may not predict performance on private production data.

The frozen scripted reference deployments produce only five distinct outcome signatures over the 54 episodes and do not recover the evidence in the new malicious scenarios. The quality report therefore flags broad identical-result groups for individual review. This does not invalidate deterministic reference recovery, but it limits claims about discriminatory power until independently capable deployments are evaluated. The baselines have not been tuned using evaluator-private answers.

## Version and rollback

An official candidate identity is immutable. Any changed episode, deployment, policy, seed, budget, tool, scoring profile, schema, or manifest creates a new pack identity. The expanded corpus cannot reuse the previous candidate identity. A rejected or superseded candidate is retained as evidence and replaced by a new version; files are never replaced under the same identity.
