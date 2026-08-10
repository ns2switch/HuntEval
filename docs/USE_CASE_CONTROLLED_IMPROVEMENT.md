# Use case: evidence-backed controlled improvement

## Goal

An operator observes recurrent duplicate task creation in a supervisor/specialist deployment and wants to test whether an explicit task-ownership rule improves the deployment without weakening safety policy or hiding a regression behind one aggregate score.

## Workflow

1. Run and verify the baseline benchmark, then generate its R5 diagnostic bundle.
2. Register the exact supervisor instruction bytes and an explicit ordered section inventory.
3. Resolve the `duplicate_task_creation` diagnosis to the compiled `missing_task_ownership` weakness rule using exact run, task, coordination, and artifact references.
4. Produce a separate proposed `add_constraint` suggestion against the mutable delegation section. HuntEval does not edit the baseline or active deployment.
5. Materialize the proposal explicitly as new bytes, register the new digest, and generate the structural diff.
6. Reject the candidate before execution if an immutable section changed, immutable coverage is incomplete, the candidate contains known benchmark-answer material, or more than the declared artifact variable changed.
7. Run baseline and candidate through the same paired benchmark matrix. Episode set, seeds, budgets, models, topology, managed-tool policy, scoring profile, statistical policy, schemas, execution policy, and binaries remain content-addressed controls.
8. Evaluate the raw paired metric vector and every declared quality, regression, resilience, resource, and verified-cost constraint. Missing and unverifiable values remain explicit.
9. If the controlled decision passes, record a `validated` lifecycle event for the exact candidate. This is experimental support under the declared controls, not universal causal proof.
10. Review the evidence and candidate diff. An explicit human approval may advance the recommendation to `approved`.
11. If the operator changes the external deployment independently, record that separately confirmed action as `adopted`. HuntEval never performs the deployment change.
12. Generate and verify the normalized JSON, static HTML, and content-addressed bundle offline.

## What the result means

A passing result supports the exact candidate for the declared benchmark, topology, models, policies, budgets, partitions, and binaries. It does not prove transfer to another deployment. Coordination overhead and resource trade-offs remain separate from investigation quality, and the raw metric vector remains authoritative.

If candidate or control bytes change, the previous validation, approval eligibility, and adoption eligibility are invalidated. Hidden-test membership and episode-level feedback remain unavailable during candidate generation and selection.

## Safety boundaries

- Immutable authorization, tool-access, filesystem, network, data-handling, ground-truth-isolation, benchmark-constraint, output-integrity, and security-control sections cannot be modified or reclassified.
- Suggested text is untrusted bounded data and cannot grant tool authority.
- Observational diagnosis alone cannot produce `validated`.
- Human review and external adoption are distinct explicit artifacts.
- No private chain of thought, production SIEM access, provider-specific optimizer, or autonomous adoption is introduced.
