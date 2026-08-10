#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/common.sh"
require_rust_toolchain

cargo build --workspace --bins
cargo test -p hunteval-domain --test schema_v07 --test diagnosis_v07
cargo test -p hunteval-evaluation \
    --test diagnostic_taxonomy \
    --test diagnostic_classification \
    --test diagnostic_recurrence \
    --test bottleneck_metrics \
    --test contribution_analysis
cargo test -p hunteval-runner \
    --test diagnostic_service \
    --test diagnostic_benchmark
cargo test -p hunteval-reporting \
    --test diagnostic_report \
    --test diagnostic_reporting
cargo test -p hunteval-cli --test diagnose
