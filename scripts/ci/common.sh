#!/usr/bin/env bash
set -euo pipefail

readonly HUNTEVAL_CI_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
readonly HUNTEVAL_RUST_VERSION="1.93.1"
readonly HUNTEVAL_CARGO_DENY_VERSION="0.20.2"

cd "$HUNTEVAL_CI_ROOT"

require_rust_toolchain() {
    local actual
    actual="$(rustc --version | awk '{print $2}')"
    if [[ "$actual" != "$HUNTEVAL_RUST_VERSION" ]]; then
        echo "error: rustc $HUNTEVAL_RUST_VERSION is required; found $actual" >&2
        return 1
    fi
}

require_cargo_deny() {
    local actual
    if ! actual="$(cargo deny --version 2>/dev/null | awk '{print $2}')"; then
        echo "error: cargo-deny $HUNTEVAL_CARGO_DENY_VERSION is required" >&2
        return 1
    fi
    if [[ "$actual" != "$HUNTEVAL_CARGO_DENY_VERSION" ]]; then
        echo "error: cargo-deny $HUNTEVAL_CARGO_DENY_VERSION is required; found $actual" >&2
        return 1
    fi
}

seeded_failure() {
    local requested_gate="$1"
    if [[ "${HUNTEVAL_CI_SEEDED_FAILURE:-}" == "$requested_gate" ]]; then
        echo "seeded failure reached canonical $requested_gate gate" >&2
        return 86
    fi
}
