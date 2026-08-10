# R6 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R6.1 through R6.4 into a reviewable implementation sequence. R6 makes improvement hypotheses reproducible from content-addressed artifact registration through controlled validation and an explicit human decision. It extends the completed R4 experiment controls and R5 evidence-backed diagnosis. It does not infer private reasoning, expose hidden-test feedback during candidate generation or selection, modify registered deployment artifacts in place, weaken immutable safety policy, or autonomously adopt a suggested change.

R6 covers:

- bounded registration of deployment configuration, instruction, output-contract, and related experimental artifacts by exact content hash;
- structural baseline/candidate diffs with an exact changed-variable inventory;
- immutable safety and trust-boundary enforcement plus benchmark-answer leakage checks;
- paired baseline/candidate execution over explicit training and validation partitions using the existing benchmark runner;
- constraint-first validation with uncertainty, missing cells, verified-resource provenance, and regression limits;
- an auditable recommendation lifecycle from proposal through testing, validation or rejection, human review, and externally confirmed adoption;
- a versioned prompt/configuration failure taxonomy and evidence-backed prompt improvement analysis;
- optional candidate patch suggestions that remain separate artifacts and never mutate or adopt deployment instructions automatically.

R2, R3, R4, and R5 remain complete with their recorded evidence. The R2.4 external-enforcement caveat remains closed by the separate administrator attestation. Existing schemas 0.3 through 0.7, protocol behavior, metric vectors, scoring profiles, statistical policy, topology controls, diagnostic artifacts, and completion evidence remain authoritative and byte-immutable. Runtime revision `079bf45` and governance revision `aacf27c` complete R6. All eleven canonical GitHub Actions jobs passed in run `31376156815`.

### Delivery status

Status is evidence-based. `planned` makes no implementation claim. `implemented` means focused behavior and local tests exist but release evidence is incomplete. `complete` requires a dedicated commit, focused acceptance tests, all canonical gates, documentation evidence, and passing GitHub Actions on that revision.

| Milestone | Status | Outcome or dependency |
|---|---|---|
| R6-00 | complete | schema 0.8 contracts, canonical examples, compatibility rules, and accepted ADR decisions |
| R6-01 | complete | bounded content-addressed artifact registry with deterministic inventory and verification |
| R6-02 | complete | deterministic structured diff with immutable-change rejection |
| R6-03 | complete | full immutable coverage and bounded answer-leakage checks with safe reason codes |
| R6-04 | complete | digest-bound fail-closed controlled equivalence |
| R6-05 | complete | evaluator-only partition authorization and single-use sealed final assessment |
| R6-06 | complete | paired orchestration through the canonical benchmark service and journal |
| R6-07 | complete | constraint-first decision preserving raw pairs, missing values, and provenance |
| R6-08 | complete | additive validate/run/resume/status/verify CLI and offline bundle verification |
| R6-09 | complete | append-only hash-linked lifecycle and deterministic state projection |
| R6-10 | complete | explicit human review and separately confirmed external adoption records |
| R6-11 | complete | digest-bound invalidation and stage-separated JSON/static-HTML reporting |
| R6-12 | complete | fourteen-category taxonomy with exact compiled-registry agreement |
| R6-13 | complete | observable-source resolution and bounded weakness hypotheses |
| R6-14 | complete | separate non-authoritative suggestion materialization without in-place writes |
| R6-15 | complete | controlled evidence linkage and content-addressed auditable bundles |
| R6-16 | complete | deterministic R6 end-to-end test and dedicated local/GitHub Actions gate |
| R6-17 | complete | local, package, protected-branch, and eleven-job remote closure evidence recorded |

The R6/v0.6 release name is independent from persisted schema versions. R6-00 selects additive schema `0.8` through accepted ADR-067. Existing artifacts are never rewritten to simulate compatibility.

## 2. Baseline audit at plan approval

The repository already provides the following R6 foundations:

1. Schema 0.7 diagnoses cite exact observable run, event, agent, action, task, evidence, finding, metric, benchmark-cell, comparison, topology-experiment, and artifact identities.
2. Recommendation hypotheses are deterministic, content-addressed through their diagnostic bundle, always require validation, and cannot currently claim approval.
3. R4 topology experiments prove declared control-variable equivalence and preserve paired cells, statistical policy, uncertainty, applicability, and topology-dependent limitations.
4. The benchmark controller already executes, journals, resumes, and verifies bounded matrices without overwriting prior attempts.
5. Versioned scoring profiles preserve the raw metric vector, explicit missing-value policy, verified-resource constraints, and constraint-first ranking.
6. Public run verification, diagnostic-bundle verification, bounded no-follow reads, redaction, secret scanning, deterministic JSON, and script-free HTML are established infrastructure boundaries.
7. A narrow in-memory experiment prototype validates one changed variable, three immutable hash categories, training/validation separation, metric regression, verified cost, and mandatory human review.
8. Deployment registrations record prompt hashes, while resolved benchmark inputs already bind configuration, episode, scoring, execution, schema, and relevant binary digests.

The audit identified the following gaps before implementation. Revision `079bf45` closes them through the owning milestones and tests recorded in the delivery table; they are retained here as the historical design input:

1. There is no persisted versioned artifact registry or structural artifact contract.
2. Existing deployment manifests do not expose a normative section inventory that can distinguish mutable instructions from immutable policy.
3. The current experiment prototype uses unbounded strings and maps, has no JSON Schema, artifact identity, paired-cell accounting, uncertainty, replay, journal, or verifier, and is not R6 evidence.
4. Immutable coverage is narrower than the roadmap requirement and does not include filesystem, network, ground-truth isolation, benchmark constraints, output integrity, or security controls.
5. No trusted partition service prevents hidden-test result access during candidate generation and selection.
6. No application service executes paired baseline/candidate matrices as one controlled improvement experiment.
7. No lifecycle artifact distinguishes proposed, testing, validated, rejected, approved, adopted, invalidated, and superseded facts without overwriting history.
8. No human-decision artifact binds the exact candidate, experiment, result, policy, and reviewer assertion.
9. Changing a candidate artifact does not yet invalidate prior validation automatically.
10. The schema 0.7 diagnostic taxonomy classifies observable deployment failures, but there is no separate reviewable taxonomy for prompt/configuration weaknesses.
11. There is no structural prompt inspection, safe suggested-change artifact, controlled A/B linkage, or R6 report/CLI surface.
12. There is no dedicated R6 end-to-end or GitHub Actions gate.

R6 extends these foundations instead of creating a parallel runner, scoring system, diagnostic system, or experiment format.

## 3. Mandatory delivery rules

Every R6 pull request must:

- preserve the domain crate's independence from filesystems, DuckDB, CLI parsing, process management, LLM providers, agent frameworks, and report rendering;
- keep scored execution in HuntEval and reuse the existing benchmark runner, sandbox, journal, verification, statistics, and reporting ports;
- consume only verified public or evaluator-safe observable diagnosis sources and never request, store, reconstruct, or infer private chain of thought;
- keep ground truth, hidden partition membership, reference answers, private review material, and hidden-test feedback outside candidate generation and selection inputs;
- treat artifact content, section labels, diff operations, reviewer labels, rationale text, and generated suggestions as bounded untrusted input;
- register exact bytes before comparison and use lowercase SHA-256 content identities throughout;
- prove exactly one declared experimental artifact variable changed while recording every changed structural section;
- fail closed when a baseline or candidate is opaque, malformed, linked, oversized, unsupported, stale, or not structurally comparable;
- reject any modification to authorization, tool access, filesystem, network, data handling, ground-truth isolation, benchmark constraints, output integrity, security controls, or another registered immutable section;
- reject private reasoning requests, direct scored-tool access, hidden-answer material, episode-specific answer memorization, and required-provenance removal;
- keep objective measurements, raw metric vector, scoring profile, optional aggregate score, and comparison/ranking in their authoritative order;
- never impute missing, failed, unavailable, non-comparable, or unverifiable measurements;
- require a passing controlled experiment before `validated`, and an exact explicit human decision before `adopted`;
- treat `validated` as experimental support for the exact candidate and controls, not universal causal proof;
- invalidate validation when candidate, baseline, experiment, partition policy, statistical policy, scoring profile, constraint policy, schemas, or relevant binaries change;
- never modify or replace a deployment artifact as a side effect of diagnosis, suggestion, validation, review, or adoption recording;
- prohibit autonomous adoption through R6 and v1.0; an adoption record only attests an external human action against an exact digest;
- use stable Rust, typed errors, bounded collections, no first-party `unsafe`, and no `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` in production paths;
- keep production Rust files below 500 lines and split cohesive modules before 300 lines where practical;
- add positive, negative, malformed-input, deterministic/replay, compatibility, leakage, hidden-test isolation, stale-artifact, authorization, and resource-bound tests for every changed boundary;
- update contracts, threat model, metrics semantics, ADRs, CLI documentation, migration, rollback, and known limitations with the behavior;
- keep all repository artifacts in English.

The canonical gates remain:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/e2e.sh
git diff --check
```

R6-16 adds the deterministic controlled-improvement gate shown above. It is mandatory locally and in GitHub Actions.

## 4. Architecture and dependency direction

R6 keeps Clean Architecture boundaries explicit:

```text
hunteval-domain
  <- hunteval-evaluation
  <- hunteval-statistics

hunteval-domain + hunteval-statistics
  <- hunteval-reporting

domain/application/reporting ports + infrastructure adapters
  <- hunteval-runner <- hunteval-cli

hunteval-sandbox and filesystem/process adapters
  -> hunteval-runner only
```

- `hunteval-domain` owns schema 0.8 value objects, opaque identifiers, lifecycle states, hashes, references, validation invariants, and typed errors. It has no filesystem, process, provider, CLI, or rendering dependency.
- `hunteval-evaluation` owns pure artifact-diff policy, immutable-section decisions, leakage-result reduction, controlled-equivalence validation, prompt weakness mapping, and candidate decision logic. It receives bytes or typed values through caller-owned ports and performs no I/O.
- `hunteval-statistics` owns paired summaries and policy-bound uncertainty; R6 does not implement a second statistics engine.
- `hunteval-runner` owns bounded no-follow artifact loading, registry storage, partition authorization, benchmark orchestration, append-only lifecycle journals, atomic projections, invalidation, and offline verification.
- `hunteval-reporting` receives validated DTOs and renders deterministic JSON plus escaped static HTML without reading private artifacts.
- `hunteval-cli` parses commands, invokes application services, separates stdout from diagnostics, and contains no experiment or approval policy.
- `hunteval-sandbox` continues to enforce process and network boundaries. R6 introduces no unrestricted network path and no direct tool execution.

No provider-specific, SIEM-specific, LLM-specific, or agent-framework-specific dependency is added. A new crate requires an ADR and dependency-direction proof; the initial plan keeps R6 within the existing boundaries and splits submodules by cohesion.

## 5. Decisions accepted in R6-00

R6-00 accepted ADR-067 through ADR-075 without modifying accepted ADR-001 through ADR-066. Revision `079bf45` implements their runtime behavior and tests.

### ADR-067 — Add immutable controlled-improvement contracts

- Schema 0.8 adds artifact registration, structural diff, controlled improvement experiment, equivalence, validation decision, lifecycle, human decision, prompt recommendation, and bundle-manifest artifacts.
- Schemas 0.3 through 0.7 remain byte-immutable and readable through their existing behavior.
- Unknown versions, fields, variants, states, references, policies, or applicability values fail closed.

### ADR-068 — Register exact bytes and compare only declared structure

- Every experimental artifact is identified by its exact bytes, kind, media type, schema, and SHA-256 digest.
- Structurally diffable instruction/configuration artifacts use a bounded versioned section inventory. Opaque legacy artifacts may be registered for provenance but cannot enter structural experiments.
- Diff operations use typed section identifiers and operations; arbitrary filesystem patches and inferred Markdown structure are not normative evidence.

### ADR-069 — Put immutable safety policy outside candidate authority

- Immutable section classes include authorization, tool access, filesystem, network, data handling, ground-truth isolation, benchmark constraints, output integrity, and security controls.
- The immutable policy registry is versioned and content-addressed. Candidate files cannot redefine which sections are immutable.
- Any missing, changed, reclassified, reordered in a semantically relevant way, or ambiguously parsed immutable section makes the candidate ineligible.

### ADR-070 — Separate candidate selection from hidden-test evaluation

- A trusted evaluator-only partition policy controls training, validation, and hidden-test membership.
- Candidate generation and selection can consume training observations and declared validation decisions only; hidden-test membership, metrics, failures, and per-episode feedback are unavailable.
- A frozen candidate may receive one final hidden-test assessment for release or adoption review. That assessment cannot feed further candidate generation or selection under the same experiment lineage.

### ADR-071 — Reuse benchmark journals for controlled paired experiments

- R6 orchestration resolves baseline and candidate matrices into the existing benchmark cell and attempt model.
- Improvement journals add only experiment-scoped transitions and references; they do not replace or edit benchmark journals.
- Pairing, resume, failures, non-comparability, statistical policy, and exact result verification remain authoritative.

### ADR-072 — Make recommendation state append-only and digest-bound

- Lifecycle events are append-only and project deterministically to current state.
- Allowed transitions are explicit; a candidate or control digest change appends `invalidated` and cannot retain `validated` or `adopted` state.
- History is never rewritten to conceal rejection, invalidation, or supersession.

### ADR-073 — Separate experimental validation, human approval, and adoption

- A passing controlled decision may produce `validated`; it cannot produce approval or adoption.
- A human decision binds exact recommendation, candidate, experiment, validation, policy, reviewer identifier, UTC time, and explicit confirmation.
- HuntEval never edits the active deployment. `adopted` records an explicitly confirmed external action against the exact approved digest.

### ADR-074 — Treat prompt recommendations as bounded hypotheses until controlled support

- A versioned reviewable taxonomy maps exact R5 classifications and observable source families to candidate prompt/configuration weaknesses.
- Mapping logic remains compiled typed Rust; taxonomy data contains no executable expressions.
- Suggested changes cite exact evidence and artifact sections. Observational traces alone cannot produce `validated`.

### ADR-075 — Keep suggested patches separate and non-authoritative

- A suggested-change artifact records typed operations against mutable sections, rationale, expected effects, risks, and required validation.
- Generation never writes into a registered baseline or deployment tree and never changes an active configuration.
- A candidate becomes testable only after explicit materialization into new bytes, registration under a new digest, full safety validation, and a new experiment manifest.

## 6. Contract and compatibility strategy

R6-00 freezes JSON Schemas and canonical examples before Rust implementation. The planned schema 0.8 set is:

- `registered-artifact.schema.json` — exact digest, kind, media type, schema, size, safe label, and provenance;
- `structured-artifact.schema.json` — bounded section inventory, section class, canonical order, content, and exact section hashes;
- `artifact-registry.schema.json` — deterministically ordered unique registrations and registry digest;
- `artifact-diff.schema.json` — baseline/candidate hashes, typed section operations, changed sections, changed variable, and immutable-policy result;
- `improvement-policy.schema.json` — immutable classes, permitted mutable targets, size/growth limits, leakage policy, regression constraints, verified-resource requirements, and human-review requirement;
- `improvement-experiment.schema.json` — baseline/candidate, partition-policy reference, paired matrix, declared changed variable, control hashes, scoring/statistical/improvement policies, and lineage;
- `improvement-equivalence-result.schema.json` — actual changes, controls, eligibility, reasons, and exact source hashes;
- `validation-decision.schema.json` — paired samples, raw metric deltas, uncertainty, constraints, missing cells, applicability, and decision;
- `recommendation.schema.json` — exact diagnostic evidence, attribution, suspected weakness, target artifact/section, suggested change, expected effects, validation reference, and lifecycle identity;
- `recommendation-event.schema.json` — append-only lifecycle transition and previous-event hash;
- `recommendation-state.schema.json` — deterministic projection with current state and invalidation status;
- `human-decision.schema.json` — explicit approve/reject decision over exact candidate and validation bytes;
- `adoption-record.schema.json` — explicit external-adoption confirmation for an approved exact digest;
- `prompt-failure-taxonomy.schema.json` — versioned prompt/configuration weakness definitions and source requirements;
- `improvement-report.schema.json` — normalized experiment, recommendation, validation, review, adoption, limitations, and exact sources;
- `improvement-bundle-manifest.schema.json` — bounded public inventory and hashes for offline verification.

Normative artifact kinds are expected to include `deployment_configuration`, `instruction`, `output_contract`, `tool_description`, `coordination_policy`, and `other_configuration`. Only schema-supported structured artifacts are diffable. Binary, executable, model-weight, secret-bearing, opaque, or unsupported artifacts remain registrable provenance but are ineligible as R6 candidate variables.

Compatibility is additive:

- schema 0.3 through 0.7 sources remain unchanged and valid without R6 artifacts;
- schema 0.7 hypotheses can seed a schema 0.8 recommendation only through an explicit adapter that resolves every cited source and retains `unvalidated` status;
- the narrow pre-R6 experiment prototype is not silently serialized as schema 0.8 or treated as validation evidence;
- old deployment prompt hashes remain valid provenance but are not structurally diffable without a new explicit structured-artifact registration;
- a schema 0.8 reader rejects unknown states and newer incompatible versions before interpreting payload fields;
- rollback disables schema 0.8 writers and orchestrators while retaining readers and never deletes emitted registries, journals, decisions, or review records.

### R6-00 — Contract freeze and architecture decisions

1. **Objective and outcome:** accept the schema version, artifact boundaries, lifecycle semantics, partition policy, and ADR decisions required before production implementation.
2. **Affected areas:** schema 0.8 README/contracts/examples, `ADR.md`, `CONTRACTS.md`, `METRICS_AND_RANKING.md`, `PROMPT_OPTIMIZATION.md`, `THREAT_MODEL.md`, and this plan.
3. **Public contracts and compatibility:** every schema listed above, common bounded definitions, canonical examples, meta-schema validation, and explicit schema 0.3–0.7 behavior.
4. **Security impact:** threat review covers immutable policy ownership, artifact parsing, answer leakage, hidden-test oracle risk, lifecycle forgery, review authority, and non-autonomous adoption.
5. **Ground-truth isolation:** schemas make evaluator-only partition policy and sealed final assessment impossible to serialize through candidate-visible or public recommendation fields.
6. **Positive tests:** every canonical example validates offline and every cross-schema reference resolves without network access.
7. **Negative and malformed tests:** unknown versions/fields/states, missing bounds, private fields, invalid digests, illegal lifecycle transitions, mutable safety classifications, hidden partition fields, and unbounded collections.
8. **Deterministic/replay tests:** canonical example serialization and lifecycle projection fixtures are byte-stable.
9. **Acceptance and gates:** all open questions in section 16 are resolved or explicitly deferred with safe unavailable behavior; schema/meta-schema, documentation, policy, secret-scan, and canonical gates pass.
10. **Documentation/ADR:** ADR-067 through ADR-075 are accepted and no accepted earlier ADR is weakened.
11. **Migration:** compatibility matrices state that no old artifact gains inferred structure, validation, approval, adoption, or partition membership.
12. **Rollback:** remove only unimplemented schema 0.8 writer registration while retaining the reviewed plan; no persisted earlier artifact changes.
13. **Known limitation:** R6-00 defines contracts and decisions only and provides no runtime R6 capability.

## 7. R6.1 — Artifact registry and safe diffs

### R6-01 — Bounded content-addressed artifact registry

1. **Objective and outcome:** register exact artifact bytes once and retrieve a deterministic inventory suitable for audit and experiment resolution.
2. **Affected areas:** schema 0.8 registry contracts/examples, domain artifact types, runner registry storage port and filesystem adapter, verification reason codes.
3. **Public contracts and compatibility:** `RegisteredArtifact`, `ArtifactKind`, `ArtifactMediaType`, `ArtifactRegistry`, `ArtifactId`, and digest-bound lookup; earlier prompt/configuration hashes remain unchanged provenance.
4. **Security impact:** bounded regular-file reads, no symlink or hard-link traversal, no secret values in labels/errors, deterministic redaction, and no executable interpretation.
5. **Ground-truth isolation:** public registry roots cannot include evaluator-private roots; hidden artifacts and private hashes are rejected from serializable registries.
6. **Positive tests:** supported JSON, YAML, and UTF-8 instruction artifacts; duplicate-byte deduplication; deterministic ordering; exact digest and size verification.
7. **Negative and malformed tests:** traversal, absolute paths, links, device files, oversized files, invalid UTF-8 where required, duplicate identities, hash mismatch, unknown kind/media type/schema, and concurrent registration.
8. **Deterministic/replay tests:** identical bytes produce byte-equivalent registry entries and projections across insertion orders.
9. **Acceptance and gates:** registry verification detects any byte change and exposes only safe relative labels plus hashes.
10. **Documentation/ADR:** contracts, threat model, artifact operations, ADR-067 and ADR-068 status.
11. **Migration:** existing hashes can be referenced but gain no fabricated structure.
12. **Rollback:** disable new registration while keeping read/verify support and existing bytes intact.
13. **Known limitation:** registry identity proves byte equality, not semantic correctness or artifact authorship.

### R6-02 — Structured baseline/candidate diff

1. **Objective and outcome:** produce a deterministic structural diff and prove exactly one declared experimental artifact variable changed.
2. **Affected areas:** structured-artifact and diff schemas, domain section/diff types, evaluation diff policy, runner artifact resolver.
3. **Public contracts and compatibility:** `StructuredArtifact`, `ArtifactSection`, `SectionClass`, `ArtifactDiff`, `DiffOperation`, and `ChangedVariable`; opaque legacy artifacts remain ineligible.
4. **Security impact:** strict parser bounds, deny unknown fields, canonical section identifiers, no arbitrary patch paths, and no contributor-defined executable diff rules.
5. **Ground-truth isolation:** section content and diff output are scanned for private identifiers before public serialization; private comparison material is never embedded.
6. **Positive tests:** add/replace/remove mutable section, unchanged semantic structure, section reorder according to declared canonical semantics, and one artifact change with multiple recorded section operations.
7. **Negative and malformed tests:** duplicate/overlapping sections, unsupported operation, changed artifact kind, ambiguous encoding, multiple artifact variables, unregistered hashes, stale bytes, and excessive diff size.
8. **Deterministic/replay tests:** canonical diff is byte-equivalent for identical inputs and stable across registry insertion order.
9. **Acceptance and gates:** every observed section change appears exactly once; undeclared or second-variable drift makes the comparison ineligible.
10. **Documentation/ADR:** structural format and ADR-068 rationale.
11. **Migration:** existing free-form instruction files require explicit new structured registration; no automatic heading inference.
12. **Rollback:** retain registry entries and omit structural eligibility/diff output.
13. **Known limitation:** semantic equivalence beyond declared structure is not inferred.

### R6-03 — Immutable policy and answer-leakage enforcement

1. **Objective and outcome:** reject unsafe candidates before any scored execution or selection feedback is produced.
2. **Affected areas:** improvement-policy schema, evaluation policy engine, runner trusted leakage-check adapter, redaction/secret-scan integration, threat model.
3. **Public contracts and compatibility:** `ImprovementPolicy`, `ImmutableSectionClass`, `CandidateSafetyResult`, `LeakageCheckResult`, and safe reason codes; no existing safety policy is weakened.
4. **Security impact:** candidate cannot define the allowlist, reclassify immutable content, request private reasoning, bypass managed tools, or alter authorization, tool, filesystem, network, data, benchmark, integrity, or security controls.
5. **Ground-truth isolation:** preselection scans disclose only safe pass/fail reasons; any final hidden-test integrity scan runs after candidate freeze and exposes no matched answer, identifier, episode, or location to candidate generation.
6. **Positive tests:** mutable planning, evidence, delegation, stopping, communication, and error-recovery changes that preserve all immutable hashes.
7. **Negative and malformed tests:** removed/renamed/reclassified immutable section, answer ID, expected conclusion, hidden episode reference, private path/hash, chain-of-thought request, direct tool execution, network enablement, provenance removal, and encoded/fragmented leakage fixtures.
8. **Deterministic/replay tests:** same policy and bytes produce identical ordered safety reasons and digest.
9. **Acceptance and gates:** unsafe or unverifiable candidates fail before matrix creation and no rejection diagnostic becomes a hidden-test oracle.
10. **Documentation/ADR:** ADR-069 and ADR-070, threat-model unsafe-improvement boundary, leakage operator guidance.
11. **Migration:** old three-category immutable hashes are accepted only as legacy evidence and cannot satisfy schema 0.8 coverage.
12. **Rollback:** refuse new candidate execution; never fall back to the narrower legacy policy.
13. **Known limitation:** leakage scanning is a defense-in-depth detector and cannot prove absence of semantic memorization.

### R6-04 — Controlled artifact equivalence and R6.1 closure

1. **Objective and outcome:** bind registry, diff, safety, and all non-experimental controls into one fail-closed eligibility result.
2. **Affected areas:** experiment/equivalence schemas, domain control hashes, evaluation equivalence reducer, runner resolution adapter, reporting source references.
3. **Public contracts and compatibility:** `ImprovementExperiment`, `ImprovementControlHashes`, `ImprovementEquivalenceResult`, eligibility and reason-code enums.
4. **Security impact:** exact hashes cover episode, seed set, budgets, models, tool policy, scoring profile, statistical policy, execution policy, topology, schemas, binaries, baseline, candidate, and improvement policy.
5. **Ground-truth isolation:** controls reference evaluator-private inputs only through trusted non-public identities where required; public output contains no private path, hash, or partition membership.
6. **Positive tests:** one instruction artifact change with every required control equal and every structural change declared.
7. **Negative and malformed tests:** model/budget/tool/topology/seed/schema/binary/policy drift, omitted control, extra changed variable, unsafe diff, stale registry, duplicate cells, and unsupported version.
8. **Deterministic/replay tests:** equivalence reduction and reason ordering are byte-stable from the same resolved inputs.
9. **Acceptance and gates:** no experiment can start unless the result is eligible and cites exact registry, diff, safety, and control artifacts.
10. **Documentation/ADR:** controlled-comparison semantics and R6.1 closure evidence requirements.
11. **Migration:** R4 topology experiments remain independent inputs and are referenced rather than converted.
12. **Rollback:** retain equivalence readers and prevent new R6 execution.
13. **Known limitation:** the result proves declared artifact/control equality, not broad causal transfer beyond the experiment.

## 8. R6.2 — Experiment orchestration

### R6-05 — Partition policy and hidden-test isolation

1. **Objective and outcome:** authorize training and validation use while making hidden-test membership and feedback unavailable to candidate generation and selection.
2. **Affected areas:** evaluator-only partition policy contract, runner partition authorization service, benchmark resolver, safe audit events.
3. **Public contracts and compatibility:** `PartitionPolicy`, `ExperimentPartition`, `SelectionAuthorization`, and sealed final-assessment reference; existing public episode classifications gain no hidden membership field.
4. **Security impact:** least-privilege service inputs, separate output capabilities, bounded requests, and no API returning hidden per-cell results during selection.
5. **Ground-truth isolation:** partition policy and hidden membership remain evaluator-only; public manifests, registries, reports, logs, and candidate contexts cannot serialize them.
6. **Positive tests:** training diagnosis, validation comparison, frozen-candidate final assessment, and authorized aggregate release decision.
7. **Negative and malformed tests:** hidden partition requested for generation/selection, membership enumeration, repeated oracle-like final checks, candidate changed after freeze, private serialization, and cross-lineage reuse.
8. **Deterministic/replay tests:** authorized public audit projection is stable and omits the same private fields on replay.
9. **Acceptance and gates:** candidate generation and selection processes cannot access hidden membership, metrics, failures, or episode-level feedback.
10. **Documentation/ADR:** ADR-070, partition operator procedure, and threat-model gaming controls.
11. **Migration:** existing benchmarks without a partition policy are ineligible for R6 selection, not silently treated as training.
12. **Rollback:** disable final assessment and selection authorization while preserving ordinary benchmark execution.
13. **Known limitation:** operational separation depends on trusted benchmark governance and cannot prevent knowledge acquired outside HuntEval.

### R6-06 — Paired matrix orchestration

1. **Objective and outcome:** execute baseline and candidate over exactly paired authorized cells through the existing benchmark service.
2. **Affected areas:** runner improvement service, benchmark manifest resolver, journal references, scheduling/resume adapters, CLI application port.
3. **Public contracts and compatibility:** `ImprovementRunPlan`, paired baseline/candidate cell references, attempt references, experiment journal events, and terminal state.
4. **Security impact:** the same sandbox, managed-tool mediation, budgets, process cleanup, output bounds, and denied network policy apply to both arms.
5. **Ground-truth isolation:** evaluator attaches truth only after each deployment boundary; candidate never receives baseline outputs, evaluator feedback, or private labels during a run.
6. **Positive tests:** complete paired matrix, bounded parallelism, interruption/resume, one failed arm, and exact attempt history.
7. **Negative and malformed tests:** ineligible equivalence, unpaired seed, candidate mutation after planning, journal tampering, concurrent controller, forged result digest, timeout, crash, and missing binary.
8. **Deterministic/replay tests:** identical verified results project to equivalent plan/state/decision inputs; retries append attempts without overwriting history.
9. **Acceptance and gates:** R6 orchestration delegates cell execution to the canonical runner and retains every missing, failed, and non-comparable arm.
10. **Documentation/ADR:** ADR-071 and CLI/operator flow.
11. **Migration:** ordinary R4/R5 benchmark journals remain valid and are referenced read-only.
12. **Rollback:** stop new improvement scheduling; individual run and benchmark artifacts remain independently verifiable.
13. **Known limitation:** stochastic deployment byte identity is not guaranteed; reproducibility is evaluated through declared repetitions.

### R6-07 — Controlled validation decision

1. **Objective and outcome:** decide whether the exact candidate satisfies declared quality, regression, resilience, resource, verified-cost, uncertainty, and comparability constraints.
2. **Affected areas:** validation/improvement-policy schemas, evaluation decision reducer, statistics integration, domain applicability and constraint types.
3. **Public contracts and compatibility:** `ValidationDecision`, `ValidationStatus`, `CandidateConstraint`, metric deltas, intervals, paired counts, violations, unverifiable constraints, and source references.
4. **Security impact:** policies are content-addressed, finite, bounded, and cannot be authored by candidate output or override immutable sections.
5. **Ground-truth isolation:** decision consumes normalized evaluator results; public citations expose metric/cell identities but not private truth or hidden-test detail.
6. **Positive tests:** improved candidate, allowed bounded regression, verified-cost constraint, resilience constraint, statistically descriptive pass policy, and explicit aggregate-score omission.
7. **Negative and malformed tests:** missing/failed pairs, insufficient samples, non-finite values, unknown metric/version, mismatched direction/range, self-reported cost under verified constraint, disqualifying violation, and multiplicity guard.
8. **Deterministic/replay tests:** cell order does not affect normalized decision bytes; same policies and verified inputs reproduce the same result.
9. **Acceptance and gates:** only `passed` can support `validated`; unavailable/unverifiable controls never become satisfied; raw metric vectors remain authoritative.
10. **Documentation/ADR:** metrics/ranking decision hierarchy, constraint behavior, and limitations.
11. **Migration:** the narrow in-memory prototype may be removed after equivalent tests move to schema 0.8; it produces no persisted migration claim.
12. **Rollback:** keep decision readers and mark new experiments unsupported; never reinterpret a previous failure as pass.
13. **Known limitation:** passing establishes bounded experimental support under declared controls, not universal superiority.

### R6-08 — Experiment CLI, verification, and R6.2 closure

1. **Objective and outcome:** expose offline validate/run/resume/status/verify commands without putting policy in argument parsing.
2. **Affected areas:** CLI args/controllers, runner improvement service/verifier, benchmark CLI documentation, completion/help snapshots.
3. **Public contracts and compatibility:** commands equivalent to `improvement validate`, `improvement run`, `improvement resume`, `improvement status`, and `improvement verify`; JSON status and stable exit codes.
4. **Security impact:** safe relative inputs/outputs, existing-target refusal, bounded diagnostics, no secrets, no network expansion, and no hidden-test selection option.
5. **Ground-truth isolation:** CLI cannot accept or print private ground-truth paths or hidden partition selectors; trusted policy configuration remains outside candidate-visible output.
6. **Positive tests:** parse, dry validation, complete run, resume, machine-readable status, and offline verification.
7. **Negative and malformed tests:** traversal, links, overwrite, unsupported schema, stale candidate, invalid lifecycle, hidden selection request, tampered bundle, and stdout/stderr separation.
8. **Deterministic/replay tests:** status and verification are byte-stable for identical journals and artifacts.
9. **Acceptance and gates:** verification detects every referenced-byte change and no command can adopt or edit an artifact.
10. **Documentation/ADR:** command reference, exit codes, operator examples, and R6.2 closure checklist.
11. **Migration:** existing CLI commands and exit codes remain unchanged.
12. **Rollback:** remove additive commands while leaving bundles verifiable through library readers.
13. **Known limitation:** CLI orchestration is local; distributed scheduling remains deferred.

## 9. R6.3 — Recommendation lifecycle

### R6-09 — Append-only lifecycle state machine

1. **Objective and outcome:** record every recommendation transition without overwriting proposal, testing, validation, rejection, or invalidation history.
2. **Affected areas:** recommendation event/state schemas, domain transition reducer, runner journal storage/projection, verification.
3. **Public contracts and compatibility:** `RecommendationEvent`, `RecommendationState`, `RecommendationStatus`, sequence, previous digest, caused-by reference, and typed transition reasons.
4. **Security impact:** bounded hash chain, single-writer lock, atomic projection, no free-form executable state, and strict transition allowlist.
5. **Ground-truth isolation:** events cite public/evaluator-safe artifacts only; hidden results appear only through an authorized sealed decision reference.
6. **Positive tests:** proposed to testing to validated, proposed/testing to rejected, validated to invalidated, supersession, and resumed projection.
7. **Negative and malformed tests:** skipped transition, validated without pass, adopted without human decision, duplicate sequence, altered history, future causal reference, stale digest, and unknown state.
8. **Deterministic/replay tests:** replay produces byte-equivalent state; insertion/retry cannot rewrite terminal facts.
9. **Acceptance and gates:** current state is a projection, history is authoritative, and observational evidence alone cannot emit validated.
10. **Documentation/ADR:** ADR-072 and lifecycle diagram.
11. **Migration:** schema 0.7 unvalidated hypotheses adapt only to a new `proposed` event with exact resolved evidence.
12. **Rollback:** stop appending new events and retain reader/replay support.
13. **Known limitation:** append-only integrity is local content integrity, not a cryptographic human signature.

### R6-10 — Human review and external adoption records

1. **Objective and outcome:** require a recorded explicit human decision before an exact validated candidate can be marked adopted.
2. **Affected areas:** human-decision/adoption schemas, domain types, runner review-record service, CLI confirmation flow, verifier.
3. **Public contracts and compatibility:** `HumanDecision`, `ReviewDecision`, `AdoptionRecord`, reviewer ID, UTC timestamp, exact recommendation/candidate/experiment/validation/policy hashes, and explicit confirmation flags.
4. **Security impact:** no self-generated approval, no implicit approval from command execution, no candidate-controlled reviewer fields, and no deployment write capability.
5. **Ground-truth isolation:** review artifacts contain bounded decision reasons and sealed assessment references, never hidden episode results or answer material.
6. **Positive tests:** approve exact validated candidate, reject candidate, record separately confirmed external adoption, and independently verify hashes.
7. **Negative and malformed tests:** approval without validation, stale candidate, failed decision, missing confirmation, candidate-authored review, timestamp/ID errors, duplicate conflicting decisions, and adoption without approval.
8. **Deterministic/replay tests:** decision identity and lifecycle projection are stable from exact inputs.
9. **Acceptance and gates:** HuntEval cannot reach `adopted` without a passing validation, explicit approval, and explicit external-adoption confirmation; no command modifies deployment bytes.
10. **Documentation/ADR:** ADR-073, reviewer responsibilities, and solo/independent review limitations.
11. **Migration:** no legacy recommendation is approved or adopted automatically.
12. **Rollback:** retain decisions and remove the adoption-record command; active deployment state remains external.
13. **Known limitation:** HuntEval records a human assertion but does not authenticate real-world deployment rollout without a future signing integration.

### R6-11 — Invalidation, lifecycle reports, and R6.3 closure

1. **Objective and outcome:** invalidate stale validation automatically and render a complete auditable lifecycle without stage confusion.
2. **Affected areas:** runner invalidation service, reporting improvement modules, normalized JSON/static HTML, bundle verifier, CLI status.
3. **Public contracts and compatibility:** invalidation causes for candidate, baseline, experiment, policies, schemas, controls, and binaries; lifecycle report sections and exact artifact inventory.
4. **Security impact:** render structured cited fields only, escape all untrusted text, prohibit scripts/handlers, and fail closed on stale or missing sources.
5. **Ground-truth isolation:** reports show authorized aggregate decisions and limitations, not hidden membership, per-episode hidden feedback, private paths, or private hashes.
6. **Positive tests:** valid lifecycle, rejected candidate, invalidated validation, approved but not adopted, and externally adopted candidate.
7. **Negative and malformed tests:** changed candidate retaining validated state, fake approval, missing source, private reference, active HTML, hostile rationale, oversized lifecycle, and JSON/HTML stage mismatch.
8. **Deterministic/replay tests:** JSON is authoritative and HTML is a byte-stable projection for fixed inputs.
9. **Acceptance and gates:** any bound-byte change invalidates prior validation; reports visibly separate proposed, testing, validated, rejected, approved, adopted, and invalidated facts.
10. **Documentation/ADR:** lifecycle reporting, invalidation matrix, and R6.3 closure evidence.
11. **Migration:** R5 diagnostic reports remain valid and may link to, but are never rewritten as, R6 lifecycle reports.
12. **Rollback:** omit additive lifecycle rendering while retaining journals and verification.
13. **Known limitation:** report availability does not make a candidate active in an external deployment.

## 10. R6.4 — Prompt improvement analysis

### R6-12 — Prompt/configuration failure taxonomy and compiled mapping registry

1. **Objective and outcome:** establish a versioned reviewable weakness taxonomy whose mappings are executable only through typed compiled rules.
2. **Affected areas:** schema 0.8 taxonomy/example, `taxonomies` artifact, domain weakness types, evaluation mapping registry, R5 classification adapter.
3. **Public contracts and compatibility:** `PromptFailureTaxonomy`, `PromptWeaknessDefinition`, required diagnostic codes/source families/target kinds, taxonomy digest, and registry digest.
4. **Security impact:** taxonomy is bounded metadata with no scripts, expressions, templates, filesystem paths, instructions, or tool authority.
5. **Ground-truth isolation:** mappings consume only exact observable R5 sources and registered artifact structure; private truth and hidden results are prohibited.
6. **Positive tests:** categories for role ambiguity, missing output contract, evidence requirements, acceptance criteria, stopping condition, tool-use policy, error handling, delegation, duplicated responsibility, task ownership, conflict resolution, excessive communication, evidence sharing, and specialist invocation.
7. **Negative and malformed tests:** duplicate/unknown code, registry drift, unsupported source, executable payload, private reference, missing requirement, oversized description, and unknown field.
8. **Deterministic/replay tests:** taxonomy/registry digests and mapping order are stable from exact bytes.
9. **Acceptance and gates:** no weakness hypothesis exists without a registered typed rule and its complete exact evidence set.
10. **Documentation/ADR:** ADR-074, taxonomy review process, and compatibility policy.
11. **Migration:** R5 failure taxonomy remains unchanged; schema 0.8 references its exact code and digest.
12. **Rollback:** remove the new mapping registry from writers while preserving taxonomy readers.
13. **Known limitation:** taxonomy coverage is finite and absence of a match does not prove prompt quality.

### R6-13 — Artifact inspection and bounded weakness hypotheses

1. **Objective and outcome:** inspect registered structured sections and formulate a bounded candidate weakness only when exact diagnosis and artifact evidence support it.
2. **Affected areas:** evaluation prompt-analysis modules, runner artifact/source resolver, domain recommendation types, diagnostic adapter.
3. **Public contracts and compatibility:** attribution, hypothesis, target artifact/section, evidence sufficiency, limitations, and distinction among observation, classification, attribution, and hypothesis.
4. **Security impact:** no model call is required, no prose-derived hidden reasoning, no arbitrary content execution, and section content is treated as untrusted.
5. **Ground-truth isolation:** inspection receives no evaluator truth, hidden partition result, private path/hash, or reference answer; metric citations remain evaluator-safe normalized references.
6. **Positive tests:** duplicate task to missing ownership, unsupported finding to missing evidence rule, tool error to insufficient recovery, unresolved conflict to missing resolution policy, and excessive messages to communication constraint.
7. **Negative and malformed tests:** observational proximity only, role-label guess, absent target artifact, immutable target, ambiguous section, unsupported taxonomy mapping, stale source, and malicious instruction text.
8. **Deterministic/replay tests:** same resolved sources and artifact bytes produce the same hypothesis ID, target, rationale code, and limitations.
9. **Acceptance and gates:** recommendations cite exact affected runs/events/tasks/actions/findings/coordination events/metrics as applicable and never claim prompt causality.
10. **Documentation/ADR:** conceptual workflow and exact stage semantics.
11. **Migration:** schema 0.7 recommendation hypotheses may seed analysis but cannot become experimentally supported without schema 0.8 validation.
12. **Rollback:** retain R5 diagnosis and omit prompt weakness hypotheses.
13. **Known limitation:** inspection can identify a plausible missing or conflicting rule, not the deployment's hidden reasoning process.

### R6-14 — Safe suggested-change artifact

1. **Objective and outcome:** optionally emit a structured candidate suggestion against mutable sections without changing baseline or active deployment files.
2. **Affected areas:** recommendation/suggested-change schemas, evaluation suggestion policy, runner output writer, CLI proposal command.
3. **Public contracts and compatibility:** typed `add_section`, `replace_section`, `remove_section`, or `add_constraint` operations; rationale, expected effects, trade-offs, validation requirement, and target hashes.
4. **Security impact:** immutable targets and unrestricted raw patches are impossible to represent; generated text is bounded untrusted data and cannot add authority.
5. **Ground-truth isolation:** suggestions cannot contain answer identifiers, hidden episode material, private references, or evaluator-only feedback.
6. **Positive tests:** add task ownership, clarify evidence-sharing, add stopping condition, and strengthen mutable output acceptance criteria.
7. **Negative and malformed tests:** immutable target, multiple undeclared artifacts, answer leakage, policy weakening, private reasoning request, direct tool authority, excessive growth, output-contract removal, and stale baseline.
8. **Deterministic/replay tests:** rule-based suggestion artifact and diff preview are byte-stable for fixed inputs.
9. **Acceptance and gates:** output status is `proposed`, `validation_required` is true, no registered bytes change, and materialization requires a separate explicit action followed by registration and safety validation.
10. **Documentation/ADR:** ADR-075 and user-visible proposal/materialization boundary.
11. **Migration:** free-form R5 recommendation text remains historical and is not parsed into patch operations.
12. **Rollback:** disable suggestion writing without affecting diagnosis or registered artifacts.
13. **Known limitation:** initial suggestions are deterministic templates/rules; provider-driven generation remains outside this plan and would require a separate ADR and threat review.

### R6-15 — Controlled A/B linkage, reporting, and R6.4 closure

1. **Objective and outcome:** connect exact evidence, suggestion, materialized candidate, controlled experiment, validation, and human review into one auditable recommendation.
2. **Affected areas:** runner recommendation service, reporting improvement components, CLI compare/status/verify output, use-case documentation.
3. **Public contracts and compatibility:** exact baseline/candidate hashes, artifact diff, experiment/equivalence/decision references, lifecycle state, expected and observed effects, limitations, and content-addressed bundle.
4. **Security impact:** no observational-only validation, no hidden-test selection feedback, no immutable change, no automatic adoption, and every source is verified before rendering.
5. **Ground-truth isolation:** bundle contains only authorized observable artifacts and normalized decisions; evaluator-only partition and truth artifacts are excluded.
6. **Positive tests:** proposed recommendation, controlled pass, controlled rejection, regression failure, human approval, external adoption record, and changed-candidate invalidation.
7. **Negative and malformed tests:** fake experiment, ineligible controls, missing pairs, stale diff, hidden feedback source, immutable mutation, approval without pass, universal causal wording, and artifact tampering.
8. **Deterministic/replay tests:** complete JSON/bundle/HTML regeneration and offline verification are byte-equivalent for fixed inputs.
9. **Acceptance and gates:** every recommendation stage cites exact evidence; only a passing controlled experiment can support validated; explicit human approval is required before externally confirmed adoption.
10. **Documentation/ADR:** end-to-end use case, operator review, limitations, and R6.4 closure evidence.
11. **Migration:** R5 bundles remain standalone and may be referenced by digest without rewriting.
12. **Rollback:** omit R6 recommendation projection while preserving diagnosis, experiments, decisions, and lifecycle history.
13. **Known limitation:** evidence-backed support is specific to the declared benchmark, topology, models, policies, budgets, partitions, and candidate bytes.

## 11. Integration and closure

### R6-16 — End-to-end controlled-improvement and CI gate

1. **Objective and outcome:** prove registration, safe diff, leakage rejection, paired execution, validation, lifecycle, human review, invalidation, prompt analysis, reporting, and offline verification on reproducible artifacts.
2. **Affected areas:** `scripts/ci/r6-improvement.sh`, E2E artifact manifest, GitHub Actions job, canonical fixtures, malformed corpus, secret scan.
3. **Public contracts and compatibility:** bounded CI inventory, exact hashes, verifier result, stage inventory, private-data scan, and unsupported-claim inventory.
4. **Security impact:** least-privilege workflow, bounded retention, no write token, no secrets, no hidden-test detail, and existing sandbox enforcement.
5. **Ground-truth isolation:** include canary and serialization tests proving generation/selection outputs contain no truth, hidden membership, hidden metrics, or private hashes.
6. **Positive tests:** one deterministic mutable instruction candidate across paired training/validation cells, passing decision, explicit review record, and separate external-adoption fixture.
7. **Negative and malformed tests:** immutable change, answer leakage, hidden selection, unpaired cells, regression, unverified cost, stale candidate, fake approval, candidate change after validation, and hostile HTML.
8. **Deterministic/replay tests:** clean repeated execution produces byte-equivalent schema, registry, diff, experiment, decision, lifecycle, report, and bundle artifacts.
9. **Acceptance and gates:** all positive artifacts verify; every negative fails with a stable typed reason; CI publishes only bounded non-secret evidence.
10. **Documentation/ADR:** CI operation and milestone evidence format.
11. **Migration:** canonical older schema examples and gates continue passing unchanged.
12. **Rollback:** remove only the new CI job after reverting its owning implementation; no existing gate is weakened.
13. **Known limitation:** CI uses deterministic local reference deployments and does not validate external provider behavior.

### R6-17 — R6 release closure

1. **Objective and outcome:** prove all R6 exit criteria on one revision and record exact evidence without changing R2–R5 history.
2. **Affected areas:** README, roadmap, R6 plan status, completion-evidence document, release checklist, schema/package inventory, CI evidence.
3. **Public contracts and compatibility:** exact revisions and hashes for schemas, policies, taxonomies, registries, diffs, experiments, decisions, lifecycle, review/adoption records, reports, bundles, and relevant binaries.
4. **Security impact:** full security, leakage, redaction, secret-scan, sandbox, and no-autonomous-adoption evidence.
5. **Ground-truth isolation:** closure records only safe aggregate evidence and explicitly verifies hidden-test non-disclosure.
6. **Positive tests:** all focused tests, canonical gates, deterministic R6 gate, clean-tree E2E, and non-publishing release-candidate dry run.
7. **Negative and malformed tests:** retained R6 corpus plus gate failure-propagation proof.
8. **Deterministic/replay tests:** exact clean regeneration and offline bundle verification hashes are recorded.
9. **Acceptance and gates:** every R6.1–R6.4 exit criterion passes locally and remotely on the evidence revision.
10. **Documentation/ADR:** accepted ADR status, completion evidence, operator/use-case docs, migration, rollback, and limitations.
11. **Migration:** earlier completion evidence and artifacts remain byte-unchanged.
12. **Rollback:** no production release is published; a remote failure returns R6 to active status while readers remain available.
13. **Known limitation:** autonomous prompt adoption, production SIEM scored execution, distributed orchestration, and provider-driven suggestion generation remain out of scope.

## 12. Milestone dependency graph

```text
R5 complete
  -> R6-00 contracts and ADR decisions

R6-00
  -> R6-01 artifact registry
       -> R6-02 structural diff
            -> R6-03 immutable/leakage policy
                 -> R6-04 equivalence and R6.1 closure

R6-04
  -> R6-05 partition isolation
       -> R6-06 paired orchestration
            -> R6-07 validation decision
                 -> R6-08 CLI/verification and R6.2 closure

R6-08
  -> R6-09 lifecycle journal
       -> R6-10 human decision/adoption record
            -> R6-11 invalidation/reporting and R6.3 closure

R5 diagnosis + R6-01 + R6-03
  -> R6-12 prompt/configuration taxonomy
       -> R6-13 artifact inspection/hypothesis
            -> R6-14 suggested-change artifact

R6-08 + R6-11 + R6-14
  -> R6-15 controlled A/B recommendation and R6.4 closure
       -> R6-16 end-to-end/CI
            -> R6-17 release closure
                 -> R7 knowledge and extension planning
```

R6.1 is the critical path. Taxonomy work may begin after the artifact and safety contracts stabilize, but R6.4 cannot close before controlled orchestration, lifecycle, invalidation, and human decision are complete. No milestone may create a parallel benchmark, statistics, scoring, diagnosis, or verification system.

## 13. Milestone handoff checklist

Before completing any R6 milestone:

1. the objective and user-visible outcome are implemented without unrelated R7 or post-v1.0 scope;
2. every affected contract has a schema, canonical example, bounds, validation, and compatibility coverage;
3. security and ground-truth-isolation effects are documented and negatively tested;
4. positive, negative, malformed-input, deterministic/replay, stale-artifact, leakage, hidden-test, and resource-bound tests pass;
5. every artifact, diff, experiment, decision, lifecycle event, and recommendation resolves exact content-addressed sources;
6. exactly one experimental artifact variable changes and every structural change is recorded;
7. immutable policy and answer-leakage checks pass before candidate execution;
8. raw metrics, uncertainty, missing cells, resource provenance, and constraints remain explicit without imputation;
9. no observational evidence can produce validation, approval, adoption, or universal causal wording;
10. first-party production code contains no unsafe, panic shortcut, unbounded input, private leakage, provider coupling, or artifact mutation side effect;
11. source files remain cohesive and within repository size policy;
12. exact focused commands and all canonical gates pass;
13. documentation, ADR status, migration, rollback, limitations, and `git diff --check` are current;
14. no private, generated, secret-bearing, or unrelated artifact is tracked;
15. a descriptive commit exists before status changes to `complete`, with remote evidence required for release closure.

Remote failure returns the milestone to active status until the same revision passes locally and remotely.

## 14. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| opaque artifacts receive structural claims | false control equivalence | require schema-supported section inventory; opaque values remain provenance-only |
| candidate reclassifies a safety section | policy weakening | immutable registry is external, versioned, hashed, and candidate-inaccessible |
| leakage checker becomes a hidden-answer oracle | benchmark compromise | safe reason codes, no match detail, candidate freeze, one final sealed assessment |
| diff misses a second variable | invalid causal comparison | exact registry inventory plus structural and control-hash equivalence |
| R6 creates a second runner | inconsistent recovery/evidence | reuse benchmark cells, attempts, journal, sandbox, verification, and statistics |
| missing cells are imputed | biased validation | retain failed/non-comparable pairs and explicit applicability |
| aggregate gain hides regression | unsafe adoption | constraint-first decision, raw vector authoritative, required regression limits |
| self-reported cost satisfies a hard cap | unverifiable decision | require exact measured or verified-adapter provenance per metric contract |
| observational diagnosis becomes validated | unsupported causality | lifecycle transition requires passing controlled decision digest |
| validation survives candidate mutation | stale approval | digest-bound invalidation event and verifier rejection |
| reviewer action is inferred | unauthorized adoption | explicit confirmation artifact; no implicit command-side approval |
| adoption command edits deployment | autonomous modification | record external assertion only; no deployment write port in service |
| taxonomy data executes logic | injection/nondeterminism | metadata-only schema and compiled typed registry equality |
| suggestion targets immutable policy | security boundary change | typed mutable target enum and fail-closed diff/safety validation |
| generated text leaks private data | benchmark/secret disclosure | exact source allowlist, leakage scan, redaction, secret scan, bundle verification |
| R6 report overstates transfer | misleading recommendation | bind exact topology/models/policies/partitions and preserve limitations |
| schema 0.8 breaks old artifacts | lost reproducibility | additive readers, explicit adapters, writer-only rollback, immutable fixtures |
| modules become unmaintainable | review and security defects | cohesive submodules, 300-line review threshold, 500-line hard policy |

## 15. R6 completion definition

R6 is complete only when:

1. schema 0.8 contracts are bounded, versioned, content-addressed, deny unknown fields, and preserve schemas 0.3 through 0.7;
2. supported deployment configuration and instruction artifacts register by exact bytes and are structurally diffable;
3. exactly one declared experimental artifact variable changes and every changed section is recorded;
4. immutable safety and trust-boundary sections cannot be modified, removed, renamed, reclassified, or bypassed;
5. candidate leakage checks reject known answer material without exposing hidden-test feedback or acting as a selection oracle;
6. baseline/candidate matrices preserve episode, seed, budgets, models, topology, tool policy, scoring, statistics, execution policy, schemas, and binaries unless explicitly experimental;
7. paired orchestration uses the existing sandboxed benchmark runner, records all attempts, and resumes without rewriting history;
8. decisions retain raw metric vectors, uncertainty, missing/non-comparable cells, explicit constraints, and verified resource provenance;
9. no unavailable or unverifiable metric is silently imputed or converted to zero;
10. recommendations distinguish observation, classification, attribution, hypothesis, suggested change, experimental support, human approval, adoption, rejection, and invalidation;
11. observational evidence alone cannot mark a recommendation validated;
12. changing any bound candidate or control artifact invalidates prior validation and downstream approval/adoption eligibility;
13. adoption requires a passing controlled experiment, explicit human approval, and explicit external-adoption confirmation;
14. HuntEval never modifies or adopts deployment instructions autonomously;
15. the prompt/configuration taxonomy covers every roadmap-required weakness category and exactly matches compiled mapping rules;
16. every prompt recommendation cites exact observable sources, baseline/candidate hashes, structural diff, and validation manifest where applicable;
17. normalized JSON is authoritative, static HTML is escaped and script-free, and every displayed claim verifies offline;
18. public artifacts contain no private truth, hidden partition membership or feedback, private paths/hashes, secrets, or private chain of thought;
19. complete quality, security, adversarial, benchmark-science, diagnosis, improvement, end-to-end, documentation, and package gates pass locally and in GitHub Actions on the evidence revision.

Completion evidence must record exact commands, revisions, toolchain, schema/policy/taxonomy/registry hashes, baseline and candidate hashes, diff/equivalence/experiment/decision hashes, paired sample accounting, lifecycle and human-decision hashes, report/bundle/verifier hashes, secret-scan results, known limitations, and ADR status changes.

## 16. R6-00 architecture resolutions

R6-00 resolves the contract questions as follows:

1. A schema 0.8 structured artifact is JSON with an explicit ordered section inventory, stable identifiers, policy-owned classes, exact content, and section hashes. HuntEval does not infer Markdown structure.
2. The initial mutable classes are task planning, evidence requirements, delegation strategy, stopping conditions, communication format, error recovery, and output contract. A versioned policy explicitly allows operations; removal is eligible only for a mutable class when the policy allows it and the resulting artifact remains structurally valid.
3. Instruction, deployment-configuration, output-contract, tool-description, and coordination-policy artifacts may become the single variable after explicit structure and registration. `other_configuration` remains provenance-only until a later contract assigns safe semantics.
4. One final assessment is authorized for one frozen candidate digest and lineage. Its membership and episode-level results remain evaluator-only, and it cannot authorize another candidate in that lineage.
5. Every improvement policy requires at least one minimum-quality constraint and one maximum-regression constraint. Resource, resilience, and verified-cost constraints are selected explicitly where applicable; immutable safety, leakage, partition isolation, and human review are unconditional gates.
6. A versioned policy may accept a descriptive result only when conservative constraints pass. The report must retain descriptive claim strength; `validated` never becomes a conclusive or transferable claim by implication.
7. Schema 0.8 records a bounded reviewer identifier, UTC time, exact hashes, and explicit confirmation. This is an auditable assertion rather than a cryptographic signature; signing remains future work.
8. The canonical weakness taxonomy records initial R5 diagnostic-code and source-family requirements for all fourteen roadmap categories. Executable agreement is deferred to the compiled registry in R6-12.
9. Deterministic suggestions may use `add_section`, `replace_section`, `remove_section`, or `add_constraint` only when both the improvement policy and weakness definition allow the operation.
10. The narrow in-memory experiment API remains legacy-only until R6-07 migrates its focused tests, then it is removed rather than serialized or treated as schema 0.8 evidence.

Any later change to these trust boundaries, persisted contracts, hidden-test policy, or adoption semantics requires a new ADR.

## 17. Proposed ADR updates

R6-00 adds ADR-067 through ADR-075 as accepted contract decisions. Owning implementation milestones must preserve them and add their runtime tests:

- R6-00: ADR-067 contract version and compatibility;
- R6-01/R6-02: ADR-068 registration and structural diff;
- R6-03: ADR-069 immutable policy and ADR-070 hidden-test separation;
- R6-06: ADR-071 benchmark journal reuse;
- R6-09/R6-11: ADR-072 append-only lifecycle and invalidation;
- R6-10: ADR-073 validation/review/adoption separation;
- R6-12/R6-13: ADR-074 evidence-backed prompt analysis;
- R6-14: ADR-075 non-authoritative suggestions.

No accepted ADR-001 through ADR-066 is reopened or weakened.

## 18. Exact acceptance command inventory

Existing commands remain mandatory after every milestone:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/e2e.sh
git diff --check
```

Owning milestones add focused commands equivalent to:

```bash
cargo test -p hunteval-domain --test schema_v08 --test improvement_v08
cargo test -p hunteval-evaluation --test artifact_diff
cargo test -p hunteval-evaluation --test candidate_safety
cargo test -p hunteval-evaluation --test improvement_equivalence
cargo test -p hunteval-evaluation --test validation_decision
cargo test -p hunteval-evaluation --test prompt_improvement
cargo test -p hunteval-runner --test artifact_registry
cargo test -p hunteval-runner --test partition_isolation
cargo test -p hunteval-runner --test improvement_service
cargo test -p hunteval-runner --test recommendation_lifecycle
cargo test -p hunteval-runner --test improvement_verification
cargo test -p hunteval-reporting --test improvement_reporting
cargo test -p hunteval-cli --test improvement
```

R6-16 must add:

```bash
./scripts/ci/r6-improvement.sh
```

R6-17 must additionally run the non-publishing release-candidate procedure from `RELEASE_CHECKLIST.md` on a clean tree and require every canonical GitHub Actions job on the same evidence revision. Exact target names may change only in the milestone that creates them and must be updated here in the same change. No milestone is complete while a required local or remote gate fails.

## 19. Known limitations retained through R6

- R6 validates exact registered candidates under declared controls; it does not prove transfer to different topologies, models, datasets, policies, or production environments.
- Artifact structure is explicit and versioned; opaque legacy instruction/configuration bytes are not structurally compared by inference.
- Leakage checks reduce known benchmark-answer risk but cannot prove absence of semantic memorization learned outside HuntEval.
- Prompt/configuration weakness mappings are deterministic and finite; absence of a recommendation does not prove absence of a weakness.
- Initial suggested changes are bounded deterministic templates or operations. Provider-driven candidate generation is not introduced.
- Human decisions and adoption records are content-addressed assertions, not cryptographic signatures or deployment automation.
- `adopted` records an explicitly confirmed external action; HuntEval does not modify the active deployment.
- Hidden-test results remain unavailable during generation and selection; final assessment governance depends on trusted local operators.
- Verified provider cost remains unavailable without a verifiable adapter.
- No autonomous prompt adoption, production SIEM connector, unrestricted network access, distributed execution, web dashboard, Kubernetes deployment, or private chain-of-thought collection is introduced.
