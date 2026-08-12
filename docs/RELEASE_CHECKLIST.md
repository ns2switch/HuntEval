# Release-candidate checklist

HuntEval release candidates are verification exercises. They do not publish a production release.

## Before tagging

- Confirm the worktree is clean and the candidate commit is on protected `main`.
- Confirm every required CI job, including the blocking Trivy source scan, adversarial protocol, benchmark science, evidence-backed diagnosis, controlled improvement, knowledge and extensions, framework connectors, upstream framework conformance, commercial connector replay, R8 compatibility, and R8 supply chain, passed for the exact revision.
- Confirm the GitHub settings attestation is current.
- Review schema, diagnostic-taxonomy, and protocol compatibility, security impact, known limitations, and accepted ADR changes.
- Confirm no credentials, evaluator-only artifacts, partial runs, or unrestricted environment diagnostics are present in the package inputs.

## Dry run

Run locally with a new absolute output directory:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/e2e.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/r8-compatibility.sh
./scripts/ci/r8-supply-chain.sh
./scripts/ci/r8-release.sh /tmp/hunteval-rc
(cd /tmp/hunteval-rc && sha256sum -c SHA256SUMS)
```

An authorized release maintainer may create a unique annotated tag matching `v*-rc.*`. Pushing that tag starts `Release candidate dry run`, whose read-only matrix builds Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64 on native runners and uploads separate seven-day artifacts. Each job must install, smoke-test, hash, sign, verify, and pass the blocking Trivy candidate scan. The Trivy result is uploaded separately and must be preserved with the candidate evidence. The workflow does not create a GitHub Release, move an existing tag, publish a package, or claim scored execution outside Linux.

## Evidence and rollback

Record the revision, tag, workflow URL, native runner and target identity, platform-matrix hash, archive and native-evidence hashes and signatures for all four targets, Trivy version and JSON result hashes, stable and fuzz toolchains, Python support version, sandbox capability report, execution-policy hash, sandbox and resource-launcher hashes, protocol compatibility index and fuzz corpus hashes, conformance result hash, benchmark manifest hash, dataset hashes, deployment hashes, scoring-profile hash, runner and worker hashes, schemas 0.8, 0.9, and 1.0, the R8 interface-inventory and freeze-manifest hashes, improvement-policy, taxonomy, registry, diff, equivalence, validation, lifecycle, review/adoption, analytical corpus/index/query/audit hashes, extension manifest/policy/resolution/conformance hashes, SDK compatibility and wheel hashes, report and bundle hashes, normalized result digest, run-verification and secret-scan result hashes, checksum inventories, known limitations, and ADR status changes. If any value or gate differs, discard the candidate, fix through a reviewed commit, and use a new tag. Never overwrite or delete an existing candidate tag as a correction mechanism.
