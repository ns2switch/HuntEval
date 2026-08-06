#!/usr/bin/env sh
set -eu

metadata_file=$(mktemp)
trap 'rm -f "$metadata_file"' EXIT HUP INT TERM

cargo metadata --no-deps --format-version 1 > "$metadata_file"
python3 scripts/check_dependency_direction.py "$metadata_file"
