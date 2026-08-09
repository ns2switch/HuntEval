# R4 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R4.1 through R4.4 into a reviewable implementation sequence. R4 improves benchmark validity, statistical policy, dataset contribution safety, and topology-aware comparative evaluation. It does not reopen completed R2 or R3 behavior, introduce a universal score, or move production SIEM execution into the pre-v1.0 scope.

R4 covers:

- broader deterministic episode coverage, including benign, multi-stage, cross-boundary, and ambiguous cases;
- versioned statistical policies for descriptive and inferential comparison claims;
- bounded contributor tooling for episode scaffolding, validation, documentation, and review bundles;
- normative topology artifacts, controlled topology experiments, topology-aware metrics, and auditable ablations.

R2 remains complete with its recorded R2.4 external-enforcement caveat. R3 remains complete with the evidence in `R3_COMPLETION_EVIDENCE.md`. Existing benchmark execution, scoring-profile semantics, constraint-first ranking, sandboxing, protocol, verification, redaction, and secret-scanning contracts remain authoritative. R5 diagnosis and R6 controlled improvement remain future work.

### Delivery status

Status values are evidence-based. `planned` makes no implementation claim. `implemented` means behavior and focused local tests exist but release evidence is incomplete. `complete` requires a dedicated commit, focused acceptance tests, all canonical gates, documentation evidence, and passing GitHub Actions on that revision.

| Milestone | Status | Outcome or dependency |
|---|---|---|
| R4-00 | implemented | schema 0.6 contracts, canonical examples, boundaries, and accepted ADR-053 through ADR-059 |
| R4-01 | implemented | optional public classification is validated and content-addressed without answer labels |
| R4-02 | implemented | one deterministic benign episode per provider and evaluator-only benign semantics |
| R4-03 | implemented | deterministic multi-stage and cross-boundary episodes for all three providers |
| R4-04 | implemented | nine independent approvals bind exact public, private, query, and review-policy bytes |
| R4-05 | implemented | versioned comparison class, minimum-pair policy, and claim-strength enforcement |
| R4-06 | implemented | paired effects, declared-confidence intervals, and multiplicity policy |
| R4-07 | implemented | bounded confidence and severity calibration with unavailable states |
| R4-08 | implemented | policy-bound normalized comparison output is available through the CLI and E2E gate |
| R4-09 | implemented | safe non-overwriting episode scaffold command and negative path tests |
| R4-10 | implemented | bounded validation and canonical provider, recovery, review, and leakage gates are integrated |
| R4-11 | implemented | ground-truth-free documentation and content-addressed reviewer inventory rendering |
| R4-12 | implemented | explicit normative deployment-topology artifact, loader, digest, and validation |
| R4-13 | implemented | three conformant reference topology artifacts share the 108-cell matrix |
| R4-14 | implemented | controlled experiment and fail-closed exact-change equivalence contracts |
| R4-15 | implemented | separate topology-aware observable dimensions with explicit unavailable values |
| R4-16 | implemented | the 108-cell E2E produces an auditable paired topology experiment and policy-bound reduction |
| R4-17 | implemented | deterministic JSON/static HTML topology reports are integrated into the CLI and E2E artifacts |
| R4-18 | implemented | local closure gates pass; dedicated revision and remote GitHub Actions evidence remain required |

The R4/v0.4 release name is independent from persisted schema versions. Existing schema 0.3, 0.4, and 0.5 artifacts remain immutable. R4-00 selects one additive schema version for new benchmark-science artifacts; no existing source artifact is rewritten to simulate compatibility.

## 2. Baseline audit

The repository already provides useful R4 foundations:

1. Nine deterministic synthetic cloud episodes cover three identity-focused categories for AWS, Azure, and Google Cloud.
2. Public manifests and private ground truth are loaded separately, content-addressed, path-validated, and protected by leakage tests.
3. Fixture regeneration, Parquet schema, package index, reference query, and exact digest tests already exist.
4. The benchmark matrix preserves exact deployment, episode, seed, configuration, scoring-profile, schema, policy, and binary identities.
5. Paired differences, deterministic bootstrap intervals, stability summaries, wins/ties/losses, raw metric vectors, and constraint-first ranking exist.
6. Scoring profiles retain explicit missing-value behavior; unavailable or unverifiable metrics need not become zero.
7. Single-agent, supervisor-worker, and supervisor-specialist reference peers execute through the same managed protocol and sandbox.
8. Observable coordination, duplicate-work, useful-communication, utilization, efficiency, resilience, and quality primitives already exist.

The audit also identifies the R4 gaps:

1. No scored episode is explicitly benign, so empty-ground-truth behavior is not exercised by the complete benchmark matrix.
2. Existing generated scenarios are short identity sequences and do not cover long multi-stage or cross-account, project, or tenant investigations.
3. Difficulty and capability tags have no normative versioned contract or leakage review rule.
4. Ground-truth and reference-query review is tested mechanically but has no independent, content-addressed review record.
5. Bootstrap output alone does not define minimum samples, effect-size semantics, comparison classes, multiple-comparison handling, or allowed claim language.
6. Confidence and finding severity have no calibration contract.
7. The fixture tool regenerates a fixed catalog but cannot safely scaffold or comprehensively validate a contributed package.
8. Contributor documentation and review bundles are not generated from validated metadata.
9. Protocol registration exposes a narrow architecture enum and agent list but not a normative, versioned deployment topology.
10. Benchmark definitions do not declare controlled topology variables or prove equivalence of all non-experimental variables.
11. Existing observational coordination metrics cannot establish marginal-agent or role contribution.
12. Reports do not yet separate topology overhead, utilization, resilience, and quality into a controlled experimental projection.

These gaps define the implementation order. R4 extends the existing runner, statistics, fixture, and reporting paths rather than creating parallel benchmark systems.

## 3. Mandatory delivery rules

Every R4 pull request must:

- preserve the domain crate's independence from DuckDB, filesystem adapters, CLI parsing, provider SDKs, LLM providers, and agent frameworks;
- keep private ground truth, reference answers, review-only notes, and hidden partition results outside deployment-visible artifacts and sandbox mounts;
- treat authored metadata, topology labels, role names, documentation text, and review notes as bounded untrusted input;
- preserve exact artifact bytes and hashes for episodes, topology, policies, experiments, schemas, binaries, and generated results;
- retain the authoritative raw metric vector and explicit applicability before any scoring or ranking projection;
- derive every optional aggregate only through an explicit versioned scoring profile;
- never impute unavailable, missing, or unverifiable values and never infer unsupported topology metrics;
- preserve constraint-first ranking where it applies;
- label descriptive, exploratory, validation, hidden-test, experimental, topology-dependent, and inconclusive results explicitly;
- reject causal contribution claims from observational traces alone;
- enforce paired control-variable equivalence before a controlled topology comparison is eligible;
- use stable Rust, typed errors, bounded collections, no first-party `unsafe`, and no panic shortcuts in production paths;
- keep production Rust files below 500 lines and split cohesive modules before 300 lines where practical;
- add positive, negative, malformed-input, deterministic/replay, leakage, compatibility, and resource-bound tests for every changed boundary;
- update contracts, schemas, threat model, ADRs, CLI documentation, migration behavior, rollback behavior, and known limitations with the behavior;
- keep all repository artifacts in English.

The canonical completion gates remain:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/e2e.sh
git diff --check
```

R4-04 introduces a deterministic dataset gate and R4-17 introduces a topology benchmark gate. Once added, both become mandatory locally and in GitHub Actions.

## 4. Architecture decisions to close in R4-00

R4-00 is documentation and contract work. It adds proposed ADR-053 through ADR-059 without modifying accepted ADR-001 through ADR-052.

### ADR-053 — Add immutable benchmark-science contracts

- New R4 artifacts use one additive schema version selected before Rust types are written.
- Schema 0.3, 0.4, and 0.5 examples remain byte-immutable compatibility fixtures.
- Readers use explicit adapters; writers emit only the current authored form.
- Unknown versions, fields, enum variants, relationships, and missing required controls fail closed.

### ADR-054 — Separate public episode classification from private review evidence

- Difficulty, capability, provider, investigation-shape, and benign-case tags may be public only when they cannot disclose answers.
- Ground truth, reference queries, expected paths, recovery details, reviewer notes, and hidden partition membership remain private.
- Independent review records bind exact public and private artifact hashes but expose only a public approval status and safe reason codes to contributor-facing output.
- Changing any bound artifact invalidates the review record.

### ADR-055 — Make statistical claim policy explicit and versioned

- A statistical policy defines comparison class, minimum paired samples, interval method, effect-size method, multiplicity policy, calibration policy, and allowed claim strength.
- Descriptive output remains available below inferential thresholds, but cannot be labeled conclusive.
- Exploratory, validation, and hidden-test comparisons remain distinct; hidden results cannot influence candidate selection.
- The policy is content-addressed and included in normalized comparison provenance.

### ADR-056 — Use one bounded contributor service behind the CLI

- Episode scaffold, validation, documentation, and review-bundle commands call application services through filesystem ports.
- Scaffolding refuses existing targets and writes only beneath a validated new package root.
- Validation never repairs, normalizes, or overwrites authored files.
- Public output cannot include private ground truth, private paths, query answers, or reviewer notes.

### ADR-057 — Represent deployment topology as a normative artifact

- A framework-neutral topology artifact identifies agents, roles, specialization, model assignments, delegation and coordination relationships, memory boundaries, allocation policy, execution pattern, and reviewer roles.
- Agent and relationship identifiers are stable, bounded, unique, and referentially valid.
- A single-agent topology is a first-class baseline.
- Protocol registration remains operational session data and must conform to, not replace, the authored topology artifact.

### ADR-058 — Prove control-variable equivalence for topology experiments

- A topology experiment identifies baseline and candidate artifact hashes, paired cells, controlled variables, and the exact declared experimental variables.
- Episode, seed, budgets, models, managed-tool policy, scoring profile, execution policy, binaries, and other declared controls must match unless explicitly varied.
- An undeclared difference makes the comparison ineligible; there is no best-effort equivalence.
- Every candidate change produces a new experiment identity and invalidates previous results.

### ADR-059 — Separate topology observation from experimental contribution

- Coordination overhead, duplicate work, evidence propagation, task allocation, parallelism, utilization, role activity, resilience, verified cost, and investigation quality remain separate measurements.
- Observational topology summaries never claim marginal or causal contribution.
- Marginal-agent and role contribution require a passing controlled ablation and are labeled experimental and topology-dependent.
- Role-specific results are not transferable to another topology unless a separate controlled experiment establishes that claim.

## 5. Planned architecture and contracts

```text
hunteval-domain        episode metadata, topology, experiment identities, normative DTOs
hunteval-statistics    statistical policy evaluation, effect size, uncertainty, calibration
hunteval-evaluation    topology measurement contracts over trusted observable inputs
hunteval-fixture-tool  deterministic scaffold/generation core without CLI or runner dependencies
hunteval-runner        contributor and topology experiment use cases through ports
hunteval-reporting     validated statistical and topology projections
hunteval-cli           thin command adapters
```

Allowed dependency direction remains inward. Fixture, filesystem, Parquet, CLI, and report rendering adapters must not enter `hunteval-domain` or `hunteval-statistics`.

The additive R4 schema set is expected to include:

- public episode classification metadata;
- private episode review record;
- statistical policy and calibration result;
- contributor validation and review-bundle manifests;
- deployment topology;
- topology experiment manifest and equivalence result;
- topology analysis and ablation result.

Exact field names, bounds, enum inventories, compatibility behavior, and canonical examples are frozen in R4-00 before implementation.

## 6. R4.1 — Episode coverage expansion

### R4-01 — Public episode classification metadata

1. **Objective:** define safe, reviewable difficulty and capability metadata without encoding answers.
2. **Affected areas:** domain episode contracts, additive schemas, canonical examples, loader adapters, contract documentation.
3. **Compatibility:** legacy episodes adapt with classification unavailable; source files remain unchanged.
4. **Security and isolation:** public enums and labels use a fixed allowlist; free-form private rationale cannot enter the public manifest.
5. **Tests:** canonical round trip, unknown version/field/tag, duplicates, empty values, oversized sets, legacy adaptation, serialization determinism, and explicit leakage fixtures.
6. **Acceptance:** every new episode declares validated classification metadata; old episodes remain readable and comparisons state unavailable classification.
7. **Migration and rollback:** retain readers and remove only the new writer on rollback; never synthesize tags for legacy inputs.

### R4-02 — Benign episodes and empty-ground-truth semantics

1. **Objective:** add at least one explicitly benign scored episode per provider and exercise the complete evaluation path.
2. **Affected areas:** fixture catalog/generator, datasets, benchmark manifests, reference queries, evaluation fixtures, reporting limitations.
3. **Contracts:** benign ground truth has empty malicious events/entities/path/techniques, explicit acceptable benign conclusions, and metric-specific applicability.
4. **Security and isolation:** public data may be ambiguous but cannot label the correct conclusion; private truth remains evaluator-only.
5. **Tests:** deterministic generation, empty denominators, false-positive findings, correct benign conclusion, unsupported path metrics, reference recovery, leakage, and end-to-end inclusion.
6. **Acceptance:** the paired matrix scores benign cases without fabricated positives, division-by-zero behavior, or implicit zero imputation.
7. **Rollback:** remove new benchmark membership through a new benchmark version; never rewrite a published definition.

### R4-03 — Multi-stage and cross-boundary episodes

1. **Objective:** add longer attack paths, ambiguous benign alternatives, and cross-account/project/tenant cases for every provider.
2. **Affected areas:** deterministic generators, telemetry schemas, ground-truth contracts, reference query fixtures, benchmark membership.
3. **Contracts:** stable stage identifiers, ordered expected paths, explicit time windows, boundary identifiers, and public investigation-shape metadata.
4. **Tests:** reordered/partial stages, long timelines, duplicate identifiers, cross-boundary joins, benign alternatives, deterministic bytes, reference recovery, and leakage.
5. **Acceptance:** each provider has deterministic multi-stage and cross-boundary coverage with exact package hashes and no deployment-visible answers.
6. **Rollback:** publish a new benchmark membership manifest excluding defective episodes while preserving the faulty bytes for audit.

### R4-04 — Independent review records and R4.1 closure

1. **Objective:** require content-addressed security review for new truth and reference queries.
2. **Affected areas:** private review contract, dataset validation service, reviewer fixtures, dataset CI gate, threat model and contribution guide.
3. **Contracts:** reviewer identity is opaque; approval binds public/private/query hashes, review policy version, timestamp, safe status, and bounded reason codes.
4. **Tests:** stale review, changed truth/query/public bytes, duplicate reviewer, missing approval, malformed timestamp, symlink/traversal, leakage in public projection, deterministic validation.
5. **Acceptance:** no new episode enters a benchmark release without a valid independent review record and passing reference recovery/leakage tests.
6. **Dependencies:** R4-01 through R4-03.

## 7. R4.2 — Statistical policy

### R4-05 — Comparison classes and minimum paired samples

1. **Objective:** define when output is descriptive, exploratory, validation-grade, hidden-test, or inferentially conclusive.
2. **Affected areas:** statistics contracts, additive schema, benchmark resolver, canonical policies, documentation.
3. **Contracts:** comparison class, minimum paired count, missing-pair handling, confidence level, claim-strength result, and stable insufficiency reasons.
4. **Tests:** below/at/above thresholds, missing/non-comparable pairs, invalid confidence, zero samples, unknown class, policy hashing, deterministic output.
5. **Acceptance:** sample count and claim strength accompany every comparative result; below-threshold results cannot be conclusive.
6. **Rollback:** preserve policy readers and fall back only to explicitly descriptive output.

### R4-06 — Effect sizes, uncertainty, and multiplicity

1. **Objective:** add bounded paired effect sizes and an explicit multiple-comparison policy.
2. **Affected areas:** statistics crate, comparison adapter, normalized result contracts, tests and metric documentation.
3. **Contracts:** paired mean/median difference where defined, standardized effect only with valid variance, interval, wins/ties/losses, family identifier, adjusted threshold or explicit no-adjustment exploratory label.
4. **Tests:** zero variance, ties, missing pairs, non-finite values, small samples, multiple families, deterministic resampling, directionality, and unavailable effect sizes.
5. **Acceptance:** every reported effect specifies method, direction, sample count, interval, policy, and applicability; unavailable values remain unavailable.
6. **Dependencies:** R4-05.

### R4-07 — Confidence and severity calibration

1. **Objective:** measure calibration only when comparable structured predictions and private outcomes exist.
2. **Affected areas:** statistics/evaluation contracts, trusted evaluator input, private reduction adapter, schemas and fixtures.
3. **Contracts:** bounded bins, Brier-style confidence error where applicable, severity confusion summary, sample count, applicability, and calibration-policy hash.
4. **Security and isolation:** only normalized aggregates leave the trusted evaluator; labels and per-case private outcomes are never deployment-visible.
5. **Tests:** perfect/inverted/constant confidence, empty and single-class sets, malformed values, severity mismatch, deterministic bins, and serialization without private labels.
6. **Acceptance:** calibration is unavailable when requirements fail and cannot be inferred from narrative text.

### R4-08 — Statistical claim projection and R4.2 closure

1. **Objective:** project statistical policy into normalized JSON and static HTML without overstating results.
2. **Affected areas:** reporting DTOs/renderers, runner normalization, schemas, verification and snapshot tests.
3. **Contracts:** exact policy digest, comparison class, sample counts, effects, intervals, multiplicity, calibration, claim strength, and limitations.
4. **Tests:** incomplete matrices, descriptive-only results, adjusted comparisons, unavailable metrics, escaping, deterministic JSON/HTML, tamper verification.
5. **Acceptance:** every comparative claim exposes sample size and uncertainty, and unsupported inference language is absent.
6. **Dependencies:** R4-05 through R4-07.

## 8. R4.3 — Dataset contribution tooling

### R4-09 — Safe episode scaffold

1. **Objective:** provide a contributor command that creates one minimal valid package skeleton.
2. **Affected areas:** fixture-tool core, runner contributor service, CLI arguments, templates, documentation.
3. **Contracts:** provider, episode ID, classification, output root, created-file inventory, and bounded deterministic template version.
4. **Tests:** supported providers, invalid IDs, existing target, traversal, symlink parent, absolute injected child path, partial-write cleanup, deterministic files, no private answer in public template.
5. **Acceptance:** scaffolding writes only beneath a new validated root, refuses overwrite, and produces a package that reaches validation with explicit incomplete authoring reasons.
6. **Rollback:** remove the command while retaining validation and templates as non-writing fixtures.

### R4-10 — Complete contributor validation

1. **Objective:** validate authored provider schemas, stable identifiers, deterministic generation, leakage, reference recovery, review status, and bounds through one command.
2. **Affected areas:** runner validation use case, fixture-tool adapters, provider validators, CLI, machine-readable result schema.
3. **Contracts:** ordered checks with `passed`, `failed`, `unavailable`, safe reason codes, artifact fingerprints, and no matched secret or ground-truth values.
4. **Tests:** malformed YAML/JSON/Parquet, unknown provider, schema drift, stale generated bytes, recovery mismatch, leakage, symlink/traversal, oversized files, and deterministic result ordering.
5. **Acceptance:** the command is read-only, fails closed, returns nonzero on any required failure, and validates every canonical episode.
6. **Dependencies:** R4-04 and R4-09.

### R4-11 — Documentation and review bundles

1. **Objective:** generate public package documentation and a bounded reviewer bundle from validated metadata.
2. **Affected areas:** reporting/templates, contributor service, CLI, review bundle manifest, secret scanning.
3. **Contracts:** public documentation contains safe metadata and checks only; private bundle has an explicit inventory and hashes and is never placed under public roots.
4. **Tests:** active-content escaping, private-field denial, deterministic output, stale validation, unsafe target, bundle bounds, tamper detection, and secret scan.
5. **Acceptance:** public docs are reproducible and ground-truth free; reviewer bundles are auditable, private, bounded, and content-addressed.
6. **Dependencies:** R4-10.

## 9. R4.4 — Multi-agent topology benchmarking

### R4-12 — Normative deployment-topology artifact

1. **Objective:** represent deployment topology explicitly and independently of any agent framework.
2. **Affected areas:** domain topology module, additive schema/example, benchmark deployment configuration, resolver, protocol conformance adapter, contracts and threat model.
3. **Contracts and compatibility:** topology kind, agents, roles, specialization, model assignment, delegation/coordination edges, memory boundaries, allocation policy, execution pattern, reviewer roles, exact artifact digest; legacy deployments have topology unavailable and are not silently classified.
4. **Security:** bounded untrusted labels; no embedded credentials, instruction bodies, environment values, tool authority, or private episode data.
5. **Tests:** positive variants, duplicate/dangling/self edges, cycles where forbidden, invalid supervisor/reviewer cardinality, inconsistent memory/allocation/execution declarations, oversized graphs, unknown fields/versions, deterministic hash, legacy behavior.
6. **Acceptance:** every new topology comparison binds a validated versioned artifact before execution; protocol registration must conform to its declared agent identities and roles.
7. **Migration and rollback:** retain topology-aware readers; disable new comparison eligibility rather than guessing topology for legacy artifacts.

### R4-13 — Required reference topology artifacts

1. **Objective:** bind single-agent, supervisor-worker, and supervisor-specialist reference deployments to normative artifacts and run them through one paired matrix.
2. **Affected areas:** reference deployment configurations, topology examples, resolver, conformance and benchmark fixtures.
3. **Tests:** artifact-to-registration parity, same managed-tool mediation, same episode/seed/budget/policy controls, deterministic execution, tampered topology, and framework-neutral serialization.
4. **Acceptance:** the three required reference topologies complete the same paired matrix and retain exact topology hashes in run and report provenance.
5. **Dependencies:** R4-12.

### R4-14 — Controlled topology experiment contract

1. **Objective:** prove that every changed variable is declared and every other relevant variable is equivalent.
2. **Affected areas:** domain experiment contracts, runner resolver/equivalence service, schemas, CLI validation, verification.
3. **Contracts:** baseline/candidate topology hashes, experiment class, changed-variable paths, controlled-variable inventory, paired cell identities, status, and stable mismatch reasons.
4. **Security and isolation:** hidden-test outcomes and private ground truth cannot appear in experiment selection input; immutable tool and safety policies cannot be changed as topology variables.
5. **Tests:** one and multiple declared changes, undeclared differences, stale hashes, episode/seed/budget/model/tool/scoring/policy/binary drift, hidden feedback, malformed paths, deterministic identity, and candidate invalidation.
6. **Acceptance:** an experiment is eligible only when every non-experimental control matches and every actual difference is declared.
7. **Rollback:** retain validation results but disable execution of new experiment manifests.

### R4-15 — Topology-aware observable metrics

1. **Objective:** formalize separate measurements for coordination overhead, duplicate work, evidence propagation, task allocation, parallelism, utilization, role activity, resilience, verified cost, and investigation quality.
2. **Affected areas:** trusted evaluation view, evaluation metric modules, registry, scoring-profile compatibility, statistics inputs, metric documentation.
3. **Contracts:** every metric defines range, direction, denominator, applicability, provenance, edge behavior, and topology requirements; contribution is excluded from observational metrics.
4. **Tests:** single-agent applicability, idle/missing agents, duplicate tasks, evidence handoff chains, sequential/parallel intervals, role reassignment, unavailable resource data, forged references, deterministic replay, and metric-registry parity.
5. **Acceptance:** raw dimensions remain independently visible; unsupported metrics are unavailable and no role contribution is inferred.
6. **Dependencies:** R4-12 and existing trusted observable reduction.

### R4-16 — Marginal-agent and controlled-ablation execution

1. **Objective:** execute auditable agent removal, specialist replacement, critic disablement, memory-boundary, and allocation-policy ablations.
2. **Affected areas:** runner experiment orchestrator, benchmark journal, statistics pairing, result verification, canonical experiment examples.
3. **Contracts:** baseline/candidate attempts, exact changed variables, control verification, quality/resource/coordination deltas, uncertainty, experimental status, topology-dependence label, and failure/inconclusive states.
4. **Tests:** each required ablation, interrupted resume, missing pairs, failed candidate, unchanged candidate, multiple changes, changed artifact invalidation, deterministic replay, and no hidden-result selection.
5. **Acceptance:** marginal-agent and ablation experiments produce content-addressed auditable artifacts and never promote observational association to causality.
6. **Dependencies:** R4-13 through R4-15.

### R4-17 — Topology comparison reporting

1. **Objective:** normalize and render controlled topology comparisons without universal role or agent rankings.
2. **Affected areas:** reporting DTOs/renderers, runner adapter, schemas, CLI, static snapshots and verification.
3. **Contracts:** topology identities, controls and changes, authoritative metric vectors, scoring profile, optional aggregate, constraint-first result, uncertainty, overhead/resource sections, experimental labels, limitations, and exact sources.
4. **Tests:** heterogeneous observability, unavailable topology metrics, non-equivalent controls, incomplete ablations, role labels, escaping, deterministic JSON/HTML, tamper detection, and prohibited universal-transfer language.
5. **Acceptance:** coordination overhead and resources are separate from quality; every contribution result is experimental/topology-dependent; unsupported equivalence is stated as a limitation.
6. **Dependencies:** R4-08 and R4-13 through R4-16.

### R4-18 — R4 release closure

1. **Objective:** prove all R4 release criteria on one revision and record exact evidence.
2. **Affected areas:** canonical dataset/topology CI gates, release checklist, README, roadmap, completion evidence, package secret scan.
3. **Tests:** full positive/negative dataset catalog, statistical policy, three-topology matrix, controlled ablations, verification, clean-cache parity, and package reproduction.
4. **Acceptance:** all R4.1–R4.4 exit criteria and canonical local/remote jobs pass on the same revision; exact hashes, known limitations, and ADR status changes are recorded.
5. **Dependencies:** R4-04, R4-08, R4-11, and R4-17.

## 10. Dependency graph

```text
R3 complete
  -> R4-00 contracts and ADRs

R4-00
  -> R4-01 classification
       -> R4-02 benign episodes
       -> R4-03 multi-stage/cross-boundary episodes
            -> R4-04 independent review and R4.1 closure

R4-00
  -> R4-05 comparison policy
       -> R4-06 effects and multiplicity
       -> R4-07 calibration
            -> R4-08 statistical reporting and R4.2 closure

R4-04
  -> R4-09 scaffold
       -> R4-10 validation
            -> R4-11 documentation/review bundles and R4.3 closure

R4-00
  -> R4-12 topology artifact
       -> R4-13 reference topologies
       -> R4-14 controlled experiment
       -> R4-15 topology metrics
            -> R4-16 ablations
R4-08 + R4-13 + R4-14 + R4-15 + R4-16
  -> R4-17 topology reporting and R4.4 closure

R4-04 + R4-08 + R4-11 + R4-17
  -> R4-18 release closure
```

R4.1 and R4.2 may proceed in parallel after R4-00. R4.3 depends on the completed dataset-review contract. R4.4 contract work may begin after R4-00, but controlled claims and final reporting depend on the statistical policy from R4.2.

## 11. Milestone handoff checklist

Before completing any R4 milestone:

1. objective and user-visible outcome are implemented without unrelated scope;
2. affected contracts have schema, canonical example, validation, and compatibility coverage;
3. security and ground-truth-isolation effects are documented and negatively tested;
4. positive, negative, malformed-input, deterministic/replay, and resource-bound tests pass;
5. metric additions define range, direction, denominator, applicability, provenance, and edge behavior;
6. experimental comparisons prove declared control equivalence and record every changed variable;
7. first-party production code contains no unsafe, panic shortcuts, unbounded input, private leakage, or unsupported causal claim;
8. source files remain cohesive and within repository size policy;
9. exact focused commands and canonical gates pass;
10. documentation, ADR status, migration, rollback, and known limitations are current;
11. `git diff --check` passes and no private, generated, or unrelated artifact is tracked;
12. a descriptive commit exists before status changes to `complete` with evidence.

Remote failure returns the milestone to active status until the same revision passes locally and remotely.

## 12. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| public classification discloses answer structure | benchmark leakage | fixed safe taxonomy, leakage tests, independent review, remove membership through a new benchmark version |
| benign episodes reward empty submissions | invalid quality signal | acceptable conclusion and false-positive contracts, evidence requirements, explicit applicability |
| expanded datasets become synthetic templates rather than realistic investigations | weak benchmark validity | provider-specific review, ambiguous alternatives, reference recovery, documented limitations |
| small sample claims appear conclusive | misleading comparison | versioned minimum-sample policy and descriptive-only fallback |
| multiplicity produces selective claims | false discoveries | explicit family policy, adjusted thresholds or exploratory label |
| calibration exposes private labels | ground-truth leakage | trusted aggregation boundary and normalized aggregate output only |
| scaffold overwrites contributor work | data loss | new-root-only semantics, no overwrite, bounded cleanup |
| review bundle enters a public root | private disclosure | separate validated destination, inventory, negative mount/leakage tests, secret scan |
| topology schema privileges one framework | reduced portability | framework-neutral identities and relationships, no provider-specific runtime fields |
| undeclared topology drift invalidates causality | false contribution claim | exhaustive control inventory and fail-closed equivalence |
| role result is presented as universal | invalid transfer claim | mandatory topology-dependent label and prohibited reporting language |
| additional metrics become an implicit global score | ranking distortion | authoritative metric vector, explicit scoring profile, no automatic inclusion |
| R4 breaks historical artifacts | loss of reproducibility | immutable schemas and fixtures, additive readers, rollback writer only |

## 13. R4 completion definition

R4 is complete only when:

1. every episode class has deterministic generation, reference recovery, leakage tests, and independent content-addressed review;
2. explicitly benign, multi-stage, cross-boundary, and ambiguous cases exist for every provider as required by R4.1;
3. benchmark versions identify exact membership, statistical policy, scoring profile, and relevant schema hashes;
4. every comparative claim exposes sample count, uncertainty, effect applicability, multiplicity treatment, and claim strength;
5. descriptive results cannot be presented as statistically conclusive;
6. fixture regeneration remains byte-identical under the pinned toolchain;
7. contributor scaffold, validation, documentation, and review-bundle workflows are bounded, deterministic, non-overwriting, and leakage-safe;
8. deployment topology is explicit in normative versioned artifacts;
9. single-agent, supervisor-worker, and supervisor-specialist deployments run through the same paired matrix with declared controls;
10. topology comparisons preserve every control variable and record every changed variable;
11. marginal-agent and controlled-ablation experiments produce auditable artifacts;
12. coordination overhead and resource trade-offs are separate from investigation quality;
13. unsupported topology metrics remain unavailable;
14. reports do not imply universal role or agent performance across topologies;
15. the complete quality, security, adversarial, dataset, statistical, topology, end-to-end, verification, and package gates pass locally and in GitHub Actions on the closure revision.

Completion evidence records exact commands, revisions, toolchains, benchmark membership and input hashes, review-policy hashes, statistical-policy hashes, topology and experiment hashes, runner/worker hashes, normalized result and verification hashes, secret-scan results, known limitations, and ADR status changes.

## 14. Initial acceptance command inventory

Existing commands remain mandatory:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/e2e.sh
git diff --check
```

The owning milestones must add stable focused commands equivalent to:

```bash
cargo test -p hunteval-domain --test schema_v06 --test science_v06
cargo test -p hunteval-runner --test cloud_fixtures
cargo test -p hunteval-runner --test dataset_review
cargo test -p hunteval-statistics --test statistical_policy
cargo test -p hunteval-statistics --test calibration
cargo test -p hunteval-fixture-tool --test contributor
cargo test -p hunteval-runner --test topology_equivalence
cargo test -p hunteval-evaluation --test topology_metrics
cargo test -p hunteval-reporting --test topology_reporting
cargo test -p hunteval-cli --test benchmark_validate
```

Exact test target names may change only in the milestone that creates them and must be reflected here in the same change.

The implemented focused R4 gate is:

```bash
./scripts/ci/r4-science.sh
```

It runs `schema_v06`, `science_v06`, `contributor`, `determinism`, `schema`, `cloud_fixtures`, `dataset_review`, `topology_equivalence`, `statistical_policy`, `calibration`, `topology_metrics`, `topology_reporting`, and `benchmark_validate`. The dataset review test verifies all nine approved R4 records against the versioned review policy. The E2E gate executes 108 cells and produces content-addressed controlled topology experiment, observation, JSON report, and static HTML report artifacts.
