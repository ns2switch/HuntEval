# R8 candidate operations

## Scope

R8 creates immutable, non-publishing release-candidate evidence. It does not publish v1.0, authorize production SIEM execution, enable unrestricted network access, or convert preview connectors into supported interfaces.

## Build and verify

Use a clean protected revision and a new absolute output directory:

```bash
./scripts/ci/r8-compatibility.sh
./scripts/ci/r8-supply-chain.sh
./scripts/ci/r8-release.sh /tmp/hunteval-r8-candidate
```

The Linux rehearsal builds the Rust binaries and Python wheel, creates deterministic package and dependency evidence, generates an SPDX 2.3 SBOM with canonical Cargo purl references, scans public package contents, computes checksums, installs the archive into a new confined directory, runs a CLI smoke check, signs the release evidence with an ephemeral rehearsal identity, and verifies the detached signature offline. The protected release-candidate workflow performs the equivalent native package, install, smoke, evidence-signing, and verification path for Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64. It then runs pinned Trivy v0.73.0 against each verified candidate SBOM and fails on `HIGH` or `CRITICAL` vulnerabilities, including vulnerabilities without a fix. Trivy reports are separate workflow evidence and cannot mutate the signed candidate. The rehearsal identities are not production release identities.

Build provenance records network use explicitly. Native clean-runner candidates declare network use because Cargo and the isolated Python build backend may retrieve locked or pinned dependencies. Supply-chain fixtures that run from pre-provisioned inputs declare network isolation. The evidence builder has no implicit default: omitting both declarations fails closed.

The native package scan raises the per-file scanner bound from its 128 MiB default to the explicit 512 MiB package-member ceiling so that statically linked Windows binaries remain scannable. The CLI rejects zero, unbounded, or larger values, and any incomplete scan still rejects the candidate.

Verify downloaded evidence before using any binary:

```bash
(cd /tmp/hunteval-r8-candidate && sha256sum -c SHA256SUMS)
python3 scripts/r8_supply_chain.py verify \
  --root /tmp/hunteval-r8-candidate/evidence
python3 scripts/r8_sign.py verify \
  --artifact /tmp/hunteval-r8-candidate/evidence/release-manifest.json \
  --signature-root /tmp/hunteval-r8-candidate/signature \
  --repository ns2switch/HuntEval \
  --ref refs/heads/main \
  --workflow r8-release-rehearsal \
  --at-epoch <candidate-commit-epoch>
```

Verification must use the expected repository, ref, workflow, signer identity, validity interval, key fingerprint, and revocation policy. A signature proves origin and integrity under that policy only; it does not prove security, reproducibility, compatibility, or benchmark quality.

## Install and remove

Install only into a new absolute destination:

```bash
python3 scripts/r8_install.py install \
  --archive /tmp/hunteval-r8-candidate/hunteval-rc-<revision>-x86_64-unknown-linux-gnu.tar.gz \
  --destination /opt/hunteval-<revision>
python3 scripts/r8_install.py verify --root /opt/hunteval-<revision>
/opt/hunteval-<revision>/bin/hunteval system check --format json
```

The installer rejects traversal, duplicate members, symlinks, hard links, devices, FIFOs, oversized expansions, existing destinations, missing members, and unsupported layouts. It creates no privileged post-install script and changes no user-owned run directory.

Linux and macOS candidates use deterministic `.tar.gz` archives. Windows uses a deterministic `.zip` archive and `bin/hunteval.exe`; verification must pass `--target x86_64-pc-windows-msvc`. macOS and Windows packages are preview artifacts only: installation success does not authorize scored execution or create a sandbox capability.

Removal is an explicit operator action after preserving any user-owned artifacts outside the immutable installation root. Remove only the exact verified installation directory; never use an unresolved variable, glob, repository root, home directory, or filesystem root as the target.

## Compatibility and migration

`R8_COMPATIBILITY.md` is generated from the normative JSON matrix. Missing combinations are unavailable. Compatibility never grants capability or security authority.

Existing v0.3 benchmark manifests and scoring profiles are adapted in memory by their retained readers. Protocol 0.3 is read as-is. Source bytes are never overwritten. Migration receipts bind exact source, target, edge, and implementation hashes. Future major, downgrade, undeclared, ambiguous, lossy, and private/public transitions are rejected.

## Upgrade and rollback

Install a new candidate alongside the previous immutable installation, verify it, and then change the operator-owned invocation path explicitly. Stored runs and authored manifests are not rewritten automatically.

Rollback selects the last trusted immutable installation and prior compatibility matrix. Retain the rejected candidate, signatures, receipts, readers, fixtures, and review evidence. Never move or replace a tag, patch an archive, replace a signature, broaden compatibility, or silently downgrade stored artifacts.

## Incidents and disclosure

Stop promotion when a checksum, signature, package, migration, benchmark, isolation, review, or reproducibility check fails. Preserve bounded logs and artifact identities, revoke or distrust affected signing identities, and correct the source through a reviewed pull request under a new candidate identity. Follow `SECURITY.md` and use GitHub private vulnerability reporting for sensitive findings.

## Known limitations

The only scored support candidate is native `x86_64-unknown-linux-gnu` with the required Bubblewrap capabilities. Native macOS Intel, macOS Apple Silicon, and Windows x86_64 packages remain preview until their protected jobs pass, and they have no scored-execution sandbox claim. The framework/MCP pack and commercial connectors remain preview or unavailable until their separate release gates close. Candidate construction does not provide independent review and cannot publish a production release.
