#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/common.sh"
require_rust_toolchain
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"

cargo build --workspace --bins
cargo test -p hunteval-domain --test schema_v08 --test improvement_v08
cargo test -p hunteval-evaluation \
    --test artifact_diff \
    --test candidate_safety \
    --test improvement_equivalence \
    --test validation_decision \
    --test human_review \
    --test prompt_improvement
cargo test -p hunteval-runner \
    --test artifact_registry \
    --test partition_isolation \
    --test improvement_service \
    --test improvement_e2e \
    --test recommendation_lifecycle \
    --test improvement_verification
cargo test -p hunteval-reporting --test improvement_reporting
cargo test -p hunteval-cli --test improvement
