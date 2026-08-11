#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure r8-package
require_rust_toolchain
python3 scripts/ci/test-r8-install.py
python3 scripts/ci/test-r8-platform.py
python3 scripts/r8_platform.py validate

echo "R8 package installation gate passed"
