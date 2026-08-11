#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
seeded_failure package
require_rust_toolchain
if [[ $# -ne 1 ]]; then
    echo "error: provide one new absolute output directory" >&2
    exit 2
fi
python3 scripts/r8_candidate.py \
    --target x86_64-unknown-linux-gnu \
    --runner ubuntu-22.04 \
    --output "$1"
echo "release-candidate dry-run artifacts: $1"
