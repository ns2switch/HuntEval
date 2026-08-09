# R5 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R5.1 through R5.3 into a reviewable implementation sequence. R5 expands HuntEval's narrow MVP diagnosis foundations into deterministic, evidence-backed failure classification, recurrence reporting, and controlled contribution and bottleneck analysis. It does not request private reasoning, infer intent from agent prose, validate deployment changes, adopt recommendations, or implement the R6 controlled improvement workflow.

R5 covers:

- a versioned failure taxonomy across investigation, evidence, tool use, coordination, resilience, and policy failures;
- exact attribution from classifications to validated runs, agents, actions, tasks, evidence, findings, metrics, topology artifacts, and benchmark cells;
- deterministic evidence-sufficiency levels and fail-closed omission of unsupported classifications;
- normalized JSON and static HTML diagnostic reports for individual runs and benchmark matrices;
- recurrence grouping over comparable cells without missing-value imputation;
- observable reassignment, queueing, duplicate-work, idle-time, managed-tool, and coordination bottleneck analysis;
- controlled agent-ablation contribution analysis built on R4 topology experiments and equivalence checks.

R2, R3, and R4 remain complete with their recorded implementation evidence. The R2.4 external-enforcement caveat was subsequently closed by the separate administrator attestation and protected release-candidate evidence without rewriting historical completion records. Existing schema 0.3 through 0.6 artifacts, benchmark execution, metric vectors, scoring-profile semantics, statistical policy, constraint-first ranking, sandboxing, protocol, verification, redaction, dataset review, and topology controls remain authoritative. R6 controlled improvement and prompt analysis remain future work.

### Delivery status

Status values are evidence-based. `planned` makes no implementation claim. `implemented` means behavior and focused local tests exist but release evidence is incomplete. `complete` requires a dedicated commit, focused acceptance tests, all canonical gates, documentation evidence, and passing GitHub Actions on that revision.

| Milestone | Status | Outcome or dependency |
|---|---|---|
| R5-00 | implemented | schema 0.7 diagnosis contracts, canonical examples, compatibility rules, and ADR-060 through ADR-066 are locally verified; remote release evidence remains pending |
| R5-01 | implemented | versioned bounded taxonomy and compiled rule registry |
| R5-02 | implemented | typed observable-source references and exact artifact resolution |
| R5-03 | implemented | deterministic evidence-sufficiency levels with no speculative probability |
| R5-04 | implemented | pure rule-based run classification and R5.1 closure |
| R5-05 | implemented | verified run-diagnosis application service and content-addressed artifact bundle |
| R5-06 | implemented | recurrence grouping across comparable cells and exact deployment versions |
| R5-07 | implemented | normalized JSON and static HTML diagnostic reporting |
| R5-08 | implemented | bounded diagnose/generate/verify CLI and R5.2 closure |
| R5-09 | implemented | runner-authoritative bottleneck observation projection |
| R5-10 | implemented | explicit reassignment, queueing, duplicate-work, idle-time, and managed-tool metrics |
| R5-11 | implemented | controlled agent-ablation contribution reduction using R4 controls |
| R5-12 | implemented | contribution and bottleneck report integration and R5.3 closure |
| R5-13 | implemented | deterministic diagnosis tests, adversarial inputs, and dedicated CI gate |
| R5-14 | active | local release gates and remote evidence remain required before R5 is complete |

The R5/v0.5 release name is independent from persisted schema versions. R5-00 selects additive schema `0.7` for diagnosis artifacts. Existing source artifacts remain byte-immutable and are never rewritten to simulate compatibility.

R5-00 local evidence includes offline meta-schema and canonical-example validation, fail-closed malformed and private-field cases, the canonical quality and security gates, adversarial protocol regression, and the R4 benchmark-science gate. The milestone remains `implemented`, not `complete`, until its dedicated revision passes the remaining release gates and GitHub Actions. No functional R5 classifier, resolver, report writer, verifier, or CLI command is claimed by this contract milestone.

## 2. Baseline audit

The repository already provides useful R5 foundations:

1. `TrustedRunView` is reduced from digest-verified trajectories and submissions and exposes a serializable `ObservedRun` with no ground-truth field.
2. Observable actions, tasks, evidence, findings, operational messages, task transitions, owners, and causal message references are already typed.
3. Four narrow classifiers map stable reason codes to observable event sequences or metric references and omit unknown or uncited failures.
4. Existing recommendations are deterministic hypotheses, remain `unvalidated`, and always require human review.
5. A basic diagnostic JSON renderer rejects uncited findings and rejects a validated status without a validation source.
6. The benchmark report retains exact run, cell, metric, comparison, constraint, and artifact references.
7. R4 topology artifacts, control-variable equivalence, statistical policy, paired ablation reduction, and topology-dependent reporting are complete.
8. Duplicate-work, useful-communication, task-completion, utilization, resilience, duration, resource-provenance, and topology metrics already provide reusable primitives.
9. Replay, verification, redaction, secret scanning, safe static HTML, and bounded report generation are canonical gates.

The audit also identifies the R5 gaps:

1. The current four-value `FailureKind` enum is not a persisted, reviewable taxonomy and does not cover all six roadmap categories.
2. Diagnostic evidence can cite only run IDs, event sequences, free-form metric strings, and a reason code; it cannot resolve typed agents, actions, tasks, evidence, findings, cells, topology artifacts, or exact artifact hashes.
3. The current taxonomy version is an internal `0.1` value with no JSON Schema, canonical example, content hash, or compatibility policy.
4. Classification confidence has no deterministic evidence-sufficiency contract.
5. The current diagnosis entrypoint consumes caller-authored `ObservableFailure` values instead of reducing verified run artifacts through an application service.
6. Diagnostic report types are not integrated into normalized run or benchmark reports, have no static HTML renderer, and do not group recurrence.
7. Recommendation text is a generic hypothesis and has no explicit claim-stage separation from observation, classification, experiment result, or approved change.
8. The CLI has no diagnosis generation or verification workflow.
9. Runner-authoritative event timestamps and lifecycle intervals are not projected into a dedicated bounded bottleneck input.
10. Reassignment, queueing, idle time, and managed-tool bottlenecks have no normative metric contracts or explicit unavailable states.
11. R4 controlled topology analysis reports topology-level deltas but does not project exact role/agent attribution into an R5 diagnostic contribution artifact.
12. There is no dedicated R5 end-to-end or GitHub Actions gate.

These gaps define the implementation order. R5 extends the existing trusted-view, topology experiment, statistical, reporting, and verification paths rather than creating parallel artifact or experiment systems.

## 3. Mandatory delivery rules

Every R5 pull request must:

- preserve the domain crate's independence from DuckDB, filesystem adapters, CLI parsing, provider SDKs, LLM providers, and agent frameworks;
- classify only validated structured observations, runner-owned failures, registered metrics, and content-addressed controlled experiments;
- never inspect prose to infer hidden reasoning, confusion, intent, motivation, or causal contribution;
- never request, store, or reconstruct private chain of thought;
- keep ground truth, reference answers, private review material, and hidden partition results outside public diagnostic artifacts;
- allow a public diagnosis to cite a normalized metric derived by the trusted evaluator without exposing the private records behind that metric;
- treat deployment text, reason strings, labels, taxonomy files, artifact paths, and report text as bounded untrusted input;
- require every classification and hypothesis to resolve to exact observable sources before serialization;
- omit unsupported classifications instead of guessing, assigning zero confidence, or creating placeholder claims;
- distinguish evidence sufficiency from probability and from causal validity;
- preserve observational attribution as observational and controlled contribution as experimental and topology-dependent;
- prove R4 control-variable equivalence before any agent or role contribution result is available;
- never create universal agent or role rankings across materially different topologies;
- keep investigation quality, diagnostic frequency, coordination overhead, bottlenecks, resources, and optional aggregate scores separate;
- preserve the raw metric vector as authoritative and never introduce a global score or implicit missing-value policy;
- keep R5-generated recommendation hypotheses unvalidated and non-adoptable; candidate artifacts, safe diffs, experiment orchestration, and approval lifecycle belong to R6;
- use stable Rust, typed errors, bounded collections, no first-party `unsafe`, and no panic shortcuts in production paths;
- keep production Rust files below 500 lines and split cohesive modules before 300 lines where practical;
- add positive, negative, malformed-input, deterministic/replay, compatibility, leakage, causal-overclaim, and resource-bound tests for every changed boundary;
- update schemas, contracts, metrics, threat model, ADRs, CLI documentation, migration behavior, rollback behavior, and known limitations with the behavior;
- keep all repository artifacts in English.

The canonical completion gates remain:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/e2e.sh
git diff --check
```

R5-13 adds one deterministic diagnosis gate. Once added, it becomes mandatory locally and in GitHub Actions.

## 4. Architecture decisions to close in R5-00

R5-00 is documentation and contract work. It proposes ADR-060 through ADR-066 without modifying accepted ADR-001 through ADR-059.

### ADR-060 — Add immutable evidence-backed diagnosis contracts

- Schema 0.7 adds taxonomy, source-reference, classification, diagnosis bundle, recurrence, bottleneck, contribution, and diagnostic-report artifacts.
- Schemas 0.3 through 0.6 remain immutable compatibility fixtures.
- Readers use explicit adapters where a prior narrow diagnosis type is supported; writers emit only schema 0.7.
- Unknown versions, fields, variants, references, confidence levels, and applicability states fail closed.

### ADR-061 — Separate reviewable taxonomy metadata from executable classifiers

- A content-addressed taxonomy defines stable codes, categories, safe descriptions, required source kinds, and evidence-sufficiency requirements.
- Classifier behavior remains typed Rust in `hunteval-evaluation`; taxonomy files are data, not executable expressions or scripts.
- The classifier registry and taxonomy must agree exactly on identifiers and versions.
- Changing taxonomy bytes or classifier semantics creates a new artifact identity and invalidates previous reproduction claims.

### ADR-062 — Make confidence an evidence-sufficiency label

- Confidence uses a bounded enum such as `direct`, `corroborated`, and `controlled`; it is not a model-generated probability.
- `direct` requires one complete rule-specific source set, `corroborated` requires independent typed source families or declared recurrence, and `controlled` additionally requires an eligible controlled experiment.
- A level cannot exceed the strongest evidence actually present.
- Missing required evidence omits the classification; it does not produce a low-confidence guess.

### ADR-063 — Resolve every diagnostic attribution against verified artifacts

- Source references are tagged, typed, bounded, and resolve to exact run, event, agent, action, task, evidence, finding, metric, cell, comparison, topology, or artifact identities.
- Resolution verifies artifact digests, ownership, same-run scope, event order, and referential integrity.
- Free-form JSON pointers cannot substitute for typed references when a typed identifier exists.
- Public diagnostic artifacts cannot contain ground-truth identifiers, private paths, or private hashes.

### ADR-064 — Treat diagnostic reports as deterministic projections

- Normalized JSON is the machine-readable source of truth; static HTML is a safe deterministic projection.
- Reports separate observations, classifications, recommendation hypotheses, controlled experiment results, and approved changes by explicit stage.
- R5 writers emit observations, classifications, unvalidated hypotheses, and eligible R4 experiment results. `approved_change` remains unavailable until a future R6 approval artifact exists.
- Every displayed conclusion resolves to an included source; untrusted text is escaped and active content is prohibited.

### ADR-065 — Separate recurrence from causal contribution

- Repeated classifications across comparable cells establish recurrence only.
- Agent or role contribution requires an eligible R4 controlled topology experiment, exact changed-variable inventory, paired observations, and the versioned statistical policy.
- Contribution results remain experimental and topology-dependent even when statistically supported.
- Observational recurrence, proximity, message wording, and role labels cannot create a contribution claim.

### ADR-066 — Derive bottlenecks from runner-authoritative intervals

- Queueing, task execution, tool wait, active-agent, and idle intervals use runner-authoritative trajectory order and timestamps.
- Overlapping intervals are unioned deterministically; negative, reversed, missing, cross-run, or ambiguous intervals fail or become explicitly unavailable according to the metric contract.
- Self-reported agent timing is never accepted as measured timing.
- Bottleneck measurements remain separate from investigation quality and verified resource costs.

## 5. Contract and compatibility strategy

R5-00 freezes JSON Schemas and canonical examples before Rust implementation. The initial schema 0.7 set includes:

- `diagnostic-taxonomy.schema.json` — taxonomy identity, categories, codes, source requirements, and confidence rules;
- `diagnostic-source-reference.schema.json` — tagged exact observable references;
- `failure-classification.schema.json` — code, category, attribution targets, evidence, confidence, and limitations;
- `run-diagnosis.schema.json` — one run's deterministic observations, classifications, omissions, and unvalidated hypotheses;
- `diagnostic-recurrence.schema.json` — comparable-cell grouping, exact deployment/configuration identities, sample accounting, and exclusions;
- `bottleneck-observations.schema.json` — bounded runner-authoritative lifecycle counts and intervals;
- `bottleneck-analysis.schema.json` — metric values, ranges, applicability, provenance, and limitations;
- `contribution-analysis.schema.json` — controlled experiment identity, topology hashes, changed variables, paired effects, attribution target, and experimental labels;
- `diagnostic-report.schema.json` — normalized run/benchmark diagnostic projection;
- `diagnostic-bundle-manifest.schema.json` — exact artifact inventory and hashes for offline verification.

The normative source-reference variants are planned to cover:

- trajectory event sequence plus trajectory digest;
- registered agent identity plus deployment and topology digest;
- managed action, task, evidence, finding, or operational-message identity scoped to a run;
- metric name/version and result digest;
- benchmark cell or statistical comparison identity;
- topology experiment, equivalence result, or analysis digest;
- generic safe relative artifact plus digest only when no stronger typed reference exists.

Every collection has an explicit bound. Identifiers use existing opaque domain types where available. Exact enum names, maximum sizes, confidence-source rules, and compatibility behavior are finalized in R5-00 and then treated as normative.

Compatibility rules are additive:

- schema 0.3 through 0.6 files remain byte-identical and readable by their existing readers;
- current internal diagnosis 0.1 values are not silently re-labeled as schema 0.7 artifacts;
- an explicit adapter may import a supported narrow diagnosis as `legacy_observation_only`, preserving missing typed attribution as unavailable;
- an old run or benchmark remains valid when no diagnostic bundle exists;
- a schema 0.7 report must never claim that missing legacy evidence is zero or absent behavior;
- rollback disables the schema 0.7 writer while retaining its reader and never deletes already emitted artifacts.

## 6. R5.1 — Versioned taxonomy and attribution

### R5-01 — Versioned taxonomy and compiled registry

1. **Objective:** establish one reviewable taxonomy that covers the six roadmap categories without executing contributor-authored rules.
2. **Affected areas:** schema 0.7 taxonomy contract/example, `taxonomies/diagnostic-failures-v1.json`, domain taxonomy types, evaluation classifier registry, contracts and threat-model documentation.
3. **Contracts:** `DiagnosticTaxonomy`, `FailureDefinition`, `FailureCategory`, `SourceRequirement`, stable taxonomy digest, and exact taxonomy/registry compatibility validation.
4. **Tests:** canonical schema validation; duplicate/unknown codes; empty categories; invalid bounds; unknown fields; registry drift; byte-deterministic serialization; malicious descriptions; taxonomy-size limits.
5. **Acceptance:** investigation, evidence, tool-use, coordination, resilience, and policy categories each have at least one typed rule; unsupported reason codes cannot enter a classification.
6. **Dependencies:** R5-00.
7. **Security, migration, and rollback:** taxonomy text is untrusted bounded data; executable expressions are prohibited; rollback removes the new taxonomy from the writer registry while preserving read support.

### R5-02 — Typed observable-source references

1. **Objective:** replace free-form diagnostic citations with exact source and attribution references that resolve against verified artifacts.
2. **Affected areas:** domain schema 0.7 types, evaluation source resolver ports, runner trusted-view/benchmark adapters, verification reason codes.
3. **Contracts:** `DiagnosticSourceReference`, `AttributionTarget`, `ResolvedDiagnosticSource`, `DiagnosticArtifactSet`, and typed resolution errors.
4. **Tests:** valid event/agent/action/task/evidence/finding/metric/cell/topology references; unknown, future, cross-run, wrong-owner, duplicate, stale-digest, private-path, private-hash, symlink, oversized, and malformed-pointer rejection.
5. **Acceptance:** every accepted reference resolves to exact content-addressed public or evaluator-safe evidence; no source can reference private ground-truth structure.
6. **Dependencies:** R5-00 and R5-01.
7. **Security, migration, and rollback:** runner performs bounded no-follow reads; adapters never serialize ground-truth provenance; legacy free-form references remain readable only through the explicit observation-only adapter.

### R5-03 — Deterministic evidence sufficiency

1. **Objective:** assign evidence-sufficiency levels by fixed typed rules rather than probabilistic or model-generated confidence.
2. **Affected areas:** evaluation sufficiency module, taxonomy rule validation, domain classification contract, metrics/diagnosis documentation.
3. **Contracts:** `EvidenceConfidence`, `EvidenceSufficiency`, `SourceFamily`, required/observed source inventory, and explicit limitation reason codes.
4. **Tests:** exact direct/corroborated/controlled thresholds; repeated references do not increase sufficiency; two references from one source family are not independent; missing evidence omits classification; controlled level requires eligible experiment; order-independent results.
5. **Acceptance:** the same resolved source set always produces the same level; no floating confidence score is emitted; sufficiency never implies causal validity.
6. **Dependencies:** R5-01 and R5-02.
7. **Security, migration, and rollback:** no model or network call is allowed; threshold changes require a new taxonomy version; rollback retains prior taxonomy readers.

### R5-04 — Pure rule-based classification and R5.1 closure

1. **Objective:** classify verified run failures deterministically and attribute only what observable evidence supports.
2. **Affected areas:** split evaluation modules for investigation, evidence, tools, coordination, resilience, and policy; runner diagnostic input adapter; domain classification result.
3. **Contracts:** `DiagnosticInput`, `FailureClassification`, `ClassificationOmission`, classifier registry version, taxonomy hash, and stable classification identity.
4. **Tests:** positive and negative rule fixtures for every taxonomy category; malformed inputs; unsupported source combinations; forged metric counts; agent-text injection; replay equivalence; ordering independence; bounded classification count.
5. **Acceptance:** identical verified inputs produce byte-equivalent ordered classifications; absent required evidence produces no claim; every classification includes exact taxonomy and source provenance.
6. **Dependencies:** R5-01 through R5-03.
7. **Security, migration, and rollback:** classifiers consume typed projections only and never parse message prose for meaning; each rule can be disabled by registry version rollback without modifying historical artifacts.

## 7. R5.2 — Diagnostic reports

### R5-05 — Verified run-diagnosis service

1. **Objective:** generate a content-addressed diagnosis from stored run artifacts through one application service.
2. **Affected areas:** runner `diagnostics` application module, filesystem ports, trusted evaluation input, artifact writer, bundle manifest, verifier integration.
3. **Contracts:** `RunDiagnosisRequest`, `RunDiagnosis`, `DiagnosticBundleManifest`, atomic output state, exact input/output digests, and typed generation errors.
4. **Tests:** completed, failed, partial, legacy, tampered, symlinked, oversized, missing, and concurrently replaced artifacts; atomic write failure; deterministic rerun; no-overwrite behavior unless exact regeneration is requested.
5. **Acceptance:** the service verifies inputs before classification, writes atomically, produces exact hashes, and cannot expose or require a private episode path.
6. **Dependencies:** R5-04 and existing run verification.
7. **Security, migration, and rollback:** no deployment process is started; reads are bounded and no-follow; disabling generation leaves run artifacts valid and unchanged.

### R5-06 — Recurrence grouping across comparable cells

1. **Objective:** identify repeated classifications across paired runs and deployment versions without converting missing cells into non-events.
2. **Affected areas:** evaluation recurrence reducer, runner benchmark diagnosis adapter, statistical sample accounting, domain recurrence contract.
3. **Contracts:** `DiagnosticRecurrenceGroup`, `ComparableDiagnosticCell`, occurrence count, eligible sample count, excluded-cell reasons, exact deployment/configuration/topology hashes, and descriptive-only claim strength.
4. **Tests:** repeated and unique classifications; different seeds; changed configuration; missing/failed/non-comparable cells; duplicate cells; topology drift; legacy evidence; deterministic grouping and stable identities.
5. **Acceptance:** only explicitly comparable cells share a recurrence group; numerator and denominator are visible; missing or failed cells are excluded with reasons and never imputed.
6. **Dependencies:** R5-05 and R4 statistical/comparability policy.
7. **Security, migration, and rollback:** recurrence is association, never causal contribution; rollback omits recurrence sections without changing per-run diagnoses.

### R5-07 — Normalized and static diagnostic reports

1. **Objective:** add deterministic diagnostic sections to machine-readable and human-readable run and benchmark reports.
2. **Affected areas:** reporting diagnostic modules split into types, validation, JSON, HTML, and components; runner report builder and verifier; schema 0.7 report/example.
3. **Contracts:** `DiagnosticReport`, `DiagnosticSection`, `DiagnosticClaimStage`, `DiagnosticClaim`, `RecommendationHypothesis`, recurrence, bottleneck, contribution, limitations, and artifact inventory.
4. **Tests:** JSON snapshot determinism; script-free HTML; escaping of every untrusted field; source-link resolution; stage confusion; uncited claims; fake validation; private-path leakage; oversized text/collections; accessibility landmarks.
5. **Acceptance:** JSON is authoritative; HTML contains no script or event handler; observation, classification, hypothesis, experiment result, and approved change are visibly distinct; pre-R6 output cannot claim an approved change.
6. **Dependencies:** R5-05 and R5-06; contribution sections remain unavailable until R5-11.
7. **Security, migration, and rollback:** render structured cited fields only; retain old report readers; a rollback can omit the additive diagnosis section without rewriting older reports.

### R5-08 — Diagnosis CLI, verification, and R5.2 closure

1. **Objective:** expose bounded offline generation and verification without embedding business rules in CLI parsing.
2. **Affected areas:** CLI args/controller modules, runner diagnostic service and verifier, `BENCHMARK_CLI.md`, generated shell completion if present.
3. **Contracts:** commands equivalent to `diagnose run`, `diagnose benchmark`, and `diagnose verify`; JSON status output; documented exit codes; safe output-path behavior.
4. **Tests:** argument parsing; stdout/stderr separation; existing-target refusal; traversal, absolute path, symlink, malformed taxonomy, stale digest, missing source, invalid format, and deterministic output; help snapshots.
5. **Acceptance:** generation is offline and read-only with respect to source runs; verification detects any referenced-byte change; non-success and unsupported states return documented nonzero codes.
6. **Dependencies:** R5-05 through R5-07.
7. **Security, migration, and rollback:** commands never receive ground-truth paths, start deployments, or use network access; removing the command leaves persisted artifacts independently readable.

## 8. R5.3 — Contribution and bottleneck analysis

### R5-09 — Runner-authoritative bottleneck observations

1. **Objective:** build a bounded diagnostic projection of task, agent, message, and managed-tool lifecycle intervals from verified trajectories.
2. **Affected areas:** runner stored-input projection, evaluation diagnostic input types, domain bottleneck observation schema, protocol compatibility fixtures only if an existing event lacks required observable data.
3. **Contracts:** `BottleneckObservations`, task lifecycle, assignment/reassignment, tool request/result, active-agent, queue, and measured-duration intervals plus explicit availability reasons.
4. **Tests:** complete and partial lifecycles; reassignment; overlapping tasks; simultaneous timestamps; reversed/missing timestamps; future/cross-run references; timed-out tools; failed agents; legacy trajectories; interval union determinism.
5. **Acceptance:** all timing uses runner-authoritative events; no self-reported duration is treated as measured; unsupported intervals remain unavailable.
6. **Dependencies:** R5-02 and existing replay/trusted-view validation.
7. **Security, migration, and rollback:** no protocol change is made unless R5-00 proves it necessary; any protocol addition is optional, versioned, backward-compatible, and unavailable for older runs rather than inferred.

### R5-10 — Bottleneck metric contracts

1. **Objective:** report reassignment, queueing, duplicate work, idle time, and managed-tool bottlenecks as separate measurements with exact edge behavior.
2. **Affected areas:** evaluation bottleneck metric modules, metric registry, `METRICS_AND_RANKING.md`, domain metric results, statistical summaries.
3. **Contracts:** at minimum reassignment count/rate, task queue duration/utilization, duplicate-work count/rate, agent idle duration/utilization, managed-tool wait/error/timeout summaries, and supervisor concentration where topology permits. Every metric defines range, direction, numerator, denominator, applicability, provenance, and limits.
4. **Tests:** zero denominators; unavailable duration; single-agent baseline; no tasks/tools; failed and incomplete runs; overlaps; caps; non-finite values; malformed counts; resource provenance; deterministic aggregation.
5. **Acceptance:** overhead and bottleneck metrics remain separate from investigation quality and optional scoring; no metric is silently added to a scoring profile; unavailable values are never zero-imputed.
6. **Dependencies:** R5-09 and the existing metric registry.
7. **Security, migration, and rollback:** metric versions are additive; rollback removes new selections from writers while preserving raw observations and readers.

### R5-11 — Controlled agent-ablation contribution analysis

1. **Objective:** attribute experimental contribution only when a controlled R4 topology ablation provides exact eligible evidence.
2. **Affected areas:** evaluation contribution reducer, runner topology experiment adapter, statistics policy integration, domain contribution schema, reporting source resolution.
3. **Contracts:** `ContributionTarget`, `ControlledContributionAnalysis`, topology and experiment hashes, equivalence digest, exact changed variables, paired cells, metric effects, uncertainty, claim strength, applicability, and mandatory experimental/topology-dependent limitations.
4. **Tests:** remove agent; specialist-to-generalist replacement; critic disablement; missing/failed pairs; undeclared drift; changed budgets/models/tools/scoring; multiple changed agents; insufficient samples; multiplicity guard; role-label mismatch; deterministic reduction.
5. **Acceptance:** observational input cannot produce contribution; ineligible or underpowered experiments remain unavailable/descriptive; valid results cite the exact agent/role and topology and cannot be generalized across topologies.
6. **Dependencies:** R4 controlled topology experiment/equivalence/statistics and R5-02, R5-03, and R5-10.
7. **Security, migration, and rollback:** no new experiment orchestrator is added; R5 consumes R4 artifacts; rollback retains R4 topology reports and omits the additive R5 contribution projection.

### R5-12 — Contribution and bottleneck report integration and R5.3 closure

1. **Objective:** integrate separate bottleneck measurements and controlled contribution results into diagnostic JSON and static HTML.
2. **Affected areas:** reporting components, runner benchmark diagnosis builder, CLI output, verification, use-case documentation.
3. **Contracts:** per-run bottleneck sections, benchmark bottleneck summaries, controlled contribution claims, experimental labels, unavailable reason codes, exact source links, and limitations.
4. **Tests:** controlled and observational reports; unavailable metrics; mixed topology/configuration; escaping; no universal role ranking; no implicit aggregate; source tampering; JSON/HTML equivalence.
5. **Acceptance:** reports answer where observable work accumulated and what a controlled ablation changed while keeping overhead, resources, quality, recurrence, and contribution distinct.
6. **Dependencies:** R5-07, R5-10, and R5-11.
7. **Security, migration, and rollback:** report validators reject causal wording without controlled sources; removing the new sections does not invalidate the underlying benchmark result.

## 9. Integration and closure

### R5-13 — End-to-end diagnosis and CI gate

1. **Objective:** prove deterministic diagnosis, recurrence, bottleneck, contribution, reporting, and verification on one reproducible artifact set.
2. **Affected areas:** `scripts/ci/r5-diagnosis.sh`, E2E artifact manifest, GitHub Actions diagnosis job, deterministic fixtures, negative corpus, secret scan.
3. **Contracts:** bounded CI artifact inventory, exact hashes, verifier result, unsupported-claim inventory, and no-private-data scan result.
4. **Tests:** canonical 108-cell benchmark diagnoses; at least one task failure/reassignment fixture; one managed-tool bottleneck; one recurrent classification; one eligible controlled ablation; observational-only negative; tampering; malformed schema; hostile report text; clean regeneration.
5. **Acceptance:** repeated execution from the same inputs produces byte-equivalent diagnosis artifacts; every generated claim verifies; negative fixtures fail for stable typed reasons; CI publishes only bounded non-secret outputs.
6. **Dependencies:** R5-04, R5-08, and R5-12.
7. **Security, migration, and rollback:** the job uses current least-privilege permissions and retention; workflow rollback removes only the new job after reverting its owning milestone.

### R5-14 — R5 release closure

1. **Objective:** prove all R5 release criteria on one revision and record exact evidence.
2. **Affected areas:** README, roadmap, R5 plan status, completion-evidence document, release checklist, package schema inventory, CI evidence.
3. **Contracts:** exact revisions, taxonomy and classifier hashes, schema hashes, run/benchmark diagnosis hashes, topology experiment hashes, report/verifier hashes, toolchain, and known limitations.
4. **Tests:** full local gates, clean-tree E2E, diagnosis gate, release-candidate dry run, secret scan, and all canonical GitHub Actions jobs.
5. **Acceptance:** all R5.1–R5.3 exit criteria pass locally and remotely on the evidence revision; completion evidence records exact outcomes without changing R2–R4 evidence.
6. **Dependencies:** R5-04, R5-08, R5-12, and R5-13.
7. **Security, migration, and rollback:** no production release is published; a remote failure returns R5 to active status; rollback preserves schema readers and historical artifacts.

## 10. Dependency graph

```text
R4 complete
  -> R5-00 contracts and ADRs

R5-00
  -> R5-01 taxonomy/registry
       -> R5-02 typed source resolution
            -> R5-03 evidence sufficiency
                 -> R5-04 classification and R5.1 closure

R5-04
  -> R5-05 run-diagnosis service
       -> R5-06 recurrence
            -> R5-07 JSON/HTML reports
                 -> R5-08 CLI/verification and R5.2 closure

R5-02 + existing replay
  -> R5-09 bottleneck observations
       -> R5-10 bottleneck metrics

R4 controlled topology artifacts + R5-02 + R5-03 + R5-10
  -> R5-11 controlled contribution

R5-07 + R5-10 + R5-11
  -> R5-12 contribution/bottleneck reporting and R5.3 closure

R5-04 + R5-08 + R5-12
  -> R5-13 end-to-end/CI
       -> R5-14 release closure
            -> R6 controlled improvement planning
```

R5.1 is the critical path. R5-09 observation projection may proceed after typed source resolution, but R5-10 and R5-11 cannot close before taxonomy, sufficiency, and reporting contracts are stable. R5.3 reuses R4 controls and must not create a parallel experiment format.

## 11. Milestone handoff checklist

Before completing any R5 milestone:

1. objective and user-visible outcome are implemented without unrelated R6 scope;
2. affected contracts have schema, canonical example, validation, and compatibility coverage;
3. security and ground-truth-isolation effects are documented and negatively tested;
4. positive, negative, malformed-input, deterministic/replay, causal-overclaim, and resource-bound tests pass;
5. every classification and hypothesis resolves exact observable sources;
6. every metric defines range, direction, denominator, applicability, provenance, edge behavior, and tests;
7. experimental contribution proves declared control equivalence and remains topology-dependent;
8. first-party production code contains no unsafe, panic shortcuts, unbounded input, private leakage, prose-derived hidden reasoning, or unsupported causal claim;
9. source files remain cohesive and within repository size policy;
10. exact focused commands and canonical gates pass;
11. documentation, ADR status, migration, rollback, and known limitations are current;
12. `git diff --check` passes and no private, generated, or unrelated artifact is tracked;
13. a descriptive commit exists before status changes to `complete` with evidence.

Remote failure returns the milestone to active status until the same revision passes locally and remotely.

## 12. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| classifier interprets agent prose | fabricated hidden reasoning | typed structured sources only; prose injection tests; omit unsupported classification |
| diagnostic artifact leaks ground truth | benchmark compromise | public source allowlist; evaluator-safe metric citation; private-path/hash rejection; secret scan |
| taxonomy file becomes executable | code injection or nondeterminism | metadata-only schema; compiled typed classifier registry; exact registry compatibility |
| confidence appears probabilistic | misleading certainty | bounded evidence-sufficiency enum; publish source inventory; no floating confidence |
| repeated references inflate confidence | false corroboration | deduplicate exact sources and require independent source families |
| recurrence is presented as causality | unsupported diagnosis | explicit descriptive stage and limitations; controlled experiment required for contribution |
| stale artifact references support claims | unauditable report | exact digest resolution and offline verification before rendering |
| malicious report text becomes active content | HTML injection | structured rendering, escaping, no scripts/handlers, untrusted snapshots |
| missing cells are treated as absence | biased recurrence | explicit eligible denominator and exclusion reasons; no imputation |
| timing jitter creates false bottlenecks | misleading performance analysis | runner-authoritative intervals, deterministic union, explicit unavailable states |
| idle time penalizes topology design incorrectly | invalid comparison | topology-aware denominators, role availability, separate metric vector, limitations |
| observational role activity becomes contribution | false causal claim | R4 equivalence gate, controlled ablation only, experimental/topology-dependent labels |
| role results become universal rankings | invalid transfer claim | exact topology/configuration binding and prohibited cross-topology ranking |
| diagnosis silently changes scoring | ranking distortion | diagnostic metrics excluded from profiles by default; explicit versioned selection only |
| R5 absorbs R6 prompt optimization | safety and scope expansion | hypotheses only; no candidate patch, artifact registry, validation lifecycle, or adoption |
| schema 0.7 breaks old artifacts | loss of reproducibility | immutable earlier schemas, explicit adapters, optional additive reports, writer rollback only |

## 13. R5 completion definition

R5 is complete only when:

1. the six-category taxonomy is versioned, content-addressed, bounded, reviewable, and exactly matched by a compiled classifier registry;
2. every classification resolves to exact validated observable sources and typed attribution targets;
3. missing evidence omits classifications rather than producing guesses or zero-valued claims;
4. evidence sufficiency is deterministic, non-probabilistic, and does not imply causality;
5. the same verified artifact set produces byte-equivalent run diagnoses and recurrence groups;
6. recurrence reports expose eligible samples, occurrences, and every excluded cell without imputation;
7. normalized JSON and static HTML separate observation, classification, hypothesis, experiment result, and approved-change availability;
8. every displayed claim verifies against an included content-addressed source;
9. reports contain no private ground truth, private paths, private hashes, secret values, active content, or private chain of thought;
10. reassignment, queueing, duplicate-work, idle-time, and managed-tool bottlenecks have normative metric contracts and explicit unavailable states;
11. bottleneck, resource, recurrence, investigation-quality, and contribution dimensions remain separate;
12. agent or role contribution is unavailable without an eligible controlled topology experiment;
13. controlled contribution is labeled experimental and topology-dependent and is never generalized across topologies;
14. no R5 output can validate or adopt a recommendation or modify a deployment artifact;
15. complete quality, security, adversarial, benchmark-science, diagnosis, end-to-end, verification, documentation, and package gates pass locally and in GitHub Actions on the evidence revision.

Completion evidence records exact commands, revisions, toolchains, taxonomy and classifier hashes, schema hashes, source artifact hashes, run and benchmark diagnosis hashes, recurrence sample accounting, bottleneck and controlled-experiment hashes, normalized report and verification hashes, secret-scan results, known limitations, and ADR status changes.

## 14. Initial acceptance command inventory

Existing commands remain mandatory:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/e2e.sh
git diff --check
```

Owning milestones add focused commands equivalent to:

```bash
cargo test -p hunteval-domain --test schema_v07 --test diagnosis_v07
cargo test -p hunteval-evaluation --test diagnostic_taxonomy
cargo test -p hunteval-evaluation --test diagnostic_classification
cargo test -p hunteval-evaluation --test diagnostic_recurrence
cargo test -p hunteval-evaluation --test bottleneck_metrics
cargo test -p hunteval-evaluation --test contribution_analysis
cargo test -p hunteval-runner --test diagnostic_service
cargo test -p hunteval-runner --test diagnostic_benchmark
cargo test -p hunteval-reporting --test diagnostic_reporting
cargo test -p hunteval-reporting --test diagnostic_report
cargo test -p hunteval-cli --test diagnose
```

R5-13 must add:

```bash
./scripts/ci/r5-diagnosis.sh
```

Exact target names may change only in the milestone that creates them and must be updated here in the same change. No milestone is complete while a required local or remote gate fails.

## 15. Known limitations retained through R5

- Diagnosis is deterministic and evidence-backed but remains limited to registered rules and observable structured inputs.
- Absence of a classification does not prove absence of a deployment weakness.
- Recurrence identifies repeated observable patterns, not root cause.
- Controlled contribution remains experimental and specific to the exact topology, artifact set, benchmark matrix, statistical policy, and declared changed variables.
- Timing-based bottlenecks depend on runner-authoritative observable lifecycle events and remain unavailable for insufficient legacy traces.
- Verified provider cost remains unavailable without a verifiable adapter.
- R5 does not generate candidate deployment changes, validate prompt/configuration patches, expose hidden-test feedback, or approve/adopt recommendations.
- No production SIEM connector, unrestricted network access, distributed execution, web dashboard, or autonomous optimization is introduced.
