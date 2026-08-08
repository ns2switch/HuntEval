#!/usr/bin/env bash
set -euo pipefail

readonly root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"

expect_failure() {
    local gate="$1"
    shift
    if HUNTEVAL_CI_SEEDED_FAILURE="$gate" "$@" >/dev/null 2>&1; then
        echo "error: seeded $gate failure did not propagate" >&2
        return 1
    fi
}

expect_failure policy ./scripts/ci/quality.sh policy
expect_failure security ./scripts/ci/security.sh
expect_failure e2e ./scripts/ci/e2e.sh
if HUNTEVAL_BWRAP=/definitely/missing ./scripts/ci/security.sh >/dev/null 2>&1; then
    echo "error: missing Bubblewrap capability did not fail closed" >&2
    exit 1
fi

echo "seeded gate failures propagate correctly"
