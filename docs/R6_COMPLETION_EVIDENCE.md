# R6 local implementation evidence

## Status

R6 is implemented locally at revision `079bf459c72b4a3c3dcc23c0121482d260d2c84d`. It is not marked complete because the eleven canonical GitHub Actions jobs have not yet passed on the final documentation evidence revision.

R2, R3, R4, and R5 remain complete with their recorded evidence. Their commit references, GitHub Actions evidence, schemas, and exit criteria are unchanged. The former R2.4 external-enforcement caveat remains closed by its separate administrator attestation.

## Implemented behavior

- schema 0.8 typed contracts preserve schema 0.3 through 0.7 compatibility;
- bounded no-follow artifact registration stores exact bytes, deterministic inventories, sizes, media types, and SHA-256 identities;
- explicit structured sections support deterministic diffs while immutable removal, change, rename, or reclassification fails closed;
- the improvement policy owns all nine immutable safety classes and cannot enable hidden-test selection feedback or autonomous adoption;
- direct, fragmented, and hexadecimal known-answer leakage checks return only bounded safe reason codes;
- controlled equivalence binds the exact baseline, candidate, diff, policies, episode set, seeds, budgets, models, topology, managed tools, execution policy, schemas, and binaries;
- evaluator-only partition authorization rejects hidden-test selection and limits final assessment to one frozen candidate per lineage;
- paired execution delegates to the existing benchmark service and therefore retains its sandbox, managed-tool path, attempts, failures, resume semantics, and normalized results;
- constraint-first validation preserves raw paired measurements, missing pairs, applicability, provenance, intervals, violations, and unverifiable constraints without imputation;
- recommendation history is append-only and hash-linked; validation, explicit human review, and externally confirmed adoption remain distinct;
- changed candidate bytes generate an invalidation event and cannot retain validation or approval eligibility;
- the fourteen-category prompt/configuration taxonomy must match the compiled typed mapping registry exactly;
- prompt analysis consumes exact observable R5 sources, emits bounded hypotheses, and materializes suggestions separately without changing registered or active deployment bytes;
- normalized JSON is authoritative, static HTML escapes untrusted text and contains no scripts, and bundle verification checks every exact byte plus JSON/HTML projection equivalence;
- the additive CLI exposes `improvement validate`, `run`, `resume`, `status`, and `verify`; it exposes no autonomous adoption command;
- `scripts/ci/r6-improvement.sh` and the `Controlled improvement` GitHub Actions job provide the dedicated R6 gate.

## Local evidence

Toolchain: `rustc 1.93.1 (01f6ddf75 2026-02-11)`.

The following local checks passed before this evidence document was written:

```text
./scripts/ci/quality.sh
./scripts/ci/security.sh (all checks except the final dirty-tree scan; see below)
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/e2e.sh
```

The R6 gate covers schema and typed-contract compatibility, artifact registration, structural diff, immutable policy, direct and encoded leakage, equivalence, hidden-test isolation, canonical benchmark-backed orchestration, raw paired validation, lifecycle replay, human review, external adoption confirmation, invalidation, prompt analysis, safe materialization, JSON/HTML reporting, deterministic end-to-end regeneration, offline bundle verification, and CLI surface tests.

The first R6-gate attempt exhausted local linker memory while Cargo linked multiple DuckDB-dependent integration-test binaries concurrently. No test failed. The gate now defaults `CARGO_BUILD_JOBS` to `1`; the complete rerun passed. This changes only gate resource use, not runtime semantics.

Content identities at implementation revision:

| Artifact set | SHA-256 |
|---|---|
| Sorted schema 0.8 file-hash inventory | `aeba9cbdbb8c44f7a81dbd62c3f38a8749424de64299ac477a47035247a7d420` |
| Prompt/configuration weakness taxonomy | `2d5fb227e0f3599ac4eaf7884abd83b125d00ba0a2d02a01e4306a0e3c5182c2` |
| Dedicated R6 gate | `7aab0dcaf116a105c5ffc9e88826f98aec400b9144124f598b2d9dbcbd9ccd3e` |

The quality gate includes formatting, dependency-direction policy, production-file size policy, clippy with warnings denied, the full workspace test suite, and Rust documentation generation. The security gate passed dependency audit, sandbox capability checks, workspace policy, isolation, SQL policy, worker failure/isolation, untrusted-knowledge, adversarial protocol, run-verification, and deployment-conformance checks. Its final secret scan initially reported `incomplete` solely because the dirty Git index still selected the two deleted pre-R6 experiment-prototype files; after staging the exact final tree, the repeated scan was `clean` across 646 artifacts with no findings or incomplete reasons. The public end-to-end gate completed all 108 benchmark cells and verified its report, topology, diagnosis, run, secret-scan, and R2 compatibility artifacts by SHA-256.

The non-publishing package dry-run and remote workflow evidence remain pending at this point. Both are required before changing any R6 milestone from `implemented` to `complete`.

## Known limitations

- Validation is specific to exact declared controls and is not universally transferable.
- Leakage detection reduces known-answer risk but cannot prove absence of semantic memorization learned outside HuntEval.
- Final hidden-test governance relies on trusted local operators.
- Human decisions and adoption records are content-addressed assertions, not cryptographic signatures.
- Initial suggestions are deterministic rules and templates; no provider-driven generation is implemented.
- HuntEval records external adoption but never edits the active deployment.
- Autonomous prompt adoption, production SIEM scored execution, unrestricted network access, distributed orchestration, Kubernetes, and a web dashboard remain out of scope.
