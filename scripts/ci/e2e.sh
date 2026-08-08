#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure e2e
require_rust_toolchain

cleanup=false
if [[ -n "${HUNTEVAL_E2E_OUTPUT:-}" ]]; then
    output_root="$HUNTEVAL_E2E_OUTPUT"
    if [[ "$output_root" != /* || "$output_root" == "/" || -e "$output_root" ]]; then
        echo "error: HUNTEVAL_E2E_OUTPUT must be a new absolute directory" >&2
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

cargo build --workspace
readonly cli="$HUNTEVAL_CI_ROOT/target/debug/hunteval"
readonly benchmark="$output_root/cloud-mvp"

"$cli" benchmark validate examples/cloud-mvp-benchmark.yaml >"$output_root/validation.json"
"$cli" benchmark run examples/cloud-mvp-benchmark.yaml \
    --output "$benchmark" --jobs 2 >"$output_root/run-summary.json"
"$cli" benchmark status "$benchmark" --format json >"$output_root/status.json"
"$cli" benchmark compare "$benchmark" \
    --left single-agent-scripted --right two-agent-scripted \
    >"$output_root/comparison.json"
"$cli" report generate "$benchmark" --format json
"$cli" report generate "$benchmark" --format html
"$cli" report verify "$benchmark" --format json >"$output_root/verification.json"

cp "$benchmark/benchmark-report.json" "$output_root/benchmark-report.json"
cp "$benchmark/benchmark-report.html" "$output_root/benchmark-report.html"
python3 scripts/ci/collect-r2-evidence.py \
    "$benchmark" examples/cloud-mvp-benchmark.yaml "$output_root/r2-evidence.json"
(
    cd "$output_root"
    sha256sum \
        benchmark-report.json benchmark-report.html r2-evidence.json verification.json \
        >SHA256SUMS
    sha256sum -c SHA256SUMS
)

echo "end-to-end artifacts: $output_root"
