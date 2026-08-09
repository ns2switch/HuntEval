# Release-candidate checklist

HuntEval release candidates are verification exercises. They do not publish a production release.

## Before tagging

- Confirm the worktree is clean and the candidate commit is on protected `main`.
- Confirm all ten required CI jobs, including adversarial protocol, benchmark science, and evidence-backed diagnosis, passed for the exact revision.
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
./scripts/ci/e2e.sh
./scripts/ci/release-candidate.sh /tmp/hunteval-rc
(cd /tmp/hunteval-rc && sha256sum -c SHA256SUMS)
```

An authorized release maintainer may create a unique annotated tag matching `v*-rc.*`. Pushing that tag starts `Release candidate dry run`, which has read-only contents permission and only uploads a seven-day artifact. It does not create a GitHub Release, move an existing tag, or publish a package.

## Evidence and rollback

Record the revision, tag, workflow URL, stable and fuzz toolchains, sandbox capability report, execution-policy hash, sandbox and resource-launcher hashes, protocol compatibility index and fuzz corpus hashes, conformance result hash, benchmark manifest hash, dataset hashes, deployment hashes, scoring-profile hash, runner and worker hashes, normalized result digest, run-verification and secret-scan result hashes, `SHA256SUMS`, known limitations, and ADR status changes. If any value or gate differs, discard the candidate, fix through a reviewed commit, and use a new tag. Never overwrite or delete an existing candidate tag as a correction mechanism.
