#!/usr/bin/env bash
set -euo pipefail

workspace_root=$(git rev-parse --show-toplevel)
cd "$workspace_root"

mapfile -d '' tracked_files < <(git ls-files -z --cached --others --exclude-standard)
if [ "${#tracked_files[@]}" -eq 0 ]; then
    echo "error: no tracked files were selected for secret scanning" >&2
    exit 1
fi

cargo run --quiet -p hunteval-cli -- \
    system secret-scan --root . --format json -- "${tracked_files[@]}"
