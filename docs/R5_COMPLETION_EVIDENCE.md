# R5 completion evidence

## Scope and revisions

R5/v0.5 evidence-backed diagnosis is complete. Commit `e22e71babb3911db90b93f6ad82918664704aeb6` is the evidence revision used for the local and remote records below. This closure does not reopen or alter R2, R3, or R4 completion evidence. The former R2.4 external-enforcement caveat remains closed by its separate administrator attestation.

## Quality and remote evidence

The following commands passed locally on the evidence revision:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
HUNTEVAL_E2E_OUTPUT=/tmp/hunteval-r5-evidence-e22e71b ./scripts/ci/e2e.sh
./scripts/ci/release-candidate.sh /tmp/hunteval-r5-rc-e22e71b
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
git diff --check
```

The dedicated R5 gate was also verified after removing its managed DuckDB worker executable: its explicit workspace-binary build restored the worker before the CLI diagnosis test. This prevents a warm target directory from becoming an undeclared prerequisite.

GitHub Actions run [31343374320](https://github.com/ns2switch/HuntEval/actions/runs/31343374320) passed all ten jobs on `e22e71b`: Policy, Quality, Tests, Security, Adversarial protocol, End-to-end, Documentation, Benchmark science, Evidence-backed diagnosis, and Package. The workflow used the pinned `ubuntu-22.04` runner, Rust `1.93.1`, nightly `2026-02-12` for bounded fuzz smoke tests, and the fail-closed `linux_bubblewrap` backend.

## Contract, taxonomy, and classification evidence

Schema 0.7 adds eleven diagnosis contracts without rewriting schema 0.3 through 0.6 artifacts. The SHA-256 of the sorted schema-file checksum manifest is `1871defe5944ec8c0caa50214dc31973f18c4c85893fb0a5d360f9d952dd8a38`. The corresponding canonical-example checksum-manifest SHA-256 is `114a24f6bac25104c8d6803d9408c9a1be94f1de6f5334ed94ef77489e2d53da`.

The normative six-category taxonomy has SHA-256 `097cc8a64e8df0fcb7b10b79d8786dc87062fb89c8f05b04008e20d04d577bd9`. Tests prove exact parity with the compiled registry, bounded inputs, deterministic rule evaluation, typed source resolution, private-field rejection, and omission when required evidence is absent. Evidence sufficiency remains deterministic and non-probabilistic; only an eligible controlled experiment can support a controlled contribution result.

## End-to-end diagnostic evidence

The E2E matrix completed all 108 cells with zero failed, pending, or non-comparable cells. Offline run verification accepted all 108 runs. The diagnostic verifier accepted 1,302 content-addressed artifacts with no reasons for rejection.

The principal normalized artifact hashes are:

| Artifact | SHA-256 |
|---|---|
| benchmark report | `13889ac3bd3129138486ca8afa9ab5cc45661c31d00aa28c6b0b792f77242ca8` |
| benchmark diagnostic report | `39afc036e469d19eb034e1b30566002294bede127268a9b67d022d4df13d3055` |
| diagnostic recurrence | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| diagnostic bundle manifest | `6e6eeafb4f5cce59419f4d8ef1c5fe1ef92b8379c62db5612dd73e3d78f4cd87` |
| diagnostic verification result | `dc2ca72ef1a0180eae95b94789b5698cba6afea7c333654348b8798c1f67cbbc` |
| generated-artifact secret scan | `c5005475584b6b583076522de5bbf64015aa69e2197cacdb6f5f41e6dfdd322b` |

A representative content-addressed run diagnosis has SHA-256 `5ab43dc147d0f5b100aad4dfa25f1b3e5330a2a1e3790af09333f4ee4e11dbac`; its paired bottleneck analysis has SHA-256 `f3251b190f550d29d08b3caaa74d7867a188c0b64daf94bdacbc738e1e2dbd83`. Bundle paths use identifier digests rather than raw opaque identifiers, so GitHub and NTFS-compatible artifact transport does not change the identifiers retained inside normative JSON.

The reports keep observations, classifications, unvalidated hypotheses, controlled experiment results, and approved-change availability distinct. Unsupported contribution results remain unavailable rather than inferred. Any controlled contribution remains explicitly experimental and topology-dependent. Static HTML is escaped and script-free, and verification fails closed for missing, changed, oversized, symlinked, or concurrently replaced sources.

## Secret scan and package evidence

The generated-artifact scan is schema 0.5 `clean`, covers 2,078 artifacts, and has no findings or incomplete reasons. The non-publishing release archive has SHA-256 `57eea29ae9eaf11f8e765ee6a97e371c6a4aef890a85ed53a73188e0995c5654`; its secret-scan result has SHA-256 `c58e9447d7c124a86b341b1b06eb27082bc0defd1bd56038778bc8e0f1beb8ba`. The release scan is `clean` over 62 files.

Release metadata records revision `e22e71babb3911db90b93f6ad82918664704aeb6`, Rust `1.93.1`, target `x86_64-unknown-linux-gnu`, and `production_release_published=false`. Both E2E and release `SHA256SUMS` files verified independently.

## Architecture decisions and limitations

ADR-060 through ADR-066 are accepted. No earlier ADR status changed during R5 closure.

Known limitations remain explicit:

- diagnosis is limited to registered deterministic rules and observable structured inputs; absence of a classification does not prove absence of a weakness;
- recurrence reports observable repetition, not root cause or hidden intent;
- contribution requires an eligible controlled topology experiment and is never generalized beyond that topology and artifact set;
- timing metrics remain unavailable for legacy traces without sufficient runner-authoritative lifecycle events;
- verified provider cost remains unavailable without a verifiable adapter;
- R5 does not generate or validate deployment changes, expose hidden-test feedback, or approve or adopt recommendations;
- no production SIEM connector, unrestricted network access, distributed execution, web dashboard, or autonomous optimization is introduced.

R6 is the next implementation milestone.
