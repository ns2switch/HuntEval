# R8 local implementation evidence

## Status

R8 implementation has reached every planned local boundary, but R8 is not complete. Protected CI and the four-target `v0.8.0-rc.5` rehearsal subsequently passed and are recorded in `R8_CANDIDATE_EVIDENCE.md`. R8-06 and R8-07 still require reviewers independent from the implementation author. R8-08, R8-10, and R8-11 require those reviews and a final evidence-bound candidate.

## Implemented locally

- R8-00 through R8-02: versioned interface inventory, deterministic freeze manifest, compatibility matrix and documentation projection, migration inventory, typed rejection, and content-addressed receipts.
- R8-03: deterministic bounded package inventory, normalized build provenance, SPDX 2.3 SBOM, dependency and license reports, release-manifest root, checksums, secret-aware package policy, and two-build comparison.
- R8-04: detached Ed25519 SSH signatures, exact public-key fingerprint, repository/ref/workflow identity, validity interval, revocation list, offline verification, and tamper/substitution tests. Rehearsal keys are ephemeral and are not production release identities.
- R8-05: versioned four-target matrix; deterministic tar/ZIP packaging; safe atomic installation into a new absolute destination; traversal, link, encryption, collision, and expansion rejection; fixed permissions; required CLI/schema/document checks; and native candidate evidence signed per target. Only Linux x86_64 is a scored candidate; both macOS architectures and Windows x86_64 remain preview.
- R8-08 foundation: versioned official cloud-pack root, content-addressed benchmark manifest and scoring profile, benchmark card, declared counts and seeds, and explicit exclusion of evaluator-private bytes from candidate packages.
- R8-09: operator build, verification, installation, removal, upgrade, migration, rollback, incident, disclosure, and known-limitation procedures.
- R8-10 foundation: a read-only GitHub workflow and `r8-release.sh` build, verify, install, smoke, sign, and offline-verification rehearsal without publication authority.
- R8-11 foundation: a strict versioned evidence index and closure verifier that rejects pending dependencies, milestones, reviews, checks, candidate evidence, or artifact hashes.

## Focused local commands

```text
./scripts/ci/r8-compatibility.sh
./scripts/ci/r8-supply-chain.sh
./scripts/ci/r8-signature-fixtures.sh
./scripts/ci/r8-package.sh
./scripts/ci/test-failure-propagation.sh
```

All focused commands pass. The closure fixture demonstrates both sides of the policy: the current evidence is rejected with exact blocking reasons, while a complete synthetic evidence set passes. Synthetic passing evidence is a verifier test and is not project completion evidence.

The complete local quality suite also passes on 2026-08-11: formatting, workspace Clippy with warnings denied, all workspace tests and documentation tests, security policy, R4 through R7 milestone gates, both pre-R8 connector gates, and the end-to-end benchmark. The end-to-end suite was run with `CARGO_BUILD_JOBS=1` because the four-gigabyte local executor exhausted memory during a parallel linker invocation; this changes build concurrency, not test scope or behavior.

The expanded native packager also completed an optimized Linux x86_64 build, public-package secret scan, deterministic tar assembly, supply-chain verification, wheel inspection, clean installation, CLI smoke test, root-manifest construction, detached rehearsal signature, and offline signature verification. An initial run failed closed because the old schema copy included `ground-truth.schema.json`; the package allowlist was corrected to exclude those evaluator-private schema files and the complete rehearsal then passed. Because this behavioral rehearsal used the dirty local worktree escape hatch, it is not immutable candidate or closure evidence.

## External closure blockers

1. The sole implementation author cannot independently review the same implementation. Separate security and reproducibility reviewer identities and evidence are required.
2. A final immutable candidate must bind those review records and pass the protected checks and native matrix on the resulting exact revision.
3. v0.7.1 and v0.7.2 remain pending under separate release gates. Their interfaces remain outside the stable freeze and cannot be represented as supported through R8 closure.

The earlier external blockers for protected CI, native packaging, and the initial immutable rehearsal were resolved by runs `31520408340` and `31522997192`. Roadmap governance explicitly revised the pre-R8 dependency without completing the pending connector releases.

No external evidence has been invented, no R8 milestone is marked complete, and v1.0 has not been published.
