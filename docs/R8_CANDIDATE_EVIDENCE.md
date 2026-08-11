# R8 candidate evidence

## Scope

This record captures verified implementation and native-package evidence for candidate revision `47cf61de0ad2845b4af789a51ce409457f917bed` and immutable tag `v0.8.0-rc.5`. It is not `R8_COMPLETION_EVIDENCE.md`, an independent security review, an independent reproducibility review, a production release, or a v1.0 publication decision.

Roadmap governance keeps v0.7.1 and v0.7.2 pending under their own release gates and excludes their pending interfaces from the stable v1.0 freeze set. Their incompleteness therefore no longer blocks closure of the R8 core candidate. This decision does not change their support status or authorize live commercial execution.

## Protected checks

Pull-request run [31520408340](https://github.com/ns2switch/HuntEval/actions/runs/31520408340) passed all 17 required jobs on the exact candidate revision: Policy, Quality, Tests, Security, Adversarial protocol, End-to-end, Documentation, Benchmark science, Evidence-backed diagnosis, Controlled improvement, Knowledge and extensions, Framework connectors, Upstream framework conformance, Commercial connector replay, R8 compatibility, R8 supply chain, and Package.

The End-to-end job ran the 108-cell official cloud matrix, verified normalized JSON and HTML reports, verified every run, generated topology and diagnosis artifacts, and completed the generated-artifact secret scan. It ran on the same source revision as the native candidate matrix but remains CI implementation evidence rather than an independent clean-room reproduction.

## Native non-publishing rehearsal

Release-candidate run [31522997192](https://github.com/ns2switch/HuntEval/actions/runs/31522997192) passed all four native jobs from tag `v0.8.0-rc.5`. Each job built with locked dependencies and Rust 1.93.1, packaged on the declared native runner, scanned the public package, constructed supply-chain evidence, built and inspected the Python wheel, installed into a clean destination, smoke-tested the CLI, generated checksums, signed the root manifest with an ephemeral rehearsal identity, verified that signature offline, and uploaded bounded evidence. `production_published` is `false` for every target.

| Target | Runner | Job | Release manifest SHA-256 | Native evidence SHA-256 | Checksum inventory SHA-256 |
|---|---|---|---|---|---|
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | [93884455109](https://github.com/ns2switch/HuntEval/actions/runs/31522997192/job/93884455109) | `718af6e1733f9288794920fcf712c6fe2d27c2083a23cde7c0c2c8e750ad1a91` | `78027874dfec55ba492a9b4abecc79a73e5cc926514b6807cffb2d9685b28ab3` | `417575f9151380abb621fad98ae8ab54076bf64c41642ca62f5a32012183f8e9` |
| `x86_64-apple-darwin` | `macos-15-intel` | [93884455258](https://github.com/ns2switch/HuntEval/actions/runs/31522997192/job/93884455258) | `dd991f244b4e91b0865d7aa498b461d2fa969f3bb6083253de53988d2c6d5611` | `8d371a4ae474be3e02160f0075bf23697f6720fce0cc7bdaba556e6996708605` | `42a5c6f172d25ccac2da8c160b1556255921ee1b65ba59374b975e9ed409ca79` |
| `aarch64-apple-darwin` | `macos-15` | [93884454975](https://github.com/ns2switch/HuntEval/actions/runs/31522997192/job/93884454975) | `6336b0a5ce082ea6bbe02a540e7458ce35df26212a03cf6e22a03723801f7690` | `c99f6e99c9aef5258eb56e8c89ab03bf5f4a8abf380c0b66754f393b5417b095` | `1dec5bc46a99fcd7d1e2d760a8b7b0a644872ad16da44ec8bf0ae35c2ef47dda` |
| `x86_64-pc-windows-msvc` | `windows-2022` | [93884455025](https://github.com/ns2switch/HuntEval/actions/runs/31522997192/job/93884455025) | `5a8c1a1523a16f59d9b16a9997f7cbb1655c899a26fd8f8311ca65c8e7e8314b` | `31b9416d0437853a3aba92257224fa4cf53292db5e8df525df4cc4189059b367` | `827e800f1588334b45b087024a312b4940c8ea2aa160124c82ad64746b971f6c` |

The downloaded artifacts were independently re-read during closure preparation. Every recorded root hash matched, every supply-chain bundle passed `r8_supply_chain.py verify`, and every detached signature passed `r8_sign.py verify` against repository `ns2switch/HuntEval`, ref `refs/tags/v0.8.0-rc.5`, workflow identity `r8-native-candidate`, and the candidate commit epoch.

## Remaining closure gates

- `R8_SECURITY_REVIEW.md` still requires a separately identified reviewer and exact findings disposition.
- `R8_REPRODUCIBILITY_REVIEW.md` still requires a separately identified reviewer to execute the documented empty-cache clean-room procedure.
- after those reviews, a new immutable evidence-bound candidate must pass on the final closure revision;
- only then may `R8_COMPLETION_EVIDENCE.md` be created and R8 be marked complete.

Failure of either review or of the final candidate rejects that candidate. Corrections require a reviewed commit and a new immutable tag; existing tags and artifacts are never replaced.
