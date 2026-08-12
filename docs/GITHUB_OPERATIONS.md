# GitHub operations

This document defines the repository controls required for HuntEval. Committed workflows implement the build and verification mechanics; an authorized repository administrator must configure and attest the host-side controls described here.

## Authoritative checks

The `CI` workflow uses read-only repository permissions, cancels superseded branch runs, pins every third-party action to a full commit, and exposes these required job names:

- `Policy`
- `Quality`
- `Tests`
- `Security`
- `Adversarial protocol`
- `End-to-end`
- `Documentation`
- `Benchmark science`
- `Evidence-backed diagnosis`
- `Controlled improvement`
- `Knowledge and extensions`
- `Framework connectors`
- `Upstream framework conformance`
- `Commercial connector replay`
- `R8 compatibility`
- `R8 supply chain`
- `Package`

Every job calls a repository-owned script under `scripts/ci/`. Cache keys bind the pinned runner image, operating system, architecture, Rust toolchain, compilation target, feature set, lockfile, and job purpose, so changing the runner image creates a new cache namespace. The uncached package job repeats deterministic contract and fixture tests in a clean target directory. Ordinary jobs receive no deployment or release secret.

The existing `Security` job additionally invokes Trivy v0.73.0 through `aquasecurity/trivy-action` v0.36.0 pinned to full commit `ed142fd0673e97e23eac54620cfb913e5ce36c25`. It scans source dependency manifests and repository configuration for `HIGH` and `CRITICAL` vulnerabilities and misconfigurations, including findings without an available fix. A finding, database/setup failure, timeout, malformed input, or scanner failure returns a non-zero status. The repository's bounded secret scanner remains authoritative for secret detection; Trivy is not configured to duplicate that scan or place possible secret excerpts in uploaded reports.

CI uploads only bounded logs, generated Rust documentation, normalized public benchmark reports, verification summaries, checksums, Trivy JSON results, and release-candidate binaries. Retention is three days for ordinary CI and seven days for each native release-candidate matrix artifact. The release-candidate matrix runs the same pinned Trivy release against each verified SPDX 2.3 candidate SBOM and uploads its report separately, so scanning covers the declared Cargo package inventory without mutating the signed candidate. Never upload an entire benchmark working directory, evaluator-only files, partial run directories, credentials, or runner diagnostics containing environment values.

Trivy's vulnerability database is current external security intelligence rather than a deterministic project input. Each JSON result is evidence for the exact workflow execution and database snapshot; it is not a compatibility, safety, publication, or reproducibility authority. Database and tool downloads are allowed only in CI/release infrastructure and never grant network access to an evaluated deployment.

## Required repository settings

Protect `main` with pull requests, conversation resolution, administrator enforcement, and all seventeen checks above. Require the branch to be current before merge. Disable force pushes and deletion. The repository currently has one administrator and no independent reviewer, so the pull-request rule requires zero approvals and does not require CODEOWNER self-approval. `CODEOWNERS` remains an ownership and routing record. If a second trusted maintainer is granted access, restore at least one approval, CODEOWNER review, and stale-review dismissal before that maintainer's first merge. Restrict emergency recovery to repository administrators and preserve an auditable pull request whenever GitHub is operational.

Create one active `v*` tag ruleset that restricts creation to explicit release maintainers and a separate active `v*` ruleset with no bypass actor that prohibits update and deletion after creation. Keeping immutability in a non-bypassable ruleset prevents a creation bypass from weakening existing tags. Production releases require a separately authorized manual decision; the committed release-candidate workflow has `contents: read` and cannot create a release.

The scored CI path pins GitHub-hosted `ubuntu-22.04` because HuntEval's mandatory Bubblewrap backend requires working unprivileged user namespaces; do not replace it with a moving runner label without executing the isolation and end-to-end gates first. The non-publishing package matrix additionally pins `macos-15-intel`, arm64 `macos-15`, and `windows-2022`. Those jobs prove only native build, package, installation, smoke, integrity, and rehearsal-signature behavior; they do not provide scored sandbox evidence. Use only GitHub-hosted runners or explicitly approved ephemeral self-hosted runners. Self-hosted runners must start clean, run a supported Actions Runner version, provide required host capabilities, have no production credentials, and be destroyed after the job. Fork pull requests must not receive repository secrets.

Enable private vulnerability reporting and Dependabot security updates. Dependency update pull requests must pass the complete pipeline and follow the same approval policy as other changes. Do not configure automatic merge for changes to workflows, schemas, security policy, isolation, SQL policy, fixtures, or release scripts.

## Verification

An administrator with read access to branch protection and repository rulesets runs:

```bash
GITHUB_TOKEN=<fine-grained-read-token> \
GITHUB_REPOSITORY=ns2switch/HuntEval \
./scripts/ci/verify-github-settings.sh
```

The token must be supplied through the process environment, never stored in the repository or command output. The verifier fails closed if the pull-request gate, zero-approval solo-maintainer policy, required checks, administrator enforcement, force-push prevention, deletion prevention, conversation resolution, or active protected-tag rulesets cannot be established.

Record the result in `docs/GITHUB_SETTINGS_ATTESTATION.md`. A missing name, date, revision, evidence link, or unchecked item means external governance is not certified.

## Incident rollback

Stop new merges and releases, preserve failed workflow logs within their retention window, and open a private security report when confidentiality or integrity may be affected. Revert the smallest offending commit through a pull request that passes the complete pipeline and receives independent review when another trusted maintainer is available. Do not rewrite `main` or an existing release tag. Invalidate compromised tokens, rebuild on a clean runner with empty caches, rerun all canonical scripts, and publish corrected checksums under a new release-candidate tag. Production rollback remains a maintainer decision and must reference the affected and replacement revisions.
