# GitHub settings attestation

This record is completed only by an authorized administrator after inspecting the live repository settings. Committed code cannot certify these controls by itself.

- Repository: `ns2switch/HuntEval`
- Verified revision: `b412953a08f3e2e26dff82c1aa0a729515496564`
- Administrator: `ns2switch`
- Verification date (UTC): `2026-08-09`
- Evidence link or internal record: [protected release-candidate run 31329216944](https://github.com/ns2switch/HuntEval/actions/runs/31329216944)

## Checklist

- [x] `main` requires pull requests and at least one approval.
- [x] CODEOWNER review and stale-review dismissal are required.
- [x] All nine canonical CI jobs are required and administrator enforcement is enabled.
- [x] Force pushes and branch deletion are prohibited.
- [x] Active `v*` rulesets restrict creation and prohibit update or deletion without an immutability bypass.
- [x] Ordinary workflows have read-only tokens and no deployment secrets.
- [x] Artifact retention is bounded to the documented values.
- [x] Runner trust and fork-secret policies match `docs/GITHUB_OPERATIONS.md`.
- [x] Private vulnerability reporting and dependency security updates are enabled.
- [x] Rollback and release permissions have been reviewed by a maintainer.

Verifier command result: `GitHub branch and tag settings satisfy the committed R2 policy`

## Evidence

- `main` is protected for administrators and requires an up-to-date branch, one approval, CODEOWNER review, stale-review dismissal, conversation resolution, and the `Policy`, `Quality`, `Tests`, `Security`, `Adversarial protocol`, `End-to-end`, `Documentation`, `Benchmark science`, and `Package` checks. Force pushes and deletion are disabled.
- [Ruleset 20609834](https://github.com/ns2switch/HuntEval/rules/20609834) restricts creation of `v*` tags to the `ns2switch` release maintainer. [Ruleset 20609835](https://github.com/ns2switch/HuntEval/rules/20609835) prohibits update and deletion of `v*` tags and has no bypass actors.
- GitHub Actions defaults to read-only repository permission and cannot approve pull requests. Committed workflows use `ubuntu-22.04`, full action commit pins, no deployment-secret reference, three-day ordinary retention, and seven-day release-candidate retention.
- Private vulnerability reporting, vulnerability alerts, and Dependabot security updates are enabled.
- [CI run 31322660682](https://github.com/ns2switch/HuntEval/actions/runs/31322660682) passed all nine required jobs on the verified revision.
- Annotated tag `v0.4.0-rc.1` resolves to the verified revision. Its creation recorded the authorized creation-rule bypass; the separate immutability ruleset cannot be bypassed.
- The protected [release-candidate run 31329216944](https://github.com/ns2switch/HuntEval/actions/runs/31329216944) passed in 19 minutes 48 seconds without publishing a production release.
- The downloaded package `hunteval-rc-b412953a08f3-x86_64-unknown-linux-gnu.tar.gz` passed `sha256sum -c SHA256SUMS`. Its SHA-256 digest is `a6f0fe31efa8735fb42b74d2c407dc0247302b30a618eff0ca8d3e2f4b9da0e4`; `SHA256SUMS` has digest `4eda7cc51efcdebfca0b6e3ee8acd68767d3cd773d87a8a73c996f03a2e2c807`; and `secret-scan.json` has digest `fb49f11544fcf632a84d0e0f5d5ac3d75fdb4f691ac1bc756db0340a80935ee5`. The secret scan reported `clean` for 49 artifacts with no findings or incomplete reasons.

R2-18 host-side acceptance and the R2.4 exit gate are complete. This attestation records external governance state and does not alter historical implementation evidence.

## Solo-maintainer governance amendment

- Policy revision: `d9fe4a6b2f05fc3c42faa29570185efaa4c3861b`
- Administrator: `ns2switch`
- Verification date (UTC): `2026-08-10`
- Evidence: [policy validation run 31360218897](https://github.com/ns2switch/HuntEval/actions/runs/31360218897)

The GitHub collaborators API confirmed that `ns2switch` is the repository's sole collaborator and administrator. Requiring an approval from a different account therefore made the pull-request gate impossible to satisfy without adding an otherwise unauthorized reviewer.

The live `main` protection now retains mandatory pull requests while requiring zero approving reviews. CODEOWNER review and stale-review dismissal are not required in this single-maintainer configuration. The strict up-to-date branch requirement, all ten canonical CI checks, administrator enforcement, conversation resolution, force-push prohibition, branch-deletion prohibition, and both protected release-tag rulesets remain enforced.

The committed live-settings verifier passed against the detailed branch-protection response and rulesets `20609834` and `20609835`. Pull request [#1](https://github.com/ns2switch/HuntEval/pull/1) subsequently reported `CLEAN` and `MERGEABLE` with all ten required checks successful.

If a second trusted maintainer is granted repository access, the repository must restore at least one required approval, required CODEOWNER review, and stale-review dismissal as documented in `docs/GITHUB_OPERATIONS.md`. This amendment changes only the live governance configuration needed to support a sole maintainer; it does not modify or supersede the historical R2 evidence above.
