#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
require_rust_toolchain

cleanup=false
if [[ -n "${HUNTEVAL_R8_CORPUS_OUTPUT:-}" ]]; then
    output_root="$HUNTEVAL_R8_CORPUS_OUTPUT"
    if [[ "$output_root" != /* || "$output_root" == "/" || -e "$output_root" ]]; then
        echo "error: HUNTEVAL_R8_CORPUS_OUTPUT must be a new absolute directory" >&2
        exit 2
    fi
    mkdir -p "$output_root"
else
    output_root="$(mktemp -d)"
    cleanup=true
fi
readonly output_root cleanup
if [[ "$cleanup" == true ]]; then
    trap 'rm -rf -- "$output_root"' EXIT
fi

cargo test -p hunteval-fixture-tool
cargo test -p hunteval-runner \
    --test benchmark_manifest --test corpus_expansion --test dataset_review
cargo test -p hunteval-cli --test benchmark_expanded --test benchmark_validate
cargo build -p hunteval-fixture-tool
cargo build -p hunteval-duckdb --bin hunteval-duckdb-worker

internal_inventory="$output_root/internal-corpus-inventory.json"
target/debug/hunteval-fixture-tool corpus-inventory datasets \
    --output "$internal_inventory" --internal
target/debug/hunteval-fixture-tool corpus-inventory datasets \
    --output "$output_root/public-corpus-inventory.json"
cmp examples/benchmark-corpus-inventory.json "$output_root/public-corpus-inventory.json"

HUNTEVAL_BENCHMARK_MANIFEST=examples/cloud-expanded-benchmark.yaml \
HUNTEVAL_E2E_OUTPUT="$output_root/e2e" \
    ./scripts/ci/e2e.sh

python3 scripts/ci/corpus-quality.py \
    --inventory "$internal_inventory" \
    --datasets datasets \
    --benchmark "$output_root/e2e/cloud-mvp" \
    --worker target/debug/hunteval-duckdb-worker \
    --output "$output_root/corpus-quality.json"

target/debug/hunteval system secret-scan \
    --root "$output_root" --format json -- corpus-quality.json public-corpus-inventory.json \
    >"$output_root/corpus-quality-secret-scan.json"

echo "R8 corpus artifacts: $output_root"
