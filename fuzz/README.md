# Protocol fuzzing

The fuzz package is intentionally outside the release workspace. It contains bounded targets for JSONL framing, protocol sessions, trajectory replay, and conformance transcript input. Corpus files are synthetic public inputs and must pass the repository secret scan.

CI uses `cargo-fuzz` 0.13.2 with `nightly-2026-02-12`. Run the same bounded smoke suite with `./scripts/ci/r3-adversarial.sh`; set `HUNTEVAL_SKIP_FUZZ_SMOKE=1` only for local stable-Rust iteration, never in the canonical CI job. Every discovered defect must be minimized and added to deterministic stable-Rust regression coverage before closure. Fuzz targets must not perform network, provider, filesystem, or ground-truth access.
