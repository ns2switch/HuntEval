# R8 benchmark corpus expansion plan

## Scope and interface decision

This change expands benchmark content within the existing HuntEval architecture. Protocol 0.3, episode schemas, telemetry-table schemas, topology and deployment contracts, metric definitions, scoring and statistical semantics, report semantics, and ground-truth isolation remain unchanged. The existing generic cloud control-plane telemetry fields are sufficient for the proposed scenarios.

One frozen-contract blocker was found. `schemas/v1.0/official-benchmark-pack.schema.json` requires `episode_count` to equal 18. A 54-episode official pack cannot conform to that schema. Promotion of the expanded manifest to the official pack is therefore stopped pending review of the smallest additive remedy: introduce a new content-addressed pack-contract revision that preserves the old 18-episode candidate as historical evidence and permits the new exact count. The benchmark-pack schema is not silently changed by this work.

## BC-00 baseline audit

The preserved baseline contains 18 deterministic synthetic episodes: six AWS, six Azure, and six Google Cloud episodes. It occupies 135,725 logical file bytes across 118 files. Episodes `001` through `003` predate public R4 classification artifacts. Episodes `004` through `006` add one benign investigation, one multi-stage investigation, and one cross-boundary investigation per provider and have the existing nine content-addressed independent approvals.

The machine-readable inventories are:

- `examples/benchmark-corpus-inventory.json`: deployment-safe corpus metadata;
- `doc_interna/BENCHMARK_CORPUS_BASELINE_INVENTORY.json`: evaluator-only baseline details;
- `doc_interna/BENCHMARK_CORPUS_INTERNAL_MATRIX.json`: evaluator-only expanded coverage and private aggregate inputs.

The baseline gaps are service diversity beyond identity, multiple independent benign alternatives, longer attack paths, broader cross-boundary behavior, bounded behavioral noise, and enough differentiated episodes to exercise topology comparisons. No existing episode is renamed or regenerated.

## BC-01 scenario catalog

Each provider receives exactly 12 new episodes, numbered `007` through `018`. Provider-specific actions and service names are used while the investigation structure remains paired across providers.

| Number | Family | Outcome | Difficulty | Volume | Stages | Cross-boundary | AWS focus | Azure focus | Google Cloud focus |
|---|---|---|---|---|---:|---|---|---|---|
| 007 | administrative activity review | benign | introductory | small | 0 | no | IAM/STS administration | Entra ID/RBAC administration | IAM/service-account administration |
| 008 | automation activity review | benign | introductory | small | 0 | yes | authorized cross-account automation | authorized cross-subscription automation | authorized cross-project automation |
| 009 | credential activity review | malicious | introductory | small | 2 | no | credential use | sign-in/token use | service-account impersonation |
| 010 | permission change review | malicious | introductory | small | 2 | no | role policy changes | role assignment changes | IAM policy changes |
| 011 | credential persistence review | malicious | intermediate | medium | 4 | no | access key/trust persistence | service-principal credential persistence | service-account key persistence |
| 012 | boundary role activity | malicious | advanced | large | 5 | yes | account role chaining | subscription/managed-identity transition | project impersonation transition |
| 013 | boundary data activity | malicious | intermediate | large | 5 | yes | STS, S3, and KMS | managed identity, Storage, and Key Vault | impersonation, Storage, and KMS |
| 014 | secret access review | malicious | intermediate | medium | 4 | no | Secrets Manager | Key Vault secrets | Secret Manager |
| 015 | key usage review | malicious | intermediate | medium | 4 | no | KMS | Key Vault keys | Cloud KMS |
| 016 | storage access review | malicious | intermediate | medium | 5 | no | S3 | Storage Accounts | Cloud Storage |
| 017 | serverless control review | malicious | intermediate | large | 5 | no | Lambda | Azure Functions | Cloud Functions/Cloud Run |
| 018 | container control review | malicious | advanced | large | 6 | yes | EKS | AKS | GKE |

Small, medium, and large fixtures contain 16, 28, and 40 events respectively. These are bounded reviewed-input tiers, not user-selectable unbounded generation. Noise is meaningful provider control-plane activity and includes plausible authorized confounders.

## Expected and generated coverage

The generated candidate has 54 episodes and exactly 18 episodes per provider. Its evaluator-only aggregate contains nine benign episodes, 30 multi-stage episodes, and 16 episodes with activity in more than one administrative scope. The classified subset has 12 introductory, 21 intermediate, and 12 advanced episodes; difficulty remains unavailable rather than inferred for the nine pre-R4 episodes. Among the 45 classified episodes, the distribution is 26.7% introductory, 46.7% intermediate, and 26.7% advanced.

The expansion adds Secrets Manager/Secret Manager/Key Vault, KMS/key operations, object storage, serverless control planes, and managed container control planes for every provider. Family overlap is intentional where an attack path crosses identity, boundary, and data services, but each numbered family has an independent behavioral purpose.

## Determinism, provenance, and recovery

All new content is first-party, synthetic, offline, and generated without cloud credentials or live APIs. Each package records generator version, fixed generation seed, provider, scenario family, volume tier, toolchain, source-template identity, and public content hashes. Private provenance separately binds ground truth and reference queries.

Generation uses fixed February 2026 timestamps, stable event and entity identifiers, a pinned Parquet writer configuration, deterministic ordering, and no wall-clock input. Private reference queries operate only on the public Parquet table and must recover exactly the evaluator-private malicious event set; benign queries recover an empty set.

## Security and review

Public manifests, classifications, and provenance are scanned for exact private event identifiers, entity identifiers, ATT&CK mappings, evaluator paths, reference-query markers, and ground-truth markers. Telemetry is not scanned for the investigated identifiers because the investigation evidence must exist there; it is instead checked for the absence of answer labels and private artifacts. Tree walking, bounded reads, regular-file requirements, symlink rejection, path validation, package schemas, secret scanning, and content hashing remain fail closed.

Every new package contains a content-addressed review bundle covering public telemetry and metadata, deterministic source input, private ground truth, private reference query, provenance, validation result, and review policy inputs. No `private/review.json` is generated. All 36 new episodes remain pending independent human review and are not release-eligible. A change to public telemetry, private truth, reference query, or policy invalidates a later approval under the existing review contract.

## Repository and CI impact

The 36 new packages add 708,378 logical bytes across 360 files; the complete dataset tree is 844,103 logical bytes across 478 files. This favors deterministic checked-in fixtures over massive telemetry and does not require larger package or query bounds. Runtime measurements for the 324-cell matrix and all canonical gates are recorded in the final implementation summary; any job-specific timeout proposal must be based on those measurements.

The local 324-cell execution completed in 845.65 seconds with two jobs, zero failed, pending, or non-comparable cells, and 21,240 KiB maximum resident memory reported for the controller. Report verification checked 327 artifacts. Diagnostic verification checked 3,894 artifacts. The diagnostic inventory was measured at 1,065,207 bytes, exposing an implementation capacity defect in its 1 MiB reader bound; the reader is raised narrowly to 2 MiB and retains fail-closed rejection above that limit. The owning R8 corpus CI job keeps a 60-minute bound, so no timeout increase is required.

The quality analysis reports five distinct reference-outcome signatures and flags broad identical-result groups. In particular, the frozen scripted reference deployments do not recover the new malicious evidence and therefore produce one common raw-metric outcome signature for those episodes. This is a review finding, not a reason to tune the baselines with private answers or automatically delete episodes. Scenario content, exact reference recovery, and behavior under independently capable deployments require individual review before promotion.

## Promotion sequence

1. Validate deterministic generation, schemas, recovery, leakage, bounds, and stale-review behavior.
2. Obtain independent human review for every new content-addressed bundle.
3. Review and approve the additive official-pack contract remedy described above.
4. Create a new immutable official pack identity and preserve the previous 18-episode candidate evidence.
5. Execute all 324 paired cells, verify reports and runs, and produce corpus-quality analysis.
6. Run the complete canonical and R8 gates in a clean environment.

R8 remains in progress until these corpus gates and all pre-existing independent security, reproducibility, native-workflow, and release-candidate evidence pass.

## Required follow-up governance

Promotion requires an ADR-104 addendum (or a narrowly scoped successor ADR) defining the additive official-pack contract revision, identity, compatibility, migration, and rollback behavior. The R8-08 implementation record must then bind the 54 exact memberships, 36 human review records, scoring profile, deployments, seeds, benchmark manifest, quality report, and clean-room evidence. The release interface inventory, migration inventory, official-pack fixture, interface-freeze digest, R8 evidence index, and candidate packaging references must be updated together on one reviewed exact revision. None of those future evidence updates may overwrite the historical 18-episode candidate or its completed R2–R7 evidence.
