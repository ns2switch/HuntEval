# R7 implementation plan

## 1. Purpose and scope

This document turns roadmap initiatives R7.1 through R7.3 into a reviewable implementation sequence. R7 makes verified HuntEval artifacts locally searchable, defines stable framework-neutral extension boundaries, and provides a typed Python SDK for authored manifests, protocol messages, and public result artifacts. It extends the completed R2 through R6 platform without moving orchestration authority, scored-tool execution, ground truth, or immutable safety policy outside HuntEval.

R7 covers:

- deterministic local indexing of authorized, verified public run, benchmark, report, diagnosis, topology, and controlled-improvement artifacts;
- structured local queries whose results cite exact artifact, run, event, metric, comparison, and document identities;
- explicit separation between evaluator-only analytical corpora and knowledge that may be exposed to an evaluated deployment;
- versioned out-of-process managed-tool and deployment-adapter manifests, capability declarations, and conformance results;
- compatibility fixtures for supported protocol and schema versions;
- a typed, schema-aligned Python SDK for constructing authored inputs and reading verified public outputs;
- cross-language canonical fixtures and packaging checks.

R7 does not add a production SIEM connector, unrestricted network access, in-process dynamic plugins, a provider SDK dependency, a web service, distributed storage, private-chain-of-thought collection, autonomous prompt optimization, or autonomous adoption. It does not permit an SDK or adapter to invoke scored tools directly. R2 through R6 remain complete with their recorded evidence, and their schemas, protocol behavior, security boundaries, release state, and completion evidence remain unchanged.

### Delivery status

Status is evidence-based. `planned` makes no implementation claim. `in progress` means only part of the milestone behavior and tests exists. `implemented` means the local milestone behavior and focused tests exist but release evidence is incomplete. `complete` requires a dedicated commit, focused acceptance tests, all canonical gates, documentation evidence, and passing GitHub Actions on the exact evidence revision.

| Milestone | Status | Outcome or dependency |
|---|---|---|
| R7-00 | implemented | schema 0.9 contracts, canonical examples, compatibility policy, and ADR-076 through ADR-083 |
| R7-01 | in progress | content-addressed corpus and hardened source loading exist; per-source authoritative verifier evidence remains to be bound |
| R7-02 | implemented | deterministic index build over runner-verified public artifact bytes |
| R7-03 | in progress | bounded queries and source/hash/field citations exist; deeper typed run/event/metric adapters remain |
| R7-04 | implemented | hash-linked retrieval audit, provenance, validation/build/query/verify CLI |
| R7-05 | in progress | focused isolation, tamper, private-field, and representative R2-R6 corpus tests exist; R7.1 closure remains |
| R7-06 | implemented | versioned managed-tool adapter manifest contract |
| R7-07 | implemented | versioned deployment-adapter manifest contract |
| R7-08 | implemented | deny-by-default capability, denied-network, limit, and digest enforcement |
| R7-09 | implemented | extension fixtures plus supervised deployment and one-shot managed-tool conformance with timeout, crash, malformed-output, and transcript checks |
| R7-10 | implemented | schema-aligned typed Python models and bounded validation errors |
| R7-11 | in progress | corpus/extension builders and content-addressed opaque public readers exist; roadmap-wide typed readers remain |
| R7-12 | implemented | bounded deployment-protocol peer with registration/terminal enforcement |
| R7-13 | implemented | Rust/Python fixture agreement and reproducible wheel-content dry run |
| R7-14 | in progress | dedicated local gate and `Knowledge and extensions` job exist; the complete end-to-end adapter workflow remains |
| R7-15 | planned | documentation, migration, rollback, and R7 release closure |

The R7/v0.7 release name is independent from persisted schema versions. R7-00 selects additive schema `0.9` through accepted ADR-076 and freezes the canonical fixture inventory. R7 remains incomplete until R7-15 records exact local, package, protected-branch, and remote evidence.

## 2. Baseline audit

The repository already provides useful but narrower foundations:

1. `hunteval-knowledge` implements deterministic, network-free token-overlap retrieval over author-provided UTF-8 documents, with safe relative paths, symlink rejection, document and token limits, citations, and an untrusted-content marker.
2. The runner keeps knowledge retrieval optional and disabled by default, records retrieval results in memory, and has negative coverage for retrieval-disabled execution and malicious document instructions.
3. Public run, benchmark, topology, diagnosis, and controlled-improvement artifacts are versioned, content-addressed, and independently verifiable.
4. Typed source references already identify runs, trajectory events, actions, tasks, evidence, findings, metrics, benchmark cells, comparisons, topology experiments, and generic artifacts.
5. The JSONL deployment protocol has compatibility fixtures, deterministic replay, process isolation, managed-tool mediation, and a third-party deployment conformance command.
6. The runner exposes small Rust ports for managed tools and process adapters while keeping infrastructure dependencies outside the domain and evaluation cores.
7. JSON Schemas 0.3 through 0.8 and canonical examples provide a source from which cross-language models can be derived and tested.
8. The canonical quality, security, adversarial, benchmark-science, diagnosis, controlled-improvement, end-to-end, documentation, and package gates are enforced locally and in GitHub Actions.

The audit also identifies gaps that R7 must close:

1. Current knowledge retrieval indexes authored documents, not verified historical HuntEval artifacts, and does not bind corpus membership to exact artifact hashes.
2. Corpus authorization is only an episode-local enable flag and root; it does not model evaluator analytics, deployment-visible knowledge, or explicit principals and permitted source classes.
3. Current citations address local document byte ranges but cannot resolve typed run, event, metric, comparison, report, or diagnostic sources.
4. Retrieval latency and cost fields exist but are currently deterministic zero values rather than explicit measured or unavailable provenance.
5. Retrieval records are not append-only, content-addressed public artifacts and cannot yet be verified offline.
6. There is no normalized structured query contract, deterministic analytical index manifest, query CLI, or report rendering for historical artifacts.
7. Existing Rust traits are internal composition boundaries, not versioned third-party extension contracts.
8. No normative adapter manifest declares executable identity, protocol/schema support, capabilities, filesystem access, network policy, budgets, or managed-tool authority.
9. Deployment conformance covers the core protocol but not a complete extension inventory, capability-policy agreement, or managed-tool adapter conformance.
10. There is no Python package, generated model pipeline, cross-language golden suite, packaging policy, or supported-version matrix.

These findings record the pre-R7 baseline used to design the milestones. The delivery-status table is the authoritative statement of current local implementation and release evidence.

## 3. Mandatory delivery rules

Every R7 pull request must:

- preserve Clean Architecture: domain policy stays independent from storage, CLI, Python, providers, agent frameworks, and concrete adapters;
- treat indexed artifacts, adapter output, SDK input, retrieved text, and third-party metadata as untrusted and bounded;
- index only artifacts that pass the required public verifier and whose exact bytes are content-addressed;
- keep evaluator analytical corpora physically and logically separate from deployment-visible knowledge;
- reject private ground truth, hidden partition membership or feedback, private paths and hashes, secrets, and private chain of thought from every public R7 artifact;
- keep scored-tool execution and budget accounting under the existing HuntEval runner;
- require explicit version, capability, filesystem, network, concurrency, timeout, byte, row, and cost declarations where applicable;
- fail closed on unknown versions, fields, capabilities, source kinds, citation targets, policies, and unsupported host guarantees;
- preserve raw metric vectors and existing scoring/ranking semantics without adding a global score or missing-value imputation;
- use typed errors without `unwrap()`, `expect()`, panic shortcuts, or first-party `unsafe` in production paths;
- keep Rust production files below 500 lines and split cohesive modules before 300 lines where practical;
- keep Python modules cohesive, typed, bounded, and covered by lint, test, and package checks selected in R7-00;
- use deterministic serialization, stable identifiers, SHA-256 content identities, UTC RFC 3339 timestamps, and safe relative paths;
- preserve schemas 0.3 through 0.8 and protocol fixtures byte-for-byte;
- update documentation, ADR status, migration, rollback, limitations, and exact acceptance commands with the owning change;
- avoid broad documentation churn and avoid tracking generated caches, environments, build outputs, credentials, or private material.

## 4. Architecture and dependency direction

R7 must preserve this dependency direction:

```text
hunteval-domain          stable identifiers and infrastructure-neutral R7 policy types
       ^
       |
hunteval-protocol        existing process-neutral deployment messages and replay
       ^
       |
hunteval-knowledge       pure corpus/query/citation policy and deterministic indexing
       ^
       |
hunteval-runner          verification, authorization, adapter ports, orchestration, journals
       ^
       |
hunteval-reporting       normalized R7 JSON and escaped script-free HTML projection
       ^
       |
hunteval-cli             composition, commands, filesystem adapters, process launch

schemas + fixtures  <->  pure Python SDK
                              |
                              +-- no Rust-core dependency on Python
                              +-- no runner or scored-tool authority
```

Concrete indexes, filesystems, subprocesses, package managers, and third-party adapters remain outer infrastructure. The consuming application layer owns ports. A third-party extension is an out-of-process executable speaking a bounded versioned protocol; R7 does not establish a stable Rust ABI or load third-party code into the HuntEval process.

The existing `hunteval-knowledge` crate remains optional and network-free. R7 may extend or split it into cohesive modules, but it must not become a second evaluator, report interpreter, or authorization authority. Artifact verification and corpus authorization occur before indexing. Query evaluation consumes verified normalized projections and returns typed source references; it does not reinterpret free-form text into causal, scoring, validation, or approval claims.

## 5. Architecture decisions to close in R7-00

The following ADRs are proposed. R7-00 must either accept them with exact contracts and tests or update this plan before implementation begins.

### ADR-076 — Add immutable knowledge and extension contracts

- Additive schema 0.9 defines analytical corpus, index, query, retrieval-audit, extension-manifest, conformance, and SDK compatibility artifacts.
- Schemas 0.3 through 0.8 remain immutable and readable through explicit adapters.
- Unknown fields, variants, capabilities, source classes, and newer incompatible versions fail closed.

### ADR-077 — Separate evaluator analytics from deployment-visible knowledge

- Every corpus has exactly one authorization scope and an explicit permitted source-class allowlist.
- Evaluator analytical artifacts are never exposed through a deployment session or managed retrieval tool.
- Deployment-visible corpora may contain only explicitly authored and authorized public documents and retain all existing untrusted-input controls.

### ADR-078 — Index only verified, content-addressed public artifacts

- Corpus membership binds exact artifact bytes, kind, verifier level/result, and safe provenance.
- Index construction rejects mutable paths, symlinks, unsupported artifacts, digest mismatches, private material, and failed or incomplete verification.
- An index manifest records deterministic tokenizer/index policy and exact ordered sources; changing any input changes index identity.

### ADR-079 — Make analytical answers typed deterministic projections

- R7.1 supports a bounded structured query vocabulary over normalized fields and exact source references.
- Results preserve matching records and citations; optional prose is non-normative and cannot create claims absent from verified sources.
- Query, result, latency, resource provenance, and every citation are recorded in an append-only auditable event.

### ADR-080 — Use out-of-process versioned extension contracts

- Managed-tool and deployment adapters declare supported contract versions and run as supervised processes behind existing sandbox and transport boundaries.
- HuntEval does not promise a stable Rust ABI, import adapter libraries, or grant implicit filesystem or network access.
- Contract negotiation selects one supported version or fails before any scored input is delivered.

### ADR-081 — Keep capability policy and scored-tool authority in HuntEval

- An adapter manifest is a request for capabilities, never an authorization grant.
- A runner-owned policy intersects declared requirements with benchmark and host policy, records the resolved capability set, and rejects undeclared use.
- Deployment agents and SDK clients cannot execute scored tools except through runner-mediated protocol actions.

### ADR-082 — Build the Python SDK from normative contracts

- The SDK is a pure client and artifact package aligned to committed JSON Schemas and compatibility fixtures.
- Generated files, if used, are reproducible and clearly separated from small hand-written validation, builder, and reader layers.
- The Rust workspace and domain model do not depend on Python, a Python runtime, or Python packaging tools.

### ADR-083 — Prove cross-language compatibility with canonical bytes

- Rust and Python independently parse the supported fixture inventory and produce equivalent canonical authored artifacts.
- Compatibility is defined by schema/protocol semantics and exact canonical test vectors, not by implementation-specific object layouts.
- A version is supported only while its immutable fixture set passes in both language suites.

## 6. Contract and compatibility strategy

R7-00 freezes the following schema 0.9 artifacts:

- `analytical-corpus-manifest.schema.json` — corpus identity, authorization scope, permitted source kinds, exact artifact entries, verifier requirements, limits, and policy digest;
- `analytical-index-manifest.schema.json` — ordered corpus identity, tokenizer/index policy, source hashes, field inventory, build result, and index digest;
- `analytical-query.schema.json` — bounded query kind, filters, requested fields, source families, result/byte budget, and corpus/index identity;
- `analytical-result.schema.json` — deterministic matches, typed citations, applicability, limitations, truncation, and exact query/index hashes;
- `retrieval-audit-event.schema.json` — append-only query/result identity, authorization decision, measured latency, resource/cost provenance, previous-event hash, and safe reason codes;
- `extension-capability-policy.schema.json` — allowed capability vocabulary and filesystem, network, process, time, byte, concurrency, row, and verified-cost bounds;
- `managed-tool-adapter-manifest.schema.json` — adapter identity, executable digest, supported versions, tool descriptors, required capabilities, limits, and provenance;
- `deployment-adapter-manifest.schema.json` — adapter identity, executable digest, supported protocol/schema versions, topology/deployment compatibility, required capabilities, limits, and provenance;
- `extension-resolution.schema.json` — requested and granted capabilities, policy hashes, compatibility selection, eligibility, and exact rejection reasons;
- `extension-conformance-result.schema.json` — manifest, policy, fixture inventory, behavioral checks, sandbox evidence, result, and exact hashes;
- `sdk-compatibility-index.schema.json` — supported schema/protocol versions, canonical fixtures, generated-model provenance where applicable, Python support policy, and package identity.

The exact names and field sets remain provisional until R7-00. Contract rules are mandatory:

- public schemas deny unknown fields and impose explicit collection, string, nesting, and byte bounds;
- analytical citations reuse or add explicit adapters for existing typed source references rather than introducing free-form identifiers;
- corpus entries never contain private ground-truth identifiers, private paths/hashes, hidden-test information, raw secrets, or host-specific absolute paths;
- an old artifact does not become R7-verifiable by inference; an explicit compatibility adapter must preserve absent fields as unavailable;
- schema 0.9 writers never rewrite older source artifacts;
- rollback disables 0.9 writers, index builders, extension negotiation, and SDK publication while retaining readers and emitted audit evidence;
- protocol 0.3 behavior remains unchanged unless R7-00 proves that an additive protocol version is necessary. A new protocol version requires its own immutable fixtures and negotiation tests.

### R7-00 — Contract freeze and architecture decisions

1. **Objective and user-visible outcome:** accept the persisted schema version, corpus authorization model, structured query vocabulary, extension process boundary, capability policy, SDK surface, Python support policy, and ADR decisions required before production implementation.
2. **Affected contracts and compatibility:** freeze the schema 0.9 inventory and canonical examples listed above, plus the exact supported schema/protocol fixture matrix; do not edit schemas 0.3 through 0.8.
3. **Security impact:** complete a focused threat-model update for analytical corpora, indexing, citations, third-party processes, capability resolution, cross-language parsing, and package supply chain.
4. **Ground-truth-isolation impact:** prove at contract level that evaluator analytics and deployment-visible knowledge are disjoint and that no public extension or SDK type admits private sources.
5. **Positive tests:** validate every canonical schema 0.9 example and explicit compatibility adapter against the proposed Rust domain types.
6. **Negative tests:** reject unknown versions/fields/source classes/capabilities, mixed authorization scopes, private source kinds, implicit policy grants, and unsupported compatibility claims.
7. **Malformed-input tests:** cover contract size/nesting limits, invalid identifiers/digests/timestamps/paths, duplicate semantic identities, and invalid discriminators.
8. **Deterministic/replay tests:** canonical examples round-trip deterministically and the fixture inventory has a stable ordered digest.
9. **Exact quality gates:** focused schema/domain compatibility tests, documentation checks, `git diff --check`, and all applicable existing gates in section 18.
10. **Documentation and ADR changes:** add accepted ADR-076 through ADR-083, schema README, contract tables, compatibility policy, and exact Python toolchain commands selected after dependency/license review.
11. **Migration behavior:** define explicit adapters and unavailable fields for older artifacts; never rewrite stored bytes or infer R7 authorization/capability metadata.
12. **Rollback behavior:** contracts and readers may land before writers; rollback disables composition of unimplemented writers while retaining accepted fixtures and compatibility tests.
13. **Known limitations:** contract acceptance makes no runtime, package, conformance, or release-completion claim.

## 7. R7.1 — Artifact-grounded local search

### R7-01 — Authorized analytical corpus manifest

1. **Objective and user-visible outcome:** allow an operator to declare exactly which verified public HuntEval artifacts form one local analytical corpus.
2. **Affected contracts and compatibility:** add the corpus manifest and source-entry contracts; preserve episode-authored knowledge manifests and schemas 0.3 through 0.8.
3. **Security impact:** validate bounded no-follow paths, exact hashes, allowed artifact types, verifier results, corpus size, entry count, and safe labels before accepting membership.
4. **Ground-truth-isolation impact:** define disjoint `evaluator_analytics` and `deployment_visible` scopes; reject mixed scopes and every private source class.
5. **Positive tests:** build manifests from verified run, benchmark, report, topology, diagnosis, and improvement bundles with stable ordering and identity.
6. **Negative tests:** reject failed/incomplete verification, duplicate identities, digest drift, symlinks, paths outside the root, unsupported media, private markers, and scope mixing.
7. **Malformed-input tests:** unknown fields, invalid versions, oversized collections/strings, invalid identifiers/digests, and excessive nesting fail with typed safe errors.
8. **Deterministic/replay tests:** reordered authored entries normalize to one canonical inventory and identical bytes produce the same corpus digest.
9. **Exact quality gates:** focused schema/domain/runner corpus tests plus all mandatory gates listed in section 18.
10. **Documentation and ADR changes:** accept the owning portions of ADR-076 through ADR-078; document corpus scopes and source eligibility.
11. **Migration behavior:** existing knowledge manifests remain deployment-visible authored-document inputs and are not silently converted into analytical corpora.
12. **Rollback behavior:** disable new corpus writers while retaining schema readers and immutable manifests.
13. **Known limitations:** only local verified public artifacts are eligible; remote registries and private analytical stores are not introduced.

### R7-02 — Deterministic verified-artifact index

1. **Objective and user-visible outcome:** build a reproducible local index from one accepted corpus without network access.
2. **Affected contracts and compatibility:** add index policy and manifest contracts; adapt only explicitly supported normalized public artifact versions.
3. **Security impact:** parse bounded normalized fields, never execute embedded content, use no-follow reads, cap memory/output/work, and write atomically beneath a validated new output root.
4. **Ground-truth-isolation impact:** index construction revalidates corpus scope and rejects fields or artifacts classified private before tokenization or field projection.
5. **Positive tests:** index representative run, metric, comparison, timeline, topology, diagnosis, recommendation, validation, and public document fields.
6. **Negative tests:** reject tampered sources, unsupported/newer artifacts, active content, invalid UTF-8 where text is required, decompression bombs, stale verifier results, and output overwrite.
7. **Malformed-input tests:** truncated JSON/JSONL, invalid hash chains, duplicate source keys, invalid pointers, and excessive document/field/token counts fail closed.
8. **Deterministic/replay tests:** clean rebuilds produce equivalent manifests, field inventories, postings, and query behavior independent of filesystem enumeration order.
9. **Exact quality gates:** focused knowledge/index tests, verifier tests, secret scan of generated artifacts, and section 18 gates.
10. **Documentation and ADR changes:** accept ADR-078; document indexed fields, tokenizer policy, limits, and non-indexed content.
11. **Migration behavior:** the existing authored-document token index remains available behind its current API until callers migrate explicitly.
12. **Rollback behavior:** remove the new builder from composition while retaining readable manifests; disposable index files may be rebuilt from immutable sources.
13. **Known limitations:** initial search is deterministic lexical/structured retrieval, not semantic embedding search or a causal inference engine.

### R7-03 — Structured query and typed citation resolution

1. **Objective and user-visible outcome:** answer bounded structured questions over an index with exact resolvable citations and explicit limitations.
2. **Affected contracts and compatibility:** add query/result contracts and typed analytical citation adapters for existing source identities.
3. **Security impact:** allowlist query operations and fields; cap filters, terms, result count, bytes, and work; escape all untrusted values.
4. **Ground-truth-isolation impact:** query authorization binds the corpus scope and rejects requests for private or undeclared source families.
5. **Positive tests:** retrieve run events, metric vectors, comparisons, diagnostic classifications, topology observations, and controlled-improvement states with exact citations.
6. **Negative tests:** reject free-form execution expressions, unknown fields/operators, cross-corpus references, dangling citations, private pointers, and over-budget requests.
7. **Malformed-input tests:** invalid selectors, types, ranges, Unicode boundaries, identifiers, pagination tokens, and nested boolean expressions fail safely.
8. **Deterministic/replay tests:** identical index/query bytes return byte-equivalent ordered results and citation sets; ties use stable identities.
9. **Exact quality gates:** focused query, citation, injection, and snapshot tests plus section 18 gates.
10. **Documentation and ADR changes:** accept ADR-079; publish supported question/query kinds and citation semantics.
11. **Migration behavior:** existing document retrieval requests retain their behavior and are not interpreted as analytical queries.
12. **Rollback behavior:** disable analytical query composition while preserving index/corpus verification and stored audit events.
13. **Known limitations:** results expose verified observations and existing claims; they do not synthesize new causal, scoring, validation, approval, or transfer claims.

### R7-04 — Retrieval audit, CLI, reporting, and verification

1. **Objective and user-visible outcome:** provide local `knowledge corpus validate`, `knowledge index build`, `knowledge query`, and `knowledge verify` workflows with authoritative JSON output.
2. **Affected contracts and compatibility:** add append-only retrieval audit events and normalized query reporting; commands are additive.
3. **Security impact:** CLI output uses safe relative locations/reason codes, reports escape untrusted content, and public artifacts pass secret scanning.
4. **Ground-truth-isolation impact:** the CLI never offers a flag that upgrades a deployment-visible session to evaluator analytics or reveals rejected private matches.
5. **Positive tests:** validate, build, query, inspect audit history, render static HTML, and verify a complete bundle offline.
6. **Negative tests:** reject altered index/query/result/audit bytes, broken hash links, mismatched authorization, stale source digests, unsafe output paths, and overwrite attempts.
7. **Malformed-input tests:** truncated journals, duplicate sequence numbers, invalid timestamps, excessive output, unknown reason codes, and missing artifacts fail closed.
8. **Deterministic/replay tests:** journal replay reproduces state and normalized JSON; HTML is a deterministic escaped script-free projection.
9. **Exact quality gates:** CLI integration, report snapshot, bundle verifier, secret scan, and section 18 gates.
10. **Documentation and ADR changes:** update CLI, contracts, threat model, and analytical-search use-case documentation.
11. **Migration behavior:** no existing command changes meaning; new command groups are additive.
12. **Rollback behavior:** retain read-only verification and journal readers even if builders/query writers are disabled.
13. **Known limitations:** measured local latency is reportable; provider cost stays unavailable unless supplied by an independently verifiable adapter.

### R7-05 — R7.1 closure

1. **Objective and user-visible outcome:** prove a complete artifact-grounded search workflow over representative verified R2 through R6 outputs.
2. **Affected contracts and compatibility:** freeze the R7.1 compatibility inventory and canonical examples.
3. **Security impact:** run injection, path, artifact-tampering, authorization, resource-bound, report-escaping, and secret-scan suites.
4. **Ground-truth-isolation impact:** prove that evaluator analytics cannot be requested through a deployment session and deployment-visible corpora contain no evaluator-only sources.
5. **Positive tests:** answer documented structured questions with citations to runs, events, metrics, comparisons, diagnoses, and experiments.
6. **Negative tests:** unsupported metrics and absent source families remain unavailable rather than inferred; observational sources cannot create causal wording.
7. **Malformed-input tests:** run the full malformed corpus/index/query/audit corpus under bounded resources.
8. **Deterministic/replay tests:** rebuild and replay from a clean checkout and compare normalized identities and results.
9. **Exact quality gates:** dedicated R7 knowledge script plus section 18 gates.
10. **Documentation and ADR changes:** mark R7.1 complete only with exact local and remote evidence.
11. **Migration behavior:** record the supported source-version matrix and explicit unavailable fields.
12. **Rollback behavior:** demonstrate writer-disable/read-verify behavior against emitted fixtures.
13. **Known limitations:** no remote retrieval, vector database, natural-language claim generator, or deployment access to evaluator history.

## 8. R7.2 — Stable extension contracts

### R7-06 — Managed-tool adapter contract

1. **Objective and user-visible outcome:** let a third party implement a bounded managed tool behind a stable process-neutral contract while HuntEval retains execution authority.
2. **Affected contracts and compatibility:** add managed-tool adapter manifest, request/result fixture inventory, and explicit supported-version negotiation.
3. **Security impact:** tools run out of process under runner-selected sandbox, filesystem, network, resource, output, and lifecycle policies; text/results remain untrusted.
4. **Ground-truth-isolation impact:** adapter input contains only the authorized public episode view and declared arguments; private truth and hidden-test metadata are prohibited.
5. **Positive tests:** deterministic reference adapter handles registration, valid request, typed result, truncation, timeout, and shutdown through the production supervisor.
6. **Negative tests:** undeclared tool/capability, direct agent execution, network/filesystem escape, extra output, crash, timeout, digest drift, and protocol downgrade fail safely.
7. **Malformed-input tests:** invalid JSONL, oversized lines, unknown messages, duplicate responses, wrong correlation IDs, invalid scalar types, and premature EOF.
8. **Deterministic/replay tests:** supported transcripts replay equivalently and exact adapter/policy digests appear in run provenance.
9. **Exact quality gates:** adapter schema, transport, sandbox, policy, conformance, and section 18 gates.
10. **Documentation and ADR changes:** accept ADR-080 and ADR-081; document the adapter lifecycle and trust boundary.
11. **Migration behavior:** the existing DuckDB adapter remains canonical and gains an explicit manifest adapter without changing scored SQL semantics.
12. **Rollback behavior:** disable third-party adapter selection while retaining DuckDB and manifest/conformance readers.
13. **Known limitations:** R7 conformance covers local deterministic tools; production SIEM execution remains post-v1.0.

### R7-07 — Deployment-adapter contract

1. **Objective and user-visible outcome:** define a stable manifest and launch boundary for framework-neutral single-agent and multi-agent deployment executables.
2. **Affected contracts and compatibility:** bind existing deployment/protocol/topology contracts to a versioned adapter manifest and immutable compatibility inventory.
3. **Security impact:** validate executable identity and launch arguments, minimize environment, enforce sandbox/resource policy, bound diagnostics, and reject shell interpretation.
4. **Ground-truth-isolation impact:** no adapter receives private roots, hashes, hidden partition data, evaluator corpora, or ground-truth-derived diagnostics.
5. **Positive tests:** reference single-agent, supervisor-worker, and supervisor-specialist deployments conform through the same adapter boundary.
6. **Negative tests:** registration/topology mismatch, undeclared capability, unsupported protocol, extra agents, unsafe paths, environment requests, direct tool access, and process leaks fail closed.
7. **Malformed-input tests:** malformed manifests, arguments, versions, topology identities, capability lists, and executable digests return typed safe errors.
8. **Deterministic/replay tests:** canonical transcripts and run artifacts remain semantically equivalent across the manifest-backed launch path.
9. **Exact quality gates:** deployment conformance, topology conformance, sandbox, replay, compatibility, and section 18 gates.
10. **Documentation and ADR changes:** extend ADR-080/081 coverage and deployment-author guidance.
11. **Migration behavior:** existing deployment commands adapt explicitly; no manifest is fabricated for stored historical runs.
12. **Rollback behavior:** retain the previous internal launch composition until parity is proven, then keep a reader-only adapter for old definitions.
13. **Known limitations:** no framework-specific SDK, hosted orchestrator, container scheduler, or production deployment manager is introduced.

### R7-08 — Capability resolution and policy enforcement

1. **Objective and user-visible outcome:** make every extension request and every granted capability explicit, bounded, content-addressed, and auditable.
2. **Affected contracts and compatibility:** add capability policy and resolution contracts linked to execution policy, benchmark controls, manifest, and host capability report.
3. **Security impact:** use deny-by-default intersection; reject wildcard/unknown capability, silent fallback, excess filesystem/network/process access, or unverifiable budget provenance.
4. **Ground-truth-isolation impact:** no capability can grant private-root, hidden-test, evaluator-corpus, or ground-truth access.
5. **Positive tests:** resolve minimal local read-only capabilities for the reference deployment and DuckDB-compatible managed tool.
6. **Negative tests:** requested network, write access, secret environment, dynamic extension loading, excess resources, and mismatched policy hashes make scored execution ineligible.
7. **Malformed-input tests:** duplicate/conflicting rules, invalid limits, unknown provenance, invalid paths, unsupported host capabilities, and empty policy sets fail closed.
8. **Deterministic/replay tests:** identical manifest/policy/host inputs produce the same ordered resolution and digest.
9. **Exact quality gates:** policy property tests, sandbox negative tests, resource tests, secret scan, and section 18 gates.
10. **Documentation and ADR changes:** accept ADR-081 fully; update threat model and operations guidance.
11. **Migration behavior:** existing execution policy remains authoritative; extension policy narrows it and cannot broaden older grants.
12. **Rollback behavior:** disable extension resolution and allow only existing first-party compositions under existing policy.
13. **Known limitations:** denied network remains mandatory for scored pre-v1.0 deployment execution.

### R7-09 — Compatibility fixtures, conformance CLI, and R7.2 closure

1. **Objective and user-visible outcome:** allow extension authors to prove compatibility locally before submitting an adapter.
2. **Affected contracts and compatibility:** add extension conformance result and immutable managed-tool/deployment adapter fixture indexes.
3. **Security impact:** conformance uses the production parser, sandbox, supervisor, bounds, policy resolver, redaction, and secret scanner.
4. **Ground-truth-isolation impact:** fixtures are synthetic and public; conformance cannot access benchmark private roots or hidden partitions.
5. **Positive tests:** `extension validate` and `extension conformance` succeed for canonical reference adapters and emit offline-verifiable results.
6. **Negative tests:** malicious fake adapters cover protocol injection, process escape, undeclared access, hangs, crashes, malformed output, and cleanup failure.
7. **Malformed-input tests:** mutate every contract discriminator and retained transcript family with stable expected reason codes.
8. **Deterministic/replay tests:** exact fixture inventory and conformance bytes are stable for one adapter/policy/binary set.
9. **Exact quality gates:** dedicated extension gate, existing adversarial protocol gate, and section 18 gates.
10. **Documentation and ADR changes:** publish extension authoring, version support, compatibility, security, and review documentation.
11. **Migration behavior:** existing `deployment conformance` remains supported or becomes an explicit compatibility alias with unchanged semantics.
12. **Rollback behavior:** retain fixture verification and disable new adapter execution if a contract or sandbox defect is found.
13. **Known limitations:** conformance proves contract behavior under declared fixtures and policy, not operational fitness or universal security.

## 9. R7.3 — Python SDK

### R7-10 — Typed schema-aligned Python models

1. **Objective and user-visible outcome:** provide importable typed Python representations and validators for supported authored manifests, protocol messages, and public artifacts.
2. **Affected contracts and compatibility:** derive or hand-map from the normative schema/protocol inventory selected in R7-00; Rust contracts remain authoritative.
3. **Security impact:** bound input bytes, nesting, strings, collections, integers, and unknown fields before object construction; avoid unsafe object hooks and arbitrary code loading.
4. **Ground-truth-isolation impact:** the public SDK excludes private episode and hidden-test types from its public artifact readers and examples.
5. **Positive tests:** parse every canonical supported fixture and round-trip authored artifacts with typed identifiers, digests, timestamps, enums, and optional values.
6. **Negative tests:** reject unknown versions/fields, invalid discriminators, non-finite numbers, invalid paths/digests/timestamps, and private-field injection.
7. **Malformed-input tests:** bounded fuzz/property corpus covers truncated JSON/JSONL, deep nesting, huge integers/strings, duplicate semantic keys, and invalid Unicode.
8. **Deterministic/replay tests:** canonical Python serialization matches committed vectors where exact bytes are normative and semantic equivalence elsewhere.
9. **Exact quality gates:** pinned Python formatting, lint, type-check, unit/property tests, dependency audit, and section 18 gates.
10. **Documentation and ADR changes:** accept ADR-082/083; publish supported Python and schema/protocol version policy selected in R7-00.
11. **Migration behavior:** package versions expose explicit compatibility modules; removed or renamed fields require deprecation policy rather than silent aliases.
12. **Rollback behavior:** withdraw a broken package candidate without changing Rust artifacts or overwriting an existing published version.
13. **Known limitations:** the SDK models contracts; it is not a Python reimplementation of evaluation, scoring, verification, or orchestration.

### R7-11 — Builders and verified public artifact readers

1. **Objective and user-visible outcome:** let Python users safely author manifests and inspect content-addressed public HuntEval results without manually assembling dictionaries.
2. **Affected contracts and compatibility:** add typed builders/readers for roadmap-required manifests, run artifacts, reports, and exact digest inventories.
3. **Security impact:** readers use no-follow local access, root confinement, byte limits, exact digest verification, and no implicit network fetching.
4. **Ground-truth-isolation impact:** builders cannot place ground truth into deployment-visible manifests and readers reject private artifact roots/types.
5. **Positive tests:** build deployment/benchmark/extension manifests and read verified run, benchmark, topology, diagnosis, improvement, and knowledge results.
6. **Negative tests:** reject traversal, symlinks, changed bytes, incomplete bundles, unsafe output paths, overwrite attempts, private artifacts, and unknown versions.
7. **Malformed-input tests:** invalid YAML/JSON/JSONL, duplicate identifiers, broken journals, missing inventory entries, and size-limit violations fail with stable exceptions.
8. **Deterministic/replay tests:** builders emit stable canonical authored bytes and readers agree with Rust verification fixtures.
9. **Exact quality gates:** builder/reader integration, cross-language fixtures, package-content scan, and section 18 gates.
10. **Documentation and ADR changes:** add Python quickstart, artifact-reading use case, and security guidance.
11. **Migration behavior:** old supported artifact readers remain version-specific; unavailable newer fields are not inferred.
12. **Rollback behavior:** retain model parsing while disabling a faulty builder/reader entry point in a new package version.
13. **Known limitations:** initial readers are local and offline; remote registries, cloud storage, and signing are not included.

### R7-12 — Bounded deployment-protocol client

1. **Objective and user-visible outcome:** help Python deployment authors implement the existing JSONL peer protocol without transferring runner authority.
2. **Affected contracts and compatibility:** expose supported deployment-side messages, state transitions, correlation identities, and conformance fixtures.
3. **Security impact:** bound stdin/stdout framing, message size/count, diagnostics, timeouts, and state; never execute shell commands or tools on behalf of the deployment.
4. **Ground-truth-isolation impact:** public types contain only runner-delivered public observations and mediated tool results.
5. **Positive tests:** a minimal single-agent and multi-agent Python fixture complete registration, task/action/message flow, managed-tool request, and final submission under the real runner.
6. **Negative tests:** client rejects illegal transitions, duplicate/orphan messages, forged runner fields, direct tool shortcuts, oversized output, and post-terminal messages.
7. **Malformed-input tests:** every protocol discriminator, identifier, correlation, framing, and bounded collection receives negative fixtures.
8. **Deterministic/replay tests:** Python fixture transcripts pass Rust replay and compatibility suites with stable semantic results.
9. **Exact quality gates:** Python protocol tests, Rust conformance, adversarial suite, live-process integration, and section 18 gates.
10. **Documentation and ADR changes:** document that the SDK is a deployment peer, not an orchestrator or tool authority.
11. **Migration behavior:** negotiate only explicitly supported protocol versions and retain immutable fixture modules for older supported versions.
12. **Rollback behavior:** disable the client helper while retaining models/readers; existing non-Python peers remain unaffected.
13. **Known limitations:** no model-provider client, agent-framework integration, retry policy override, or runner control API is included.

### R7-13 — Cross-language compatibility, packaging, and R7.3 closure

1. **Objective and user-visible outcome:** produce a reproducible installable SDK candidate and prove Rust/Python compatibility for the declared support matrix.
2. **Affected contracts and compatibility:** freeze the SDK compatibility index, fixture hashes, package metadata, and supported-version table.
3. **Security impact:** pin build/test tooling, audit runtime/build dependencies and licenses, inspect wheel/sdist contents, and scan packages for secrets/private/generated junk.
4. **Ground-truth-isolation impact:** package inventory and examples contain no private episodes, hidden tests, evaluator-only artifacts, or private hashes.
5. **Positive tests:** clean-environment wheel and source-package builds install and run quickstart, validation, artifact-reader, and protocol-conformance examples offline.
6. **Negative tests:** altered fixtures/package files, unsupported interpreters/versions, missing metadata, dependency drift, and non-reproducible generation fail the package gate.
7. **Malformed-input tests:** installed package repeats the public malformed contract/protocol corpus without repository-relative assumptions.
8. **Deterministic/replay tests:** two clean builds have equivalent declared contents and all cross-language golden vectors agree.
9. **Exact quality gates:** Python quality/security/package matrix, Rust/Python compatibility command, and section 18 gates.
10. **Documentation and ADR changes:** finalize ADR-082/083, installation instructions, support policy, changelog, and package verification guide.
11. **Migration behavior:** package and schema/protocol compatibility versions are explicit and independently versioned.
12. **Rollback behavior:** do not publish from ordinary CI; discard a failed candidate and issue a new version rather than overwriting artifacts.
13. **Known limitations:** signing and public package publication are release-governance decisions; R7 closure requires a non-publishing dry run.

## 10. Integration and release closure

### R7-14 — End-to-end R7 workflow and dedicated CI gate

1. **Objective and user-visible outcome:** prove local search, both extension contracts, and the Python SDK work together without weakening the benchmark path.
2. **Affected contracts and compatibility:** cover the complete schema 0.9 inventory and supported older schemas/protocol fixtures.
3. **Security impact:** run all R7 workflows under bounded local policies and scan every generated public/log/package artifact.
4. **Ground-truth-isolation impact:** exercise distinct evaluator and deployment corpora and prove no cross-scope data reaches the Python deployment fixture.
5. **Positive tests:** run a reference benchmark through manifest-backed adapters, index verified outputs, execute cited queries, read results in Python, and verify all artifacts offline.
6. **Negative tests:** seed one failure for corpus authorization, adapter capability, SDK private-field injection, citation integrity, and gate propagation.
7. **Malformed-input tests:** run retained R7 schema, query, extension, and Python protocol corpora.
8. **Deterministic/replay tests:** repeat the workflow from clean outputs and compare semantic and exact identities according to each contract.
9. **Exact quality gates:** add `scripts/ci/r7-extensions.sh` and a required `Knowledge and extensions` GitHub Actions job without weakening existing jobs.
10. **Documentation and ADR changes:** document the end-to-end use case and CI evidence format.
11. **Migration behavior:** all older compatibility suites remain in the new gate or canonical existing gates.
12. **Rollback behavior:** remove the R7 job only together with R7 writers before release; never remove historical readers/evidence silently.
13. **Known limitations:** the workflow is local, offline, and fixture-backed; it does not score production SIEM execution.

### R7-15 — R7 release closure

1. **Objective and user-visible outcome:** close v0.7 with exact implementation, compatibility, security, package, and remote governance evidence.
2. **Affected contracts and compatibility:** freeze schema 0.9 and SDK/extension compatibility matrices; preserve all R2 through R6 evidence.
3. **Security impact:** complete threat-model review, dependency/license audit, sandbox/adversarial tests, package inspection, secret scans, and limitations review.
4. **Ground-truth-isolation impact:** record negative evidence for corpus separation, adapter inputs, SDK package contents, public reports, and CI artifacts.
5. **Positive tests:** run all focused and canonical local gates plus a non-publishing release-candidate dry run from a clean tree.
6. **Negative tests:** required-check propagation, stale/tampered artifacts, unsupported compatibility, and rollback-read behavior remain enforced.
7. **Malformed-input tests:** record retained corpus hashes and stable outcomes for every new public parser/protocol boundary.
8. **Deterministic/replay tests:** record clean rebuild, index/query replay, extension transcript replay, and cross-language fixture equivalence.
9. **Exact quality gates:** every existing required job plus `Knowledge and extensions` passes on the exact evidence revision and is required by protected-branch policy.
10. **Documentation and ADR changes:** update roadmap, README, contracts, specification, operations, release checklist, completion evidence, and accepted ADR status without rewriting prior evidence.
11. **Migration behavior:** publish the exact schema/protocol/SDK support and rejection matrix.
12. **Rollback behavior:** document writer and package disablement while retaining all readers, fixtures, journals, manifests, and verification paths.
13. **Known limitations:** explicitly retain every post-v1.0 deferred item and every R7-specific limitation from section 19.

## 11. Milestone dependency graph

```text
R6 complete
  -> R7-00 contracts and ADR decisions

R7-00
  -> R7-01 authorized corpus
       -> R7-02 deterministic index
            -> R7-03 structured query/citations
                 -> R7-04 audit/CLI/verification
                      -> R7-05 R7.1 closure

R7-00
  -> R7-06 managed-tool adapter
  -> R7-07 deployment adapter
       -> R7-08 capability resolution
            -> R7-09 conformance and R7.2 closure

R7-00 + stable schemas/protocol fixtures
  -> R7-10 Python models
       -> R7-11 builders/readers
       -> R7-12 deployment-protocol client
            -> R7-13 compatibility/package and R7.3 closure

R7-05 + R7-09 + R7-13
  -> R7-14 end-to-end/CI
       -> R7-15 release closure
            -> v0.8 release-candidate planning
```

Corpus and extension work may proceed in separate reviewable branches after R7-00, but R7.3 must consume frozen normative contracts and R7 cannot close until all three initiatives converge in R7-14. No milestone may create a parallel evaluator, runner, verifier, scoring engine, protocol authority, or safety-policy system.

## 12. Delivery waves

### Wave A — Freeze boundaries and corpus authorization

1. R7-00 contracts and architecture decisions.
2. R7-01 exact corpus membership and authorization scopes.
3. R7-02 deterministic verified-artifact index.

### Wave B — Make historical evidence queryable

1. R7-03 structured queries and typed citations.
2. R7-04 audit journal, CLI, reports, and verifier.
3. R7-05 isolation evidence and R7.1 closure.

### Wave C — Stabilize third-party process boundaries

1. R7-06 managed-tool adapter contract.
2. R7-07 deployment-adapter contract.
3. R7-08 capability resolution and policy enforcement.
4. R7-09 conformance CLI and R7.2 closure.

### Wave D — Deliver the Python client surface

1. R7-10 typed models.
2. R7-11 builders and public artifact readers.
3. R7-12 bounded protocol client.
4. R7-13 cross-language and package closure.

### Wave E — Integrate and close R7

1. R7-14 end-to-end workflow and required CI job.
2. R7-15 release evidence, migration, rollback, and roadmap closure.

## 13. Milestone handoff checklist

Before completing any R7 milestone:

1. the user-visible outcome is implemented without unrelated v0.8 or post-v1.0 scope;
2. every affected contract has a schema, canonical example, explicit bounds, validation, and compatibility coverage;
3. security and ground-truth-isolation effects are documented and negatively tested;
4. positive, negative, malformed-input, deterministic/replay, stale-artifact, injection, and resource-bound tests pass;
5. every corpus, source, index, query, result, citation, adapter, policy, resolution, conformance result, fixture, and package resolves exact content-addressed inputs;
6. evaluator analytical corpora and deployment-visible knowledge remain disjoint and no implicit scope upgrade exists;
7. extension capabilities are deny-by-default, runner-authorized, and bound to exact policy and executable identities;
8. Python cannot acquire runner, evaluator, filesystem, network, scored-tool, or ground-truth authority by using the SDK;
9. unsupported data remains unavailable and no query/report creates inferred metrics, scores, causal claims, validation, approval, or adoption;
10. first-party production code contains no unsafe, panic shortcut, unbounded input, private leakage, provider coupling, or in-process third-party loading;
11. Rust and Python source files remain cohesive, typed, readable, and within their enforced size/style policies;
12. exact focused commands and all canonical gates pass;
13. documentation, ADR status, migration, rollback, limitations, and `git diff --check` are current;
14. no private, generated cache, virtual environment, credential, secret-bearing, or unrelated artifact is tracked;
15. a descriptive commit exists before status changes to `complete`, with remote evidence required for initiative and release closure.

Remote failure returns the milestone to active status until the exact revision passes locally and remotely.

## 14. Risk register

| Risk | Impact | Mitigation and rollback |
|---|---|---|
| evaluator corpus reaches a deployment | benchmark leakage | disjoint scope enum, separate composition ports, negative end-to-end proof, fail closed |
| unverified artifact enters an index | fabricated or stale answer | exact source hash plus required verifier result before indexing |
| free-form search becomes claim generation | unsupported conclusions | bounded structured vocabulary and typed source projection only |
| citation points to mutable or wrong bytes | unauditable answer | bind source digest, typed identity, field/byte range, and verifier evidence |
| malicious indexed text drives behavior | prompt/policy injection | treat all text as data, no execution, escape output, preserve untrusted marker |
| index order varies by host | irreproducible results | canonical source ordering, fixed tokenizer/policy, stable tie-breakers |
| retrieval cost is fabricated | misleading resource comparison | measured or verified-adapter provenance; otherwise explicitly unavailable |
| extension manifest is treated as authorization | privilege escalation | runner-owned deny-by-default policy intersection and recorded resolution |
| third-party code runs in-process | memory/safety boundary loss | out-of-process protocol only; no stable Rust ABI or dynamic loading |
| adapter bypasses managed tools | invalid score/provenance | runner-mediated actions only, conformance and sandbox negative tests |
| extension requests network access | pre-v1 scope/security breach | scored network remains denied; reject request before episode delivery |
| protocol/version downgrade | compatibility or policy bypass | explicit negotiation and immutable supported-version fixtures |
| SDK diverges from Rust | invalid authored inputs/readers | schema-derived inventory and bidirectional golden compatibility suite |
| generated SDK code is unreviewable | maintenance/security defects | deterministic generator, separated generated tree, small hand-written layer |
| Python parser accepts unsafe extras | contract bypass | forbid unknown fields and repeat malformed corpus across languages |
| SDK becomes alternate orchestrator | split authority and evidence | public surface limited to builders, readers, validation, and deployment peer |
| package leaks private material | benchmark/security disclosure | explicit package inventory, clean build, secret/private marker scan |
| R7 breaks older artifacts | lost reproducibility | additive readers, immutable fixtures, explicit unavailable fields, writer-only rollback |
| modules become unmaintainable | review and security defects | cohesive modules, Rust 300-line review threshold and 500-line hard limit, equivalent Python gate |

## 15. R7 completion definition

R7 is complete only when:

1. additive R7 contracts are bounded, versioned, content-addressed, deny unknown fields, and preserve schemas 0.3 through 0.8;
2. an analytical corpus contains only explicitly authorized, verified public artifact bytes and has a deterministic identity;
3. evaluator analytical corpora and deployment-visible knowledge are represented and enforced as disjoint scopes;
4. no deployment session, managed retrieval request, adapter, or SDK client can access evaluator-only analytical sources;
5. deterministic local index rebuilds over equivalent sources produce equivalent manifests and query behavior;
6. structured queries are bounded, versioned, replayable, and tied to an exact corpus and index;
7. every result claim and match cites an exact verified run, event, metric, comparison, report, diagnosis, experiment, recommendation, or document source where supported;
8. unsupported, missing, private, or unverifiable source families remain unavailable rather than inferred;
9. retrieval audit artifacts record authorization, query/result identity, citations, measured latency, resource/cost provenance, and hash-linked order;
10. managed-tool and deployment adapters use versioned out-of-process contracts and production sandbox/supervision boundaries;
11. adapter capability requests are deny-by-default and cannot broaden execution, filesystem, network, budget, tool, or data policy;
12. scored tools remain runner-mediated and no adapter or SDK can execute them directly;
13. compatibility and conformance suites cover supported versions, malformed messages, process failures, policy violations, cleanup, replay, and exact executable/policy hashes;
14. the Python SDK provides typed builders/readers and a bounded deployment peer for every roadmap-required supported contract;
15. Rust and Python accept/reject the same canonical and malformed fixtures according to the declared compatibility matrix;
16. Python package candidates build and install reproducibly in clean supported environments with audited contents and dependencies;
17. normalized JSON remains authoritative, static HTML is escaped and script-free, and public artifacts verify offline;
18. public R7 artifacts and packages contain no ground truth, hidden-test feedback, private paths/hashes, secrets, or private chain of thought;
19. existing R2 through R6 quality, security, benchmark, diagnosis, controlled-improvement, end-to-end, documentation, and package behavior remains green;
20. the dedicated R7 local gate and required GitHub Actions job pass on the exact evidence revision.

Completion evidence must record exact commands, revisions, toolchain and Python support matrix, schema/protocol/fixture hashes, corpus and authorization-policy hashes, source-verifier results, index/query/result/audit hashes, adapter executable/manifests/policies/resolution/conformance hashes, SDK compatibility/package hashes, deterministic replay results, secret scans, known limitations, and ADR status changes.

## 16. Proposed contract inventory by owner

| Owner | Planned responsibility | Must not own |
|---|---|---|
| `hunteval-domain` | infrastructure-neutral identifiers, authorization scope, source/capability policy values where generally reusable | filesystem indexing, process launch, Python generation |
| `hunteval-knowledge` | deterministic corpus/index/query/citation rules over verified inputs | source verification authority, ground truth, report claim generation |
| `hunteval-protocol` | existing deployment session semantics and any explicitly accepted additive adapter framing | adapter policy, process launch, Python runtime |
| `hunteval-runner` | artifact verification, corpus authorization, adapter ports, capability resolution, process supervision, audit journals | provider-specific clients, report rendering |
| `hunteval-reporting` | normalized analytical/conformance results and safe static projections | query execution, policy authorization |
| `hunteval-cli` | command composition, local paths, selected adapters, output routing | domain policy, scoring logic |
| Python SDK | typed contract models, builders, public readers, validation, deployment peer | evaluation, scoring, verification authority, orchestration, direct tools |

R7-00 may refine module names but cannot reverse these dependencies without a roadmap and ADR update.

## 17. Proposed ADR updates

R7-00 must add ADR-076 through ADR-083 to `ADR.md` only after their contracts and canonical examples are accepted:

- R7-00: ADR-076 contract version and compatibility;
- R7-01/R7-05: ADR-077 corpus-scope isolation;
- R7-01/R7-02: ADR-078 verified content-addressed indexing;
- R7-03/R7-04: ADR-079 typed deterministic answers and audit;
- R7-06/R7-07: ADR-080 out-of-process extensions;
- R7-08/R7-09: ADR-081 runner-owned capability policy;
- R7-10/R7-13: ADR-082 schema-aligned SDK and ADR-083 cross-language compatibility.

No accepted ADR-001 through ADR-075 is reopened or weakened.

## 18. Initial acceptance command inventory

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

Owning milestones must add focused commands equivalent to:

```bash
cargo test -p hunteval-domain --test schema_v09 --test extension_v09
cargo test -p hunteval-knowledge --test analytical
cargo test -p hunteval-runner --test knowledge_artifacts
cargo test -p hunteval-runner --test knowledge_audit
cargo test -p hunteval-runner --test extension_policy
cargo test -p hunteval-runner --test managed_tool_conformance
cargo test -p hunteval-reference-deployment --test extension_contract
cargo test -p hunteval-reporting --test knowledge_reporting
cargo test -p hunteval-cli --test r7
```

The initial pure Python package has no runtime dependency. Its exact local commands are:

```bash
PYTHONPATH=sdk/python/src python3 -m unittest discover -s sdk/python/tests -v
python3 -m compileall -q sdk/python/src sdk/python/tests
python3 -m pip wheel --disable-pip-version-check --no-deps --no-build-isolation --wheel-dir <new-directory> ./sdk/python
python3 scripts/ci/check-python-wheel.py <first-wheel> <second-wheel>
```

The wheel check compares normalized archive contents rather than ZIP timestamps and rejects unexpected paths, caches, or byte differences between repeated builds.

R7-14 must add:

```bash
./scripts/ci/r7-extensions.sh
```

R7-15 must additionally execute the non-publishing release-candidate procedure from `RELEASE_CHECKLIST.md` on a clean tree and require all canonical GitHub Actions jobs, including `Knowledge and extensions`, on the exact evidence revision. Exact target names may change only in the milestone that creates them and must be updated here in the same change. No milestone is complete while a required local or remote gate fails.

## 19. Known limitations retained through R7

- Analytical search is local, deterministic, structured, and lexical/field based; it is not an embedding service, knowledge graph, or general question-answering model.
- Search can expose only verified public observations and already-supported claims. It does not infer missing metrics, hidden reasoning, causality, experimental validation, approval, or transferability.
- Evaluator analytical corpora are never deployment-visible. Deployment-visible knowledge remains separately authored, explicitly authorized, optional, untrusted, and disabled by default.
- Initial extension contracts cover supervised local process adapters. They do not provide a stable Rust ABI, in-process plugins, production SIEM execution, unrestricted network, or hosted extension registry.
- A passing conformance suite proves behavior only for the declared adapter bytes, policy, fixtures, and supported versions.
- Verified provider cost remains unavailable without an independently verifiable adapter and cannot be replaced with self-reporting.
- The Python SDK is a contract client and deployment-peer toolkit, not a runner, evaluator, scorer, verifier authority, provider integration, or agent framework.
- Package publication and signing remain governed release actions; R7 requires reproducible non-publishing package evidence.
- No production SIEM connector, distributed storage/execution, web dashboard, Kubernetes deployment, autonomous prompt optimization/adoption, or private chain-of-thought collection is introduced.
