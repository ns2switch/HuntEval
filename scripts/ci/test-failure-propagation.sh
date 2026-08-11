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
expect_failure r8-compatibility ./scripts/ci/r8-compatibility.sh
expect_failure r8-supply-chain ./scripts/ci/r8-supply-chain.sh
expect_failure r8-package ./scripts/ci/r8-package.sh
expect_failure r8-release ./scripts/ci/r8-release.sh /tmp/hunteval-seeded-r8-release
if HUNTEVAL_BWRAP=/definitely/missing ./scripts/ci/security.sh >/dev/null 2>&1; then
    echo "error: missing Bubblewrap capability did not fail closed" >&2
    exit 1
fi

echo "seeded gate failures propagate correctly"
