#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

require_rust_toolchain
readonly clean_target="$(mktemp -d)"
trap 'rm -rf -- "$clean_target"' EXIT

cargo test -p hunteval-domain --test schema_v04
cargo test -p hunteval-fixture-tool --test determinism
CARGO_TARGET_DIR="$clean_target" cargo test -p hunteval-domain --test schema_v04
CARGO_TARGET_DIR="$clean_target" cargo test -p hunteval-fixture-tool --test determinism

echo "cached and clean target directories produce equivalent contract and fixture outcomes"
