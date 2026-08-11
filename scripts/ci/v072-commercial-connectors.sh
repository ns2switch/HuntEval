#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure v072-commercial-connectors
python3 -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else "Python 3.11 or newer is required")'

cargo test -p hunteval-commercial
cargo clippy -p hunteval-commercial --all-targets --all-features -- -D warnings
python3 scripts/ci/test-v072-live-harness.py target/debug/hunteval-commercial-worker
PYTHONPATH=sdk/python/src python3 -m unittest \
    sdk/python/tests/test_commercial_connectors.py \
    -v
python3 -m compileall -q sdk/python/src sdk/python/tests
