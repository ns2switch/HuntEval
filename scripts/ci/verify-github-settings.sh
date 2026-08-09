#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

if [[ -z "${GITHUB_TOKEN:-}" || -z "${GITHUB_REPOSITORY:-}" ]]; then
    echo "error: GITHUB_TOKEN and GITHUB_REPOSITORY are required for settings verification" >&2
    exit 2
fi
if [[ ! "$GITHUB_REPOSITORY" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
    echo "error: GITHUB_REPOSITORY is invalid" >&2
    exit 2
fi

readonly temporary="$(mktemp -d)"
trap 'rm -rf -- "$temporary"' EXIT
readonly api="https://api.github.com/repos/$GITHUB_REPOSITORY"
umask 077
printf 'header = "Authorization: Bearer %s"\n' "$GITHUB_TOKEN" >"$temporary/curl.conf"
readonly headers=(
    -H "Accept: application/vnd.github+json"
    -H "X-GitHub-Api-Version: 2022-11-28"
    --config "$temporary/curl.conf"
)

curl --fail --silent --show-error "${headers[@]}" \
    "$api/branches/main/protection" >"$temporary/protection.json"
curl --fail --silent --show-error "${headers[@]}" \
    "$api/rulesets?per_page=100" >"$temporary/ruleset-list.json"

mapfile -t ruleset_ids < <(
    python3 - "$temporary/ruleset-list.json" <<'PY'
import json
import pathlib
import sys

rulesets = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if not isinstance(rulesets, list):
    raise SystemExit("error: GitHub ruleset response has an unexpected shape")
for ruleset in rulesets:
    identifier = ruleset.get("id") if isinstance(ruleset, dict) else None
    if not isinstance(identifier, int) or identifier < 1:
        raise SystemExit("error: GitHub ruleset response contains an invalid identifier")
    print(identifier)
PY
)

ruleset_files=()
for ruleset_id in "${ruleset_ids[@]}"; do
    if [[ ! "$ruleset_id" =~ ^[1-9][0-9]*$ ]]; then
        echo "error: GitHub ruleset identifier is invalid" >&2
        exit 1
    fi
    ruleset_file="$temporary/ruleset-$ruleset_id.json"
    curl --fail --silent --show-error "${headers[@]}" \
        "$api/rulesets/$ruleset_id" >"$ruleset_file"
    ruleset_files+=("$ruleset_file")
done

python3 - "$temporary/rulesets.json" "${ruleset_files[@]}" <<'PY'
import json
import pathlib
import sys

rulesets = [
    json.loads(pathlib.Path(path).read_text(encoding="utf-8"))
    for path in sys.argv[2:]
]
pathlib.Path(sys.argv[1]).write_text(json.dumps(rulesets), encoding="utf-8")
PY

python3 scripts/ci/verify-github-settings.py \
    "$temporary/protection.json" "$temporary/rulesets.json"
