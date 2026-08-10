#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure r7-extensions
require_rust_toolchain
python3 -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else "Python 3.11 or newer is required")'

cargo test -p hunteval-domain --test schema_v09 --test extension_v09
cargo test -p hunteval-knowledge --test analytical
cargo test -p hunteval-runner --test extension_policy
cargo test -p hunteval-runner --test managed_tool_conformance
cargo test -p hunteval-reference-deployment --test extension_contract
cargo test -p hunteval-reporting --test knowledge_reporting
cargo test -p hunteval-runner --test knowledge_artifacts
cargo test -p hunteval-runner --test knowledge_audit
cargo test -p hunteval-cli --test r7
PYTHONPATH=sdk/python/src python3 -m unittest discover -s sdk/python/tests -v
python3 -m compileall -q sdk/python/src sdk/python/tests

readonly wheel_a="$(mktemp -d)"
readonly wheel_b="$(mktemp -d)"
trap 'rm -rf "$wheel_a" "$wheel_b"' EXIT
python3 -m pip wheel --disable-pip-version-check --no-deps --no-build-isolation \
    --wheel-dir "$wheel_a" ./sdk/python
python3 -m pip wheel --disable-pip-version-check --no-deps --no-build-isolation \
    --wheel-dir "$wheel_b" ./sdk/python
readonly package_a="$(find "$wheel_a" -maxdepth 1 -type f -name '*.whl' -print -quit)"
readonly package_b="$(find "$wheel_b" -maxdepth 1 -type f -name '*.whl' -print -quit)"
test -n "$package_a" -a -n "$package_b"
python3 scripts/ci/check-python-wheel.py "$package_a" "$package_b"
