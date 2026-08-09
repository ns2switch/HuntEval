#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure adversarial
require_rust_toolchain

cargo test -p hunteval-protocol --test property_framing
cargo test -p hunteval-protocol --test property_session
cargo test -p hunteval-protocol --test property_replay
cargo test -p hunteval-protocol --test conformance
cargo test -p hunteval-protocol --test compatibility_fixtures
cargo test -p hunteval-protocol --test fuzz_regressions
cargo test -p hunteval-runner --test run_verification
cargo test -p hunteval-reference-deployment --test conformance

if [[ "${HUNTEVAL_SKIP_FUZZ_SMOKE:-0}" != "1" ]]; then
    require_cargo_fuzz
    readonly fuzz_toolchain="${HUNTEVAL_FUZZ_TOOLCHAIN:-$HUNTEVAL_DEFAULT_FUZZ_TOOLCHAIN}"
    for target in jsonl_decoder protocol_session trajectory_replay conformance_input; do
        cargo "+$fuzz_toolchain" fuzz run "$target" -- -runs=1000
    done
fi
