#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure r8-compatibility
require_rust_toolchain

cargo test --locked -p hunteval-release
cargo clippy --locked -p hunteval-release --all-targets --all-features -- -D warnings
python3 scripts/ci/test-r8-closure.py
./scripts/check-dependency-direction.sh
git diff --check

echo "R8 compatibility inventory gate passed"
