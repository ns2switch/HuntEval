#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 5 ]]; then
    echo "usage: $0 COMMAND_JSON SECRET_FILE WORKER EGRESS_ENFORCEMENT_JSON OUTPUT" >&2
    exit 2
fi

python3 scripts/ci/v072-live-conformance.py \
    --command "$1" \
    --secret "$2" \
    --worker "$3" \
    --egress-enforcement "$4" \
    --output "$5"
