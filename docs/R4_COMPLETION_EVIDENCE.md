# R4 completion evidence

## Scope and revisions

R4/v0.4 benchmark science and dataset quality is complete. Commit `f9559a6956b472fcb49494e3d7471a6adbc5a06c` is the implementation evidence revision used for the artifact records below. This closure does not reopen R2 or R3, alter their recorded evidence, or remove the R2.4 external-enforcement caveat.

## Quality and remote evidence

The following commands passed locally on the evidence revision:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/e2e.sh /tmp/hunteval-r4-evidence-f9559a6
./scripts/ci/release-candidate.sh /tmp/hunteval-r4-rc-f9559a6
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```

GitHub Actions run [31321445726](https://github.com/ns2switch/HuntEval/actions/runs/31321445726) passed all nine jobs on `f9559a6`: Policy, Quality, Tests, Security, Adversarial protocol, End-to-end, Documentation, Benchmark science, and Package. The workflow used the pinned `ubuntu-22.04` runner, Rust `1.93.1`, nightly `2026-02-12` for bounded fuzz smoke tests, and the fail-closed `linux_bubblewrap` backend.

## Dataset and policy evidence

The authored benchmark manifest has SHA-256 `28fb7a122d8ae7aae3184eb2674d0580f0c5fccfb4a7aaf023c7b2de12d3a2e1`. Its resolved definition has SHA-256 `79cbacef2c043485e158f292bc95698cad5361442ef2198c029fd6aac4956612` and identifies 18 exact episode package hashes, three deployment configurations, two seeds, and the scoring profile.

The independent dataset-review policy has SHA-256 `d7b32ea129bf68b65fa612c906eb17cd3b6d97d48869a8561eec96af5bc60a0f`. The approved R4 review records bind the exact public package, private ground truth, reference query, and review-policy bytes:

| Episode | Review-record SHA-256 |
|---|---|
| `aws-iam-004` | `e66fbaf28313e974016c721eada30b226072d803ee7006e1f941aac9187c2fea` |
| `aws-iam-005` | `31d1ff5df458a110168ceb828fef49c76a5f4ca8006585108e5a2507bea34a5a` |
| `aws-iam-006` | `2616ba9dab7c8c022aebd9de2e4167acca443f8e0e7f19b4cc8a9d79c3fd29c9` |
| `azure-iam-004` | `9a06e39f5160aa12245bcf61f99aafd68f4fda5a1eb3ef13f561ac011585d0f1` |
| `azure-iam-005` | `e5ca84e784c865339391fe740b9405a86bef993db76cfdf1118bb27e56344d0e` |
| `azure-iam-006` | `cfb54e2874380265721726fdc65f6010c1653fda8cb16ca6db2b589c9deba1df` |
| `gcp-iam-004` | `35217ef4d793945f2f5b2372e18764bd108d77becd7e4623d09102b6b767dcf5` |
| `gcp-iam-005` | `1873d006f27856acfab1ea37613ce652462ec28e6ed07c2243efdd03ba984934` |
| `gcp-iam-006` | `1f9111c70b0a2ca446d3b787b5d760d7177e8b6206fe9262b3ffcbee224d3a0c` |

The versioned statistical policy has SHA-256 `51c5e5a0691754bff1dbca5172f7d8d13bb82aa2677f4e8e5d968bc8a87ef982`. The reporting scoring-profile bytes have SHA-256 `f49e1c009ca684ccd54336fd7c40d1ce7df4a2d52ba6ef65a4afd20556476f1d`; the resolved benchmark preserves the immutable normalized scoring-profile digest `9f40e2d6b211f08c98c4219e293475643c557cedd04584547d11887802651ef7`.

## Topology and comparison evidence

The normative reference topology hashes are:

| Deployment | Topology SHA-256 |
|---|---|
| `single-agent-scripted` | `7ea7d5d34834d637bde37daf1cc0ac14d81cddfeb0f49edf25360607f574531e` |
| `two-agent-scripted` | `d09ed0d9e0ed4c0bb184384e01f140d9b9c1abb63b3e9bdabc04f96485e18081` |
| `supervisor-specialist-scripted` | `999c40b7965e20e69f5376f2e496ffdfe2db3874662d9852c773226ce4cd8de1` |

The E2E matrix completed all 108 cells with zero failed, pending, or non-comparable cells. Its normalized benchmark report has SHA-256 `250ca835fcb938a7b3bdfabb66adf2a4203f8cdcd9add678078b9828b65c062e`.

The controlled single-agent to supervisor-specialist comparison records every topology field that changed and exact hashes for binaries, budgets, episodes, execution policy, managed-tool policy, models, schemas, scoring profile, and seeds. Its evidence hashes are:

- controlled experiment: `43ca13a10725ddfc1ec35679eaa2bc1e05084b7032a5791f853a46610dbd53ad`;
- topology observations: `0ba2f99fc8bbb6fc845c9d50d611eae6309f6bdac095f71a1aea721c927709c6`;
- normalized topology report: `c713d5fc0ce5070d519c6ab8cccff862cb42f05361611c82759e78b70e30a791`.

The report preserves the raw metric vector and contains no aggregate score. All four comparisons remain descriptive and non-conclusive because adjusted per-comparison evidence for the declared Holm-Bonferroni family is unavailable. Role contribution and verified cost remain explicitly unavailable rather than being inferred. Every topology contribution statement is labeled experimental and topology-dependent.

## Verification, secret scan, and package evidence

- E2E artifact manifest SHA-256: `e4d396b6c82af530a8faaa166c2f2f2144e24b9a9a25738f760bc03bdac3aa9e`;
- benchmark verification result SHA-256: `8e346dae49c152206aebcd9387deb574d35314956f1dfc3ce1af1e9fc08be7cb`;
- generated-artifact secret-scan result SHA-256: `797dd285d5fc41346ccaa82bb148a870b417cf213ae88dd1c9f4e8877b1d7890`;
- non-publishing release archive SHA-256: `7c1224d0fec8349b81e8755fe5d203950a3e5295424f03cbbc0c7a3484aa0074`;
- release secret-scan result SHA-256: `fb49f11544fcf632a84d0e0f5d5ac3d75fdb4f691ac1bc756db0340a80935ee5`.

The benchmark verifier accepted all 111 referenced artifacts. The generated-artifact scan is schema 0.5 `clean`, covers 774 artifacts, and has no findings or incomplete reasons. The release scan is also `clean`. Release metadata records revision `f9559a6956b472fcb49494e3d7471a6adbc5a06c`, Rust `1.93.1`, target `x86_64-unknown-linux-gnu`, and `production_release_published=false`.

## Architecture decisions and limitations

ADR-053 through ADR-059 are accepted. No earlier ADR status changed during closure.

Known limitations remain explicit:

- topology contribution estimates are experimental and topology-dependent; observational traces alone do not establish causality;
- the current multiplicity policy keeps multi-comparison families descriptive until adjusted per-comparison inference is available;
- verified cost remains unavailable when a deployment supplies no verifiable cost adapter;
- the cloud fixtures and reference deployments are deterministic synthetic evaluation assets, not production SIEM integrations;
- only the fail-closed `linux_bubblewrap` backend is supported for scored execution, with network access denied;
- R2.4 remains externally pending until an authorized administrator attests live branch and protected-tag enforcement;
- autonomous prompt optimization is not implemented.

R5 is the next implementation milestone.
