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
    "$api/rulesets?per_page=100" >"$temporary/rulesets.json"
python3 scripts/ci/verify-github-settings.py \
    "$temporary/protection.json" "$temporary/rulesets.json"
