# R3 completion evidence

## Scope and revisions

R3/v0.3 runner and protocol hardening is complete. The implementation is provided by commits `f0f6119`, `dbfce2c`, and `2d34517`. Commit `2d3451742d3245b458da03417b1361b574098389` is the implementation evidence revision used for the artifact records below. This closure does not reopen R2, alter its recorded evidence, or remove the R2.4 external-enforcement caveat.

## Quality and remote evidence

The following commands passed locally on the evidence revision:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/e2e.sh
./scripts/ci/release-candidate.sh <new-absolute-output-directory>
git diff --check
```

GitHub Actions run [31305219082](https://github.com/ns2switch/HuntEval/actions/runs/31305219082) passed all eight jobs on `2d34517`: Policy, Quality, Tests, Security, Adversarial protocol, End-to-end, Documentation, and Package. The workflow used the pinned `ubuntu-22.04` runner, Rust `1.93.1`, nightly `2026-02-12` for bounded fuzz smoke tests, and the `linux_bubblewrap` backend. The same backend contract passed locally on x86_64 Linux with Bubblewrap `0.11.1`; the command construction retains compatibility with the Bubblewrap `0.6.1` baseline supplied by Ubuntu 22.04.

The local adversarial command above ran deterministic properties, compatibility fixtures, conformance tests, and the retained corpus. The passing remote Adversarial protocol job additionally ran all four bounded `cargo-fuzz` targets with `cargo-fuzz 0.13.2` and the pinned nightly toolchain; this host has no `rustup` installation and therefore made no local full-fuzz claim.

## Contract, sandbox, and protocol evidence

- representative execution-policy SHA-256, identical across all 36 runs: `40cb5bc887c153f9dbcf753a06a0b92a19c04f99176ae0b0955f49065ed9e439`;
- local Bubblewrap executable SHA-256: `0abea81db798ebf6b4742ac0664802d97521547a353c2a0dbdc21d76cbbfd2c0`;
- local resource launcher (`prlimit`) SHA-256: `00be18793391b9222a041277a088022cc58088690cb0e851383bd8ec73f0fefb`;
- system capability-result SHA-256: `72307fcc1861192bebeafd3e61314f2169123b24c118546c1b3aa2d94f4b9d9f`;
- protocol compatibility index SHA-256: `64bfa2a339f846f69630707c24ac32f389fcb2c1b69512a2a6259775181401f7`;
- supervisor-worker conformance-result SHA-256: `1d13844cfa2d582ee4ee880f6ffe9d499e7b8cc8152907c28d8fe761e8c8571b`;
- conformant transcript SHA-256: `e9a994b83fc68d48e5b41ac28d328bfd5b7eea22f52cf4d2b995e3357bb9d42d`.

The retained public fuzz corpus hashes are:

| Target | Corpus entry | SHA-256 |
|---|---|---|
| `conformance_input` | `empty-transcript` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `jsonl_decoder` | `malformed-json` | `8c69fc307fed3936d6a8ac679c0079c9bfd11f9de2a43e20ae25ff2a899d9776` |
| `protocol_session` | `empty-session` | `37517e5f3dc66819f61f5a7bb8ace1921282415f10551d2defa5c3eb0985b570` |
| `trajectory_replay` | `partial-event` | `ad523e3e244bbf8d140d04337ace2dc2c4e42f9c679c7b065cf698db47b181c8` |

## Benchmark and verification evidence

- authored benchmark manifest SHA-256: `3151f298367797e6e65f5091a57d7b5727ff74fcf5cbfc97cc42eea8a14ab2fb`;
- resolved benchmark definition SHA-256: `8246f96cca0b4a1ab5b805d4df56b56228bdc98840c3cfb01e8401f05bac42c9`;
- single-agent deployment configuration SHA-256: `ee34f8e73510ad4747603ee4e896604b6d94495037b4abb838319ee85beec614`;
- two-agent deployment configuration SHA-256: `356f80b142939b9c77c1ada1eb434e1976315eff81086d17e8d424ea22f9484a`;
- scoring-profile SHA-256: `9f40e2d6b211f08c98c4219e293475643c557cedd04584547d11887802651ef7`;
- debug runner SHA-256: `24bb77acfc9bd414a5b5a7d617da4809ad8c6e63af655b232dee11335cbddede`;
- debug managed-worker SHA-256: `05e87abe1e16c34fb56e7619c81cfa228ad03bd957f49496ddccb516bd9ef270`;
- normalized benchmark-result SHA-256: `8f0f333a51979a665681638bb18bee8045c019580e4fad3d839c52c76e1605ca`;
- run-verification JSONL SHA-256: `16d3a99602715bb86ac77a229e500abf6dfbf23b5e0b791ae42b6e0ad5d96691`;
- report-verification result SHA-256: `d48e20706df6a5c270887c06dc6c4ca9012feba5568c4cbd8bbd08135d8b7ffa`.

The generated R2 evidence object has SHA-256 `62eaa25c2628c224f01977e3d7ff6b2c748e76bd693a2da81b0c2c58e2f69d7f` and records these exact episode package hashes:

| Episode | SHA-256 |
|---|---|
| `aws-iam-001` | `359cc68a6eeef450e49eb052c07a167f8e99516d60a95a4f439157e78daf3883` |
| `aws-iam-002` | `12cd1c75a00b9b79311cacfc995fd84f2553249e07f8ccd7c859e1e1d3bbcd31` |
| `aws-iam-003` | `ba67740b5a9cf8f0d5cdec68bab9960f0b29b6ef172468a149b7d74522f69d3f` |
| `azure-iam-001` | `27c953f0ec235e6756835bfcf3aca908dc6133b4ba819fdc059ccfa5ba1a66bb` |
| `azure-iam-002` | `37cd2bd9ef6563719a84d35e03fa0f10fd0834780179806cbbe128e8be6780af` |
| `azure-iam-003` | `5f5911fae0c6fbf57bcf61fdd4b49a81888b94797376aff6a51f48ae78a29e14` |
| `gcp-iam-001` | `2d8b8ab81e3161b3e7f8ad60e96e3d720da5b635946a3d9c2b149363b5ea0599` |
| `gcp-iam-002` | `755aa79acbefefedc3c1361059bf796d714c52e6cf2c5e31484965cee9dcb20c` |
| `gcp-iam-003` | `94810ccc519e3927eb2eb0a40a07de7465bff970ab333951fb429df2e637947f` |

This object is part of the verified E2E artifact set and does not alter the existing R2 completion record.

## Secret scan and package evidence

- secret-scan policy implementation SHA-256: `23028ceca729d235fae00802aebd79eb434df1b5eda6bc6738fbc846671ecbef`;
- generated E2E secret-scan result SHA-256: `01965ff2633a4b9e149de9ee7873e4d0d603f2b23f9a60f3d079a00029310b2f`;
- release secret-scan result SHA-256: `a5a447d319c7336a130795b285da957e645abb978343779eaa7ecb9fdc4dbd48`;
- non-publishing release archive SHA-256: `0271a8f399811e3b0e2a72b1b5261f0be3700d98256af1b3233479d97224636c`.

Both scan results are schema 0.5 `clean` results with no findings or incomplete reasons. The release candidate is a dry run only; no production release was published.

## Architecture decisions and limitations

ADR-047 through ADR-052 are accepted. No other ADR status changed during closure.

Known limitations remain explicit:

- only the fail-closed `linux_bubblewrap` backend is supported for scored R3 execution;
- network access remains denied and no production SIEM connector is included;
- public run verification does not recompute evaluator-only private metrics;
- attribution remains observational and does not establish causality;
- statistical intervals describe only the configured benchmark repetitions;
- R4.4 topology benchmarking and R6.4 prompt improvement analysis remain future work and are not implemented by R3.

R4 is the next implementation milestone.
