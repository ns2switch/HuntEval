# R8-00 implementation evidence

## Status and scope

R8-00 is **in progress**. This evidence records the local interface-inventory and freeze-policy implementation; it is not R8-00 or R8 completion evidence.

On 2026-08-11, roadmap governance authorized this work while v0.7.1 and v0.7.2 remain pending. The authorization does not complete either pre-R8 milestone. Their pending framework, MCP, and commercial connector interfaces are excluded from the stable freeze set, and R8 closure remains subject to the dependency rule in `R8_IMPLEMENTATION_PLAN.md`.

## Implemented evidence

- `hunteval-release` provides infrastructure-independent, versioned inventory types, bounded validation, typed errors, deterministic ordering, SHA-256 identity, and freeze-manifest derivation.
- `schemas/v1.0/release-interface-inventory.schema.json` and `schemas/v1.0/interface-freeze-manifest.schema.json` define the public machine-readable shapes.
- `examples/contracts/v1.0/release-interface-inventory.json` records the baseline revision, ownership, stability, compatibility range, fixture, gate, projection, authority, trust boundary, bounds, parser status, precondition, and limitations for each inventoried surface.
- The expanded native-platform inventory derives freeze-manifest digest `002eb890ba8a28b1baef4a6b773c48a49f576582209e841881b12b23b3f4abf8`; macOS and Windows entries remain excluded previews.
- Stable-candidate and retained entries require satisfied preconditions, a public projection, documented bounds and parser behavior, a canonical fixture, and a verification gate.
- Preview, experimental, blocked, evaluator-private, unverified, duplicate, malformed, unsupported-version, traversal-path, and unexplained entries fail closed or remain explicitly excluded.
- ADR-098 through ADR-107 are accepted with the security, compatibility, review, publication, and rollback constraints required by R8.
- `scripts/ci/r8-compatibility.sh` and the `R8 compatibility` workflow job provide the focused repository gate.

## Local verification

The following focused checks pass in the implementation worktree:

```text
cargo test --locked -p hunteval-release
  4 passed; 0 failed
cargo clippy --locked -p hunteval-release --all-targets --all-features -- -D warnings
  passed
python3 scripts/ci/test-github-settings.py
  passed
cargo fmt --check
  passed
cargo clippy --workspace --all-targets --all-features -- -D warnings
  passed
CARGO_BUILD_JOBS=1 cargo test --workspace
  passed
./scripts/ci/quality.sh policy
  passed
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
  passed
```

The workspace test was run serially because the first parallel attempt exhausted the 4 GiB local container while linking the bundled DuckDB test binaries; the serial run passed. Protected GitHub evidence is still required before R8-00 can be considered complete. The new GitHub check must exist on the protected revision before branch protection can require it.

## Known limitations and next work

- The inventory is the R8-00 baseline, not the final R8-01 compatibility matrix.
- No migration graph, release manifest, SBOM, signing, target package, independent review, or official benchmark freeze is implemented by this change.
- v0.7.1 paired scored evidence and release closure remain pending.
- v0.7.2 external egress certification, authorized live conformance, and release closure remain pending.
- No pending connector interface is represented as stable, and no R8 milestone is complete.
