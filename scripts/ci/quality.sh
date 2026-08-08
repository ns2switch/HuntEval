#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

readonly gate="${1:-all}"
require_rust_toolchain
seeded_failure "$gate"

run_policy() {
    git diff --check
    ./scripts/check-dependency-direction.sh
    ./scripts/check-source-size.sh
    find scripts -type f -name '*.sh' -print0 | xargs -0 -n1 bash -n
    python3 scripts/ci/test-github-settings.py
}

run_quality() {
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
}

run_tests() {
    cargo test --workspace
}

run_docs() {
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
}

case "$gate" in
    policy) run_policy ;;
    quality) run_quality ;;
    test) run_tests ;;
    docs) run_docs ;;
    all)
        run_policy
        run_quality
        run_tests
        run_docs
        ;;
    *)
        echo "error: unknown quality gate: $gate" >&2
        exit 2
        ;;
esac
