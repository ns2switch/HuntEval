# R7 completion evidence

## Status

R7/v0.7 knowledge and extension interfaces are complete. Implementation revision `ec18b3e949168f54f3a56818250a8ccad514f0a2` delivers the R7 contracts and runtime behavior. CI correction revisions `26d561533bd1ac12c2c6c789d8efc9e5d0915735` and `bf4fefb8e0b86e909acef090e6139ed580372dd0` provision the required sandbox and Python runtime and allow the unchanged R6 gate to complete from a cold cache. Closure revision `e206209661e4fcfcbf7560d9003a6d8025c175c9` passed all twelve canonical GitHub Actions jobs in run [31417962254](https://github.com/ns2switch/HuntEval/actions/runs/31417962254).

R2 through R6 remain complete with their recorded evidence. Their commit references, schemas, security posture, external-enforcement evidence, and exit criteria are unchanged.

## Implemented behavior

- additive schema 0.9 contracts and canonical examples preserve schemas 0.3 through 0.8 and protocol 0.3;
- evaluator-only analytical corpora remain separate from deployment-visible knowledge;
- corpus sources are root-confined, no-follow, bounded, content-addressed, independently revalidated, and rejected on private fields or digest drift;
- deterministic local indexes and bounded queries return exact source identity, kind, artifact hash, normalized field path, and excerpt citations;
- retrieval audits are append-only, hash-linked, replayable, and include measured latency and explicit resource/cost provenance;
- normalized JSON is authoritative and static HTML is escaped and script-free;
- managed-tool and deployment adapters use versioned out-of-process contracts, exact executable hashes, runner-owned deny-by-default capability resolution, denied network, bounded resources, and supervised lifecycle handling;
- conformance covers a reference deployment and managed tool plus timeout, crash, malformed output, correlation, digest, and transcript failures;
- the pure Python SDK provides strict schema 0.9 models, content-addressed public readers, protocol 0.3 registration/terminal handling, and reproducible wheel contents;
- the CrewAI connector supports single- and multi-agent crews, observable task delegation, runner-mediated scored tools, strict correlation, and structured final submissions without introducing a Rust-core or provider dependency;
- `scripts/ci/r7-extensions.sh` and the required `Knowledge and extensions` GitHub Actions job provide the dedicated R7 gate.

## Local evidence

Toolchain: `rustc 1.93.1 (01f6ddf75 2026-02-11)`.

The following gates passed before closure:

```text
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/e2e.sh
```

The dedicated R7 gate covers schema/domain compatibility, deterministic search, authorization isolation, source tampering, private-field rejection, audit replay, extension policy, both positive adapter paths, managed-tool process failures, safe reporting, CLI integration, ten Python SDK/connector tests, compilation, and repeated equivalent wheel contents.

Content identities at the governance revision:

| Artifact set | SHA-256 |
|---|---|
| Sorted schema 0.9 file-hash inventory | `07aadb35dc63ae567c977bb60241e83df274ef4e7a582cbea47e1a8b6651c439` |
| Sorted schema 0.9 and canonical-example file-hash inventory | `5a7206beeedbe08caa17636ec909f06441f03606f6ba3a8f51e0b518add03f63` |
| Dedicated R7 gate | `033be42bfc8784ed037675cc973137a5a8083fba018855cadf87a5d75dc4b3a5` |

The clean non-publishing package dry run at `26d561533bd1ac12c2c6c789d8efc9e5d0915735` packaged five binaries, schemas 0.3 through 0.9, taxonomies, public documentation, and the Python SDK. Package-local secret scanning and `SHA256SUMS` verification passed.

| Package evidence | SHA-256 |
|---|---|
| `hunteval-rc-26d561533bd1-x86_64-unknown-linux-gnu.tar.gz` | `933ab5d9ab910bed3231e639980c66ba78726c75847b39b37a39656d3d8ef22b` |
| `hunteval_sdk-0.1.0-py3-none-any.whl` | `0e8296dd9f0048cd89b81d67f3e927d1532e4527243e05e6632a4cf9ddb30cff` |
| `secret-scan.json` | `d02cffcebeb71393ec5b86bfa83c9bc69e40a2d5404d65f58d9261d8deb7e065` |
| `SHA256SUMS` | `0c3dcc95ec4f166313ced3d294117f22405bcc198cb3442e8039b87728c53a88` |
| `verification.txt` | `a842e599e856152d00a63aabf4b53e8067a5601831da2b491734b7d2d2e31bea` |

The first remote R7 run correctly failed closed because the new job had not installed Bubblewrap and the package job used an older runner Python. Revision `26d5615` installs Bubblewrap only where sandbox conformance requires it and pins Python 3.11 through the official setup action in both R7 and package jobs. The next run verified both corrections but exposed that a cold R6 gate exceeded its historical 45-minute limit while compiling `libduckdb`; revision `bf4fefb` raises only that job's limit to 60 minutes without changing or skipping any gate. A subsequent runner failed before checkout because GitHub certificate verification was temporarily unavailable. Closure revision `e206209` retriggered the unchanged tree: the cold-cache timeout correction completed successfully, the certificate failure did not recur, and all twelve canonical jobs passed without weakening TLS, tests, or fallback behavior.

Live `main` protection requires all twelve canonical checks with strict up-to-date-branch enforcement. The committed live-settings verifier passed against that protection and the existing restricted-creation and non-bypassable immutable `v*` tag rulesets.

## Known limitations

- analytical search is local deterministic lexical/field retrieval, not semantic search or causal inference;
- citations are exact content-addressed normalized field references; unsupported specialized projections remain unavailable rather than inferred;
- evaluator analytical corpora are never deployment-visible;
- extensions are supervised local processes, not an in-process Rust ABI or hosted extension registry;
- conformance applies only to the exact executable, manifest, policy, fixtures, and supported versions tested;
- the Python SDK is a contract client and deployment peer, not an evaluator, scorer, runner, or provider integration;
- the CrewAI connector does not configure providers, grant direct tools, collect private reasoning, or expose production SIEM execution;
- package publication and signing remain release-governed actions;
- production SIEM scored execution, unrestricted network access, distributed execution, Kubernetes, a web dashboard, and autonomous prompt adoption remain out of scope.
