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
python3 scripts/ci/collect-r4-topology-experiment.py \
    "$HUNTEVAL_CI_ROOT" "$benchmark" "$benchmark/benchmark-report.json" \
    "$HUNTEVAL_CI_ROOT/deployments/single-agent-scripted/topology.json" \
    "$HUNTEVAL_CI_ROOT/deployments/supervisor-specialist-scripted/topology.json" \
    "$output_root/topology-experiment.json" "$output_root/topology-observations.json"
"$cli" benchmark topology-report \
    --experiment "$output_root/topology-experiment.json" \
    --baseline-topology deployments/single-agent-scripted/topology.json \
    --candidate-topology deployments/supervisor-specialist-scripted/topology.json \
    --statistical-policy examples/contracts/v0.6/statistical-policy.json \
    --scoring-profile examples/scoring-profile-balanced.yaml \
    --observations "$output_root/topology-observations.json" \
    --seed 17 --format json >"$output_root/topology-report.json"
"$cli" benchmark topology-report \
    --experiment "$output_root/topology-experiment.json" \
    --baseline-topology deployments/single-agent-scripted/topology.json \
    --candidate-topology deployments/supervisor-specialist-scripted/topology.json \
    --statistical-policy examples/contracts/v0.6/statistical-policy.json \
    --scoring-profile examples/scoring-profile-balanced.yaml \
    --observations "$output_root/topology-observations.json" \
    --seed 17 --format html >"$output_root/topology-report.html"
"$cli" report verify "$benchmark" --format json >"$output_root/verification.json"
"$cli" diagnose benchmark "$benchmark" --output "$output_root/diagnosis"
"$cli" diagnose verify "$output_root/diagnosis" --format json \
    >"$output_root/diagnostic-verification.json"
find "$benchmark/runs" -type f -name manifest.json -print0 \
    | sort -z \
    | while IFS= read -r -d '' manifest; do
        "$cli" run verify "$(dirname "$manifest")" --format json
    done >"$output_root/run-verification.jsonl"

cp "$benchmark/benchmark-report.json" "$output_root/benchmark-report.json"
cp "$benchmark/benchmark-report.html" "$output_root/benchmark-report.html"
mapfile -d '' generated_files < <(
    cd "$output_root"
    find . -type f ! -name generated-secret-scan.json -printf '%P\0' | sort -z
)
"$cli" system secret-scan --root "$output_root" --format json -- "${generated_files[@]}" \
    >"$output_root/generated-secret-scan.json"
python3 scripts/ci/collect-r2-evidence.py \
    "$benchmark" examples/cloud-mvp-benchmark.yaml "$output_root/r2-evidence.json"
(
    cd "$output_root"
    sha256sum \
        benchmark-report.json benchmark-report.html r2-evidence.json verification.json \
        run-verification.jsonl generated-secret-scan.json topology-experiment.json \
        topology-observations.json topology-report.json topology-report.html \
        diagnostic-verification.json diagnosis/benchmark-diagnostic-report.json \
        diagnosis/benchmark-diagnostic-report.html diagnosis/diagnostic-recurrence.json \
        diagnosis/diagnostic-bundle-manifest.json \
        >SHA256SUMS
    sha256sum -c SHA256SUMS
)

echo "end-to-end artifacts: $output_root"
