#!/usr/bin/env sh
set -eu

review_limit=300
hard_limit=500
failed=0
source_list=$(mktemp)
trap 'rm -f "$source_list"' EXIT HUP INT TERM

find crates -path '*/src/*.rs' -type f -print | sort > "$source_list"
while IFS= read -r source_file; do
    line_count=$(wc -l < "$source_file")
    if [ "$line_count" -gt "$hard_limit" ]; then
        echo "error: $source_file has $line_count lines; maximum is $hard_limit" >&2
        failed=1
    elif [ "$line_count" -gt "$review_limit" ]; then
        echo "review: $source_file has $line_count lines; cohesion review required" >&2
    fi

done < "$source_list"

if [ "$failed" -ne 0 ]; then
    exit 1
fi

echo "source file sizes are valid"
