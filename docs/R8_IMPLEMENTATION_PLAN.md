# R8 implementation plan

## 1. Purpose and release position

This document turns the normative v0.8 roadmap scope into a reviewable implementation sequence. R8 is the release-candidate milestone: it freezes and audits the interfaces intended for v1.0, proves compatibility and migration behavior, produces reproducible verifiable packages, completes independent security and reproducibility review, and rehearses the official benchmark from a clean environment using only published instructions.

R8 does not redesign HuntEval or reopen completed R2 through R7 work. R2 through R7 remain complete with their recorded evidence. v0.7.1 and v0.7.2 remain separate preconditions and must satisfy their release gates, or be revised through roadmap governance, before R8-00 can freeze their interfaces. No R8 implementation or completion evidence exists yet.

R8 does not publish v1.0, enable production-scored SIEM execution, introduce unrestricted network access, add autonomous prompt adoption, create a hosted service, or collect private chain of thought. A release-candidate artifact is not a production release.

## 2. Delivery status

Status is evidence-based. `planned` makes no implementation claim. `in progress` means only part of the behavior or evidence exists. `implemented` requires the local behavior and focused tests. `complete` additionally requires canonical gates, review evidence, documentation, migration and rollback evidence, and passing protected GitHub workflows on the exact closure revision.

| Milestone | Status | Outcome |
|---|---|---|
| R8-00 | planned | precondition audit, release inventory, freeze policy, and accepted architecture decisions |
| R8-01 | planned | normative protocol, schema, CLI, artifact, SDK, connector, and platform compatibility matrix |
| R8-02 | planned | deterministic migration and explicit rejection behavior for retained artifacts |
| R8-03 | planned | reproducible build provenance, package inventories, SBOMs, dependency/license evidence, and checksums |
| R8-04 | planned | bounded signing and offline verification for release artifacts and reports |
| R8-05 | planned | installable CLI, worker, schema, taxonomy, and Python SDK candidates for every declared target |
| R8-06 | planned | independent security review, remediation, and residual-risk decision |
| R8-07 | planned | independent reproducibility review and clean-room rebuild evidence |
| R8-08 | planned | official benchmark-pack freeze, review, provenance, and clean-run acceptance |
| R8-09 | planned | installation, operations, governance, disclosure, benchmark-review, and release documentation |
| R8-10 | planned | protected release-candidate pipeline and end-to-end non-publishing rehearsal |
| R8-11 | planned | exact R8 closure evidence and v1.0 readiness decision |

No R8 milestone is complete.

## 3. Mandatory delivery rules

Every R8 pull request must:

- preserve Clean Architecture and keep release, signing, packaging, vendor, framework, and CI infrastructure outside the domain and evaluation cores;
- preserve ground-truth isolation, managed-tool authority, fail-closed sandboxing, immutable safety policy, and append-only provenance;
- freeze only interfaces backed by an explicit compatibility inventory, canonical fixtures, migration or rejection behavior, and retained readers;
- keep raw metric vectors authoritative and introduce no global score or implicit missing-value policy;
- treat source archives, dependencies, build metadata, SBOMs, signatures, packages, reports, connector output, and review documents as untrusted bounded input;
- use SHA-256 content identities and explicit signature identities; a signature never replaces content verification or policy authorization;
- require a clean source revision, locked dependencies, pinned toolchains, declared target, and empty output root for candidate construction;
- keep signing credentials outside the repository, ordinary CI, forks, logs, packages, and public run artifacts;
- prohibit publishing, tag replacement, mutation of an existing candidate, or secret-bearing CI execution without an explicit protected release action;
- retain historical compatibility fixtures and readers; rollback may disable a new writer but must not silently remove verification of existing artifacts;
- use typed errors and bounded parsing without `unwrap()`, `expect()`, first-party `unsafe`, or panic shortcuts in production paths;
- keep Rust and Python source modules cohesive, readable, and within repository size and lint policies;
- update documentation, ADRs, threat model, migration, rollback, limitations, and exact acceptance commands with the owning change;
- preserve every completed R2 through R7 evidence file and the R2.4 external-enforcement record exactly.

## 4. Release-candidate artifact set

R8-00 must freeze an explicit inventory. At minimum, one candidate contains:

- the HuntEval CLI and every worker required for the supported scored path;
- immutable schemas, taxonomies, protocol compatibility fixtures, scoring profiles, and benchmark manifests required by the declared support matrix;
- the Python SDK wheel and its compatibility inventory;
- the official versioned benchmark pack or an immutable manifest that resolves every included public artifact by digest;
- installation, operations, security, disclosure, governance, migration, rollback, and release documentation;
- normalized build provenance, package file inventories, SPDX and/or CycloneDX SBOMs selected by R8-00, dependency and license reports, secret-scan results, and SHA-256 checksums;
- detached signatures and verification material selected by the accepted signing ADR;
- a bounded release manifest linking revision, toolchains, targets, artifacts, hashes, signatures, reviews, known limitations, and release status.

The initial scored support target remains Linux with the required Bubblewrap capabilities. `x86_64-unknown-linux-gnu` may be the only supported R8 target if it is the only target that passes the complete sandbox and end-to-end suite. Any additional architecture is `experimental` or `unsupported` until it passes the same target-specific gates; cross-compilation alone is insufficient.

## 5. Architecture and dependency direction

```text
normative schemas/protocol/policies/fixtures
  -> compatibility and migration layer
       -> deterministic build and package assembly
            -> SBOM/checksum/provenance generation
                 -> isolated signing boundary
                      -> offline verifier
                           -> clean-room install and official benchmark rehearsal
                                -> bounded R8 evidence bundle

domain/evaluation/statistics/reporting
  <- no dependency on packaging, signing, CI providers, or release services
```

Release composition belongs to scripts and outer infrastructure. Verification may use reusable pure libraries, but signature providers, package registries, hosted CI, and operating-system installers never become domain dependencies. Ordinary benchmark execution must not require a network signing service.

## 6. Architecture decisions for R8-00

R8-00 must accept, revise, or reject the following proposed decisions before implementation:

- ADR-098: define a versioned stability and compatibility classification for every v1.0 candidate interface;
- ADR-099: make the release manifest the content-addressed root of one candidate artifact graph;
- ADR-100: generate normalized machine-readable SBOM and build-provenance artifacts without making either an authorization source;
- ADR-101: use detached signatures with offline-verifiable identity and immutable transparency evidence where applicable;
- ADR-102: declare target support only after native sandbox, conformance, package, install, and end-to-end evidence;
- ADR-103: require deterministic migration or typed rejection for every retained artifact family;
- ADR-104: freeze the official benchmark pack by manifest and exact content hashes;
- ADR-105: separate independent review evidence from implementation authorship and block closure on unresolved critical findings;
- ADR-106: keep candidate creation non-publishing and require a separate human-authorized v1.0 publication decision;
- ADR-107: preserve old readers and verifiers when rolling back a new writer, package, or frozen interface.

## 7. R8 milestone specifications

### R8-00 — Preconditions, inventory, and freeze decisions

1. **Objective and user-visible outcome:** establish the exact interfaces and artifacts eligible for the v1.0 compatibility freeze.
2. **Affected contracts and compatibility:** inventory every supported schema, protocol revision, CLI command, report, metric, scoring profile, topology, diagnosis, improvement, knowledge, extension, SDK, MCP, framework, and commercial connector contract.
3. **Security impact:** reject a freeze when ownership, bounds, trust boundary, parser behavior, or authority is undocumented.
4. **Ground-truth-isolation impact:** identify every private/public projection and prove no release artifact requires evaluator-private bytes.
5. **Positive tests:** every selected interface resolves canonical fixtures, owner, stability class, version range, and verification path.
6. **Negative tests:** unknown, experimental, incomplete, or precondition-blocked interfaces cannot be marked stable.
7. **Malformed-input tests:** malformed inventory entries, duplicate identities, conflicting ranges, unknown statuses, and missing owners fail closed.
8. **Deterministic/replay tests:** identical source and fixture inventories produce the same ordered freeze manifest and digest.
9. **Exact quality gates:** focused inventory tests, dependency-direction policy, documentation checks, `git diff --check`, and section 11 gates.
10. **Documentation and ADR changes:** accept the R8 ADR set, update the roadmap cross-reference, threat model, and compatibility policy.
11. **Migration behavior:** existing support statements remain authoritative until explicitly classified by the accepted inventory.
12. **Rollback behavior:** revert the freeze manifest without changing historical artifacts or completed evidence.
13. **Known limitations:** R8-00 is blocked until v0.7.1 and v0.7.2 satisfy their release gates or roadmap governance explicitly revises those prerequisites.

### R8-01 — Normative compatibility matrix

1. **Objective and user-visible outcome:** let users determine exactly which artifacts, clients, adapters, targets, and commands are supported together.
2. **Affected contracts and compatibility:** add a versioned machine-readable matrix plus a human-readable projection; preserve existing schemas and protocol fixtures byte-for-byte.
3. **Security impact:** compatibility cannot imply capability authorization, sandbox eligibility, trusted provenance, or safe network access.
4. **Ground-truth-isolation impact:** the matrix contains public identities only and never exposes private fixture membership or evaluator paths.
5. **Positive tests:** supported current and retained historical combinations validate through their canonical readers and conformance suites.
6. **Negative tests:** incompatible, removed, preview, unverifiable, or capability-mismatched combinations return typed outcomes.
7. **Malformed-input tests:** overlapping ranges, ambiguous precedence, unknown component kinds, invalid digests, and missing rejection reasons fail closed.
8. **Deterministic/replay tests:** matrix normalization and its documentation projection are byte-stable for identical inputs.
9. **Exact quality gates:** matrix schema tests, all retained compatibility fixtures, cross-language tests, and section 11 gates.
10. **Documentation and ADR changes:** publish support, deprecation, preview, removal, and end-of-support semantics.
11. **Migration behavior:** retain explicit readers/adapters for every supported historical version and name all unsupported transitions.
12. **Rollback behavior:** retain the prior matrix and readers; never broaden compatibility during rollback.
13. **Known limitations:** a compatibility result proves contract agreement, not deployment quality or operational security.

### R8-02 — Migration and rejection tooling

1. **Objective and user-visible outcome:** provide deterministic migrations where safe and precise rejection diagnostics everywhere else.
2. **Affected contracts and compatibility:** support only migration edges declared by R8-01; migrations create new artifacts and never overwrite source bytes.
3. **Security impact:** use no-follow confined reads, new output roots, byte/count/depth limits, atomic finalization, and post-write verification.
4. **Ground-truth-isolation impact:** public migrations reject private fields and cannot convert evaluator-private artifacts into public ones.
5. **Positive tests:** every declared edge migrates canonical and representative historical fixtures and passes the target verifier.
6. **Negative tests:** downgrade, ambiguous multi-hop, lossy undeclared, overwrite, traversal, symlink, and private/public conversion fail closed.
7. **Malformed-input tests:** truncated, unknown-field, invalid-version, duplicate-identity, stale-hash, and oversized inputs return stable reason codes.
8. **Deterministic/replay tests:** repeated migration from identical bytes yields identical normalized outputs and migration receipts.
9. **Exact quality gates:** focused migration CLI/library tests, retained readers, secret scan, compatibility gate, and section 11 gates.
10. **Documentation and ADR changes:** publish migration graph, rejection catalog, operator examples, and irreversible limitations.
11. **Migration behavior:** source artifacts remain immutable; receipts bind source and target hashes, tool identity, and exact edge.
12. **Rollback behavior:** disable a faulty migration writer while retaining its reader, fixtures, and receipt verifier.
13. **Known limitations:** R8 does not promise migration between semantically incompatible benchmark, scoring, or safety policies.

### R8-03 — Reproducible supply-chain evidence

1. **Objective and user-visible outcome:** produce auditable packages whose inputs, files, dependencies, licenses, build environment, and hashes are explicit.
2. **Affected contracts and compatibility:** add versioned release-manifest, package-inventory, SBOM, dependency-audit, license, and provenance artifacts.
3. **Security impact:** pin toolchains/actions, lock dependencies, isolate output roots, scan packages, and reject missing or unverified inputs.
4. **Ground-truth-isolation impact:** package allowlists exclude private datasets, hidden tests, evaluator outputs, credentials, run directories, and caches.
5. **Positive tests:** build twice from clean checkouts and verify inventories, normalized metadata, checksums, and selected reproducibility claims.
6. **Negative tests:** dirty tree, unlocked dependency, unknown license, unexpected file, secret canary, changed schema, and stale SBOM fail the build.
7. **Malformed-input tests:** malformed SBOM/provenance, duplicate paths, unsafe names, invalid hashes, and cyclic references fail verification.
8. **Deterministic/replay tests:** target artifacts are byte-identical where supported; unavoidable nondeterminism is normalized, isolated, and documented rather than hidden.
9. **Exact quality gates:** create `scripts/ci/r8-supply-chain.sh`; run package inspection, dependency/license audit, secret scan, clean rebuild, and section 11 gates.
10. **Documentation and ADR changes:** document tool versions, environment assumptions, SBOM format, provenance semantics, and verification commands.
11. **Migration behavior:** existing R7 candidate metadata remains readable and is adapted explicitly into the R8 release manifest.
12. **Rollback behavior:** discard the entire candidate and rebuild under a new identity; never patch an assembled archive.
13. **Known limitations:** source and binary reproducibility claims are target-specific and limited to the recorded build environment.

### R8-04 — Signing and offline verification

1. **Objective and user-visible outcome:** allow operators to verify candidate archives, SDK packages, reports, and the release manifest without trusting filenames or transport.
2. **Affected contracts and compatibility:** define signature inventory, signer identity/policy, verification result, timestamp/transparency material, and failure reasons.
3. **Security impact:** isolate signing after checksum and secret verification; ordinary CI and evaluated deployments receive no signing authority.
4. **Ground-truth-isolation impact:** only already-approved public candidate artifacts enter the signing boundary.
5. **Positive tests:** verify every candidate artifact and signed report from a fresh verifier environment using pinned trust policy.
6. **Negative tests:** changed bytes, substituted identity, wrong repository/ref/workflow, expired or revoked policy, missing bundle, and duplicate signature fail closed.
7. **Malformed-input tests:** truncated, oversized, unknown-algorithm, invalid-certificate, and adversarial signature bundles return typed safe errors.
8. **Deterministic/replay tests:** content hashes remain stable; signature nondeterminism is explicit and does not alter artifact identity.
9. **Exact quality gates:** signature fixture suite, offline verification, secret isolation, hostile-bundle tests, and section 11 gates.
10. **Documentation and ADR changes:** document trust roots, signer policy, verification, rotation, revocation, compromise response, and offline limitations.
11. **Migration behavior:** unsigned historical artifacts remain verifiable by digest but cannot be relabeled as signed R8 candidates.
12. **Rollback behavior:** revoke or distrust the candidate identity through policy and issue a new immutable candidate; never replace a signature in place.
13. **Known limitations:** signing proves origin and integrity under the declared policy, not benchmark quality or absence of defects.

### R8-05 — Supported-target packaging and installation

1. **Objective and user-visible outcome:** install and remove the complete candidate predictably on every declared supported target.
2. **Affected contracts and compatibility:** bind binaries, workers, schemas, taxonomies, SDK, licenses, and runtime capability requirements to the release manifest.
3. **Security impact:** use fixed file permissions, no privileged post-install scripts, no inherited secrets, safe paths, and fail-closed sandbox checks.
4. **Ground-truth-isolation impact:** installers contain no evaluator-private or hidden benchmark artifacts.
5. **Positive tests:** clean native target installs run `system check`, CLI smoke, worker handshake, schema validation, Python import, and offline fixture verification.
6. **Negative tests:** unsupported OS/architecture, missing Bubblewrap, changed worker, unsafe destination, existing-file collision, and insufficient isolation fail clearly.
7. **Malformed-input tests:** corrupt archives, duplicate paths, traversal entries, invalid permissions, symlinks, and missing manifest members are rejected.
8. **Deterministic/replay tests:** package inventory and install result are stable for the same target artifact.
9. **Exact quality gates:** native target matrix, archive extraction adversarial tests, install/uninstall smoke, package secret scan, and section 11 gates.
10. **Documentation and ADR changes:** publish supported targets, prerequisites, installation, verification, upgrade, uninstall, and limitations.
11. **Migration behavior:** upgrades preserve user-owned run artifacts and require explicit migration of authored manifests where needed.
12. **Rollback behavior:** reinstall the last trusted immutable package; never downgrade or rewrite stored artifacts silently.
13. **Known limitations:** scored execution is supported only where all Linux isolation capabilities pass; packaging does not imply scored support on other systems.

### R8-06 — Independent security review

1. **Objective and user-visible outcome:** obtain a review independent of the reviewed implementation and resolve security findings before freeze.
2. **Affected contracts and compatibility:** review all trust boundaries, parsers, protocols, migration paths, packages, signing, connectors, sandboxing, and public/private projections.
3. **Security impact:** threat-model mapping, manual review, dependency analysis, adversarial tests, and remediation evidence are mandatory.
4. **Ground-truth-isolation impact:** explicitly review episode projection, hidden tests, diagnostics, knowledge scopes, prompt experiments, connectors, packages, logs, and CI artifacts.
5. **Positive tests:** reviewers reproduce the declared safe paths and verify security controls from documented commands.
6. **Negative tests:** rerun and extend traversal, SSRF, injection, secret, process, protocol, decompression, migration, signature, and provenance attacks.
7. **Malformed-input tests:** review the retained malformed corpus and coverage map for every public parser and protocol boundary.
8. **Deterministic/replay tests:** replay security fixtures against the exact candidate revision with recorded tool and corpus hashes.
9. **Exact quality gates:** `security.sh`, adversarial protocol, connector adversarial suites, supply-chain gate, independent report verification, and section 11 gates.
10. **Documentation and ADR changes:** publish a bounded non-sensitive review summary, remediation mapping, accepted residual risks, and disclosure route.
11. **Migration behavior:** security fixes preserve compatibility unless a documented fail-closed break is required and approved before freeze.
12. **Rollback behavior:** stop R8, revoke affected candidates, preserve evidence, and revert through a reviewed change.
13. **Known limitations:** the sole maintainer cannot self-certify review independence; R8 closure requires a separately identified reviewer or organization.

### R8-07 — Independent reproducibility review

1. **Objective and user-visible outcome:** prove that a reviewer can rebuild, install, run, and verify HuntEval from the documented source revision.
2. **Affected contracts and compatibility:** consume the release manifest, build provenance, target matrix, benchmark pack, and verifier contracts without privileged internal knowledge.
3. **Security impact:** use a clean ephemeral environment, empty caches where declared, no production credentials, and bounded public outputs.
4. **Ground-truth-isolation impact:** reviewers receive only the authorized evaluator inputs required for the official benchmark and publish no private bytes.
5. **Positive tests:** a clean checkout reproduces packages, installs them, runs the benchmark, generates reports, and verifies signatures and checksums.
6. **Negative tests:** stale caches, changed dependencies, missing tools, changed datasets, altered reports, and unexpected network access invalidate the review.
7. **Malformed-input tests:** corrupted release manifests, provenance, SBOMs, packages, benchmark artifacts, and reports fail offline verification.
8. **Deterministic/replay tests:** compare exact and semantic identities under the policy defined by R8-03 and explain every allowed difference.
9. **Exact quality gates:** clean-room script, zero-cache rebuild, official benchmark, report verification, secret scan, and section 11 gates.
10. **Documentation and ADR changes:** record environment, commands, durations, resources, hashes, deviations, and reviewer identity.
11. **Migration behavior:** the review includes at least one retained historical artifact for every declared migration family.
12. **Rollback behavior:** discard the candidate when reproduction fails; correct source or documentation and create a new immutable candidate.
13. **Known limitations:** reproducibility evidence applies only to recorded targets, toolchains, inputs, and environment classes.

### R8-08 — Official benchmark-pack freeze and review

1. **Objective and user-visible outcome:** ship one versioned official cloud benchmark pack suitable for the v1.0 candidate evaluation semantics.
2. **Affected contracts and compatibility:** freeze episodes, public/private manifests, datasets, deployments, topology variants, seeds, budgets, tools, scoring profiles, and expected verification policy by digest.
3. **Security impact:** review dataset licensing, secrets, personal data, untrusted telemetry, parser bounds, tool policy, and package contents.
4. **Ground-truth-isolation impact:** public observations and deployment packages remain byte-separated from private labels and hidden-test membership.
5. **Positive tests:** every cell validates and runs from clean installation instructions; reports expose raw metric vectors and declared limitations.
6. **Negative tests:** leakage, changed hash, duplicate episode, missing control, unsupported metric, invalid license, and undeclared topology variable fail closure.
7. **Malformed-input tests:** retained schema, dataset, manifest, scoring, topology, and report malformed corpora remain active.
8. **Deterministic/replay tests:** two clean runs reproduce all deterministic identities and statistically compare nondeterministic cells under declared policy.
9. **Exact quality gates:** benchmark validation, R4 science/topology, end-to-end matrix, replay, report verification, secret scan, and section 11 gates.
10. **Documentation and ADR changes:** publish benchmark card, dataset cards, intended use, exclusions, review process, limitations, and version policy.
11. **Migration behavior:** prior benchmark packs retain independent identities and remain comparable only when their control and metric contracts permit it.
12. **Rollback behavior:** withdraw the candidate pack, retain its immutable evidence, and issue a new version; never replace an episode under the same digest/version.
13. **Known limitations:** the official pack does not establish universal agent, model, framework, topology, or provider performance.

### R8-09 — Operator, governance, and disclosure documentation

1. **Objective and user-visible outcome:** enable a new operator or contributor to install, evaluate, verify, report, disclose, and recover without undocumented maintainer knowledge.
2. **Affected contracts and compatibility:** document the exact frozen interfaces and generated support matrix; documentation examples are tested inputs.
3. **Security impact:** include secure defaults, trust model, credential boundaries, secret handling, sandbox prerequisites, incident response, and vulnerability disclosure.
4. **Ground-truth-isolation impact:** document physical and logical separation, allowed publication, review, retention, and deletion procedures.
5. **Positive tests:** clean-room reviewers complete installation, first run, benchmark, verification, migration, and rollback from documentation only.
6. **Negative tests:** unsafe configuration, missing isolation, unsupported connector mode, invalid signature, and private artifact publication paths are explicit and tested.
7. **Malformed-input tests:** documented troubleshooting maps typed malformed and compatibility failures without exposing untrusted payloads or secrets.
8. **Deterministic/replay tests:** executable snippets and documented expected artifacts run in CI from pinned fixtures.
9. **Exact quality gates:** documentation link/style checks, executable examples, CLI help snapshots, disclosure checks, and section 11 gates.
10. **Documentation and ADR changes:** update README, specification, contracts, threat model, operations, governance, security, contributing, benchmark review, and release checklist.
11. **Migration behavior:** publish supported upgrade paths, rejection behavior, artifact retention, and deprecation timelines.
12. **Rollback behavior:** document package, configuration, candidate, benchmark, signing, and governance rollback without history rewriting.
13. **Known limitations:** documentation cannot convert preview or unavailable behavior into supported functionality.

### R8-10 — Protected release-candidate pipeline and rehearsal

1. **Objective and user-visible outcome:** create one checksummed, signed, independently verifiable candidate from an immutable tag without publishing a release.
2. **Affected contracts and compatibility:** the workflow consumes only R8 release, compatibility, provenance, package, signature, review, and benchmark contracts.
3. **Security impact:** use minimal permissions, pinned actions, protected environments for signing, no fork secrets, isolated jobs, bounded artifacts, and immutable tags.
4. **Ground-truth-isolation impact:** uploaded artifacts are allowlisted and secret/private scanned; entire run or evaluator directories are prohibited.
5. **Positive tests:** protected tag or approved dispatch builds, signs, installs, runs, verifies, uploads bounded evidence, and records exact workflow identity.
6. **Negative tests:** unprotected ref, missing approval, stale base, dirty source, failed prerequisite, secret canary, duplicate tag, and artifact mismatch prevent completion.
7. **Malformed-input tests:** hostile archives, manifests, SBOMs, signatures, reports, and workflow inputs fail before publication or upload.
8. **Deterministic/replay tests:** repeat the dry run under a new immutable candidate identity and compare declared reproducible outputs.
9. **Exact quality gates:** create `scripts/ci/r8-release.sh`; require every section 11 gate plus package, signature, clean-room, benchmark, and settings verification.
10. **Documentation and ADR changes:** update GitHub operations, settings attestation procedure, release checklist, evidence template, and incident rollback.
11. **Migration behavior:** the pipeline validates retained readers and migration fixtures before candidate construction.
12. **Rollback behavior:** stop promotion and mark the candidate rejected; never move, overwrite, or delete an existing candidate tag as correction.
13. **Known limitations:** R8 creates no GitHub Release, registry publication, production deployment, or v1.0 tag automatically.

### R8-11 — R8 closure evidence and v1.0 readiness

1. **Objective and user-visible outcome:** record exact evidence that the release candidate satisfies every R8 gate and identify any remaining v1.0 decision.
2. **Affected contracts and compatibility:** freeze the release manifest, compatibility matrix, migration inventory, package target matrix, review reports, benchmark pack, and evidence index.
3. **Security impact:** require no unresolved P0/Critical defect, documented disposition of lower-severity findings, current dependency evidence, and verified signing policy.
4. **Ground-truth-isolation impact:** record scans and negative tests proving that packages, reports, signatures, logs, and CI artifacts contain no private benchmark material.
5. **Positive tests:** all local, clean-room, independent-review, protected workflow, install, benchmark, replay, and verification evidence resolves by hash.
6. **Negative tests:** missing review, stale evidence, failed check, open critical defect, unsupported target claim, or unverified artifact blocks closure.
7. **Malformed-input tests:** record corpus identities and expected typed outcomes for every frozen parser and protocol boundary.
8. **Deterministic/replay tests:** record build comparison, migration replay, protocol replay, official benchmark replay, and offline verification results.
9. **Exact quality gates:** all section 11 commands and required protected checks pass on the exact evidence revision and candidate tag.
10. **Documentation and ADR changes:** add `R8_COMPLETION_EVIDENCE.md`, update roadmap/status/checklist, and record ADR dispositions without changing prior evidence.
11. **Migration behavior:** publish the final v1.0-candidate compatibility and deprecation matrix.
12. **Rollback behavior:** retain R8 readers and evidence, reject the candidate, fix through a new reviewed revision, and issue a new immutable candidate.
13. **Known limitations:** R8 closure authorizes v1.0 release preparation only; it is not v1.0 publication and does not remove post-v1.0 deferrals.

## 8. Dependency graph and delivery waves

```text
v0.7.1 complete + v0.7.2 complete
  -> R8-00 preconditions/inventory/ADRs
       -> R8-01 compatibility matrix
            -> R8-02 migration/rejection tooling

R8-00
  -> R8-03 supply-chain evidence
       -> R8-04 signing/verification
            -> R8-05 target packaging/install

R8-01 + R8-02 + R8-03 + R8-05
  -> R8-06 independent security review
  -> R8-07 independent reproducibility review
  -> R8-08 official benchmark-pack freeze

R8-06 + R8-07 + R8-08
  -> R8-09 complete operator/governance documentation
       -> R8-10 protected candidate rehearsal
            -> R8-11 R8 closure and v1.0 readiness
```

Delivery waves:

1. **Wave A — Freeze boundaries:** R8-00 through R8-02.
2. **Wave B — Build verifiable artifacts:** R8-03 through R8-05.
3. **Wave C — Independent review and official benchmark:** R8-06 through R8-08.
4. **Wave D — Operate, rehearse, and close:** R8-09 through R8-11.

No later wave can make an earlier failing gate advisory. Security, leakage, compatibility, and reproducibility failures preempt release work.

## 9. Reviewable pull-request sequence

| PR | Milestones | Required result before merge |
|---|---|---|
| PR-01 | R8-00 | accepted inventory/ADR decisions and precondition evidence |
| PR-02 | R8-01 | versioned compatibility matrix and retained fixture coverage |
| PR-03 | R8-02 | non-overwriting migration/rejection CLI and receipts |
| PR-04 | R8-03 | release manifest, SBOM, provenance, checksum, and clean-build gate |
| PR-05 | R8-04 | isolated signing policy and hostile/offline verification suite |
| PR-06 | R8-05 | supported-target packages and native clean-install evidence |
| PR-07 | R8-06 | independent security report, fixes, and residual-risk disposition |
| PR-08 | R8-07 | independent clean-room reproduction evidence |
| PR-09 | R8-08 | frozen official benchmark pack and review artifacts |
| PR-10 | R8-09 | complete tested operator/governance documentation |
| PR-11 | R8-10 | protected non-publishing candidate workflow and rehearsal |
| PR-12 | R8-11 | exact closure evidence and roadmap status update |

PRs may be split further when code or review size requires it. Milestones must not be combined to bypass an evidence or approval boundary. Each PR ends with a descriptive commit and exact local/remote evidence; `planned` is not changed directly to `complete` without the required intermediate implementation evidence.

## 10. Test matrix and failure semantics

R8 adds coverage for:

- compatibility inventory normalization, overlap, deprecation, rejection, and cross-language agreement;
- every supported migration edge plus downgrade, overwrite, traversal, symlink, stale-hash, and private/public failures;
- archive path safety, permissions, package allowlists, SBOM/provenance parsing, dependency/license policy, and secret canaries;
- signer identity, repository/ref/workflow binding, tampering, substitution, revocation, expiration, offline verification, and hostile bundles;
- native target capability, install, upgrade, uninstall, worker handshake, CLI, SDK, and full sandbox/e2e behavior;
- clean-room, empty-cache, changed-toolchain, changed-dependency, unexpected-network, and nondeterminism detection;
- official benchmark leakage, control equivalence, topology variables, scoring profiles, metric availability, replay, and report verification;
- release workflow permission, environment, tag immutability, stale-base, failure propagation, upload allowlist, and rollback behavior.

Unsupported or unverifiable behavior remains unavailable. Missing evidence is never converted into a passing result. A signed artifact with a failed content, compatibility, security, or reproducibility check remains invalid.

## 11. Canonical acceptance commands

R8 implementation must add focused scripts with these stable entry points:

```bash
./scripts/ci/r8-compatibility.sh
./scripts/ci/r8-supply-chain.sh
./scripts/ci/r8-release.sh
```

Every R8 closure revision must also pass:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
./scripts/ci/quality.sh all
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/v072-commercial-connectors.sh
./scripts/ci/e2e.sh
git diff --check
```

The final candidate rehearsal additionally runs `release-candidate.sh` or its reviewed R8 replacement from a clean protected revision into a new absolute output directory, verifies all checksums and signatures offline, installs the resulting packages in a fresh environment, runs the official benchmark pack, and verifies the complete bounded evidence bundle.

## 12. Release closure criteria

R8 is complete only when:

- v0.7.1 and v0.7.2 prerequisites are complete or their unmet requirements were revised before R8-00 through roadmap governance;
- every intended v1.0 interface has an explicit stability class, owner, version range, compatibility fixtures, and migration or rejection behavior;
- there is no unresolved P0/Critical defect, undocumented compatibility break, failing or skipped required gate, unsupported target claim, or unverifiable release artifact;
- package builds, SBOMs, dependency/license evidence, provenance, checksums, signatures, and offline verification pass for every supported target;
- independent security and reproducibility reviews are complete and their evidence is bound to the exact candidate revision;
- the official benchmark pack runs from a clean environment using only published instructions and produces verified signed reports;
- documentation covers installation, operations, governance, disclosure, benchmark review, migration, rollback, and release procedures;
- protected GitHub checks and tag/ruleset verification pass on the exact closure revision;
- one immutable non-publishing release-candidate rehearsal completes from source checkout to signed verified reports;
- `R8_COMPLETION_EVIDENCE.md` records exact revisions, workflow URLs, artifact identities, review identities, known limitations, and rollback instructions.

R8 completion does not publish v1.0 and does not enable production-scored SIEM execution or any other post-v1.0 deferred capability.

## 13. Required future evidence and documents

Implementation will require these additions or updates:

- accepted ADR-098 through ADR-107 dispositions;
- compatibility, migration, release-manifest, provenance, package-inventory, signature, verification, and R8 evidence contracts where R8-00 determines they are normative;
- canonical positive and malformed fixtures for each new contract;
- `R8_COMPLETION_EVIDENCE.md` only after every closure criterion passes;
- threat-model sections for supply-chain inputs, signing identities, release workflows, migration tooling, installers, and benchmark publication;
- operator installation, migration, rollback, disclosure, benchmark-review, and release procedures;
- GitHub settings attestation for every new required check, protected signing environment, and immutable candidate-tag policy;
- independent security and reproducibility review records bound to the exact candidate revision.

None of these future artifacts is implementation evidence until its owning milestone passes locally and remotely.
