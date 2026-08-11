#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure r8-supply-chain
require_rust_toolchain
python3 scripts/ci/test-r8-supply-chain.py

readonly temporary="$(mktemp -d)"
trap 'rm -rf -- "$temporary"' EXIT
mkdir -p "$temporary/package/bin"
install -m 0755 /bin/true "$temporary/package/bin/hunteval-smoke"
cargo metadata --locked --format-version 1 >"$temporary/cargo-metadata.json"
python3 scripts/r8_supply_chain.py build \
    --package-root "$temporary/package" \
    --output "$temporary/evidence" \
    --metadata "$temporary/cargo-metadata.json" \
    --revision "$(git rev-parse --verify HEAD)" \
    --target x86_64-unknown-linux-gnu \
    --rust-toolchain "$HUNTEVAL_RUST_VERSION" \
    --epoch "$(git show -s --format=%ct HEAD)" \
    --network-isolated
python3 scripts/r8_supply_chain.py verify --root "$temporary/evidence"

cp -R "$temporary/evidence" "$temporary/first"
rm -rf -- "$temporary/evidence"
python3 scripts/r8_supply_chain.py build \
    --package-root "$temporary/package" \
    --output "$temporary/evidence" \
    --metadata "$temporary/cargo-metadata.json" \
    --revision "$(git rev-parse --verify HEAD)" \
    --target x86_64-unknown-linux-gnu \
    --rust-toolchain "$HUNTEVAL_RUST_VERSION" \
    --epoch "$(git show -s --format=%ct HEAD)" \
    --network-isolated
diff -ru "$temporary/first" "$temporary/evidence"

echo "R8 supply-chain evidence gate passed"
