#!/usr/bin/env bash
set -euo pipefail

python3 -c 'import sys; raise SystemExit(0 if (3, 11) <= sys.version_info < (3, 14) else "Python 3.11 through 3.13 is required for the pinned upstream matrix")'

readonly requested="${1:-all}"
readonly frameworks=(autogen crewai google-adk langgraph semantic-kernel)
readonly temporary="$(mktemp -d)"
trap 'rm -rf -- "$temporary"' EXIT

for framework in "${frameworks[@]}"; do
    if [[ "$requested" != all && "$requested" != "$framework" ]]; then
        continue
    fi
    environment="$temporary/$framework"
    python3 -m venv "$environment"
    "$environment/bin/pip" install --disable-pip-version-check --no-input --no-cache-dir \
        "./sdk/python[$framework]"
    "$environment/bin/python" scripts/ci/v071-upstream-contracts.py "$framework"
    rm -rf -- "$environment"
done
