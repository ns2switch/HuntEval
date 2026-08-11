#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure r8-release
require_rust_toolchain

if [[ $# -ne 1 || "$1" != /* || "$1" == "/" || -e "$1" ]]; then
    echo "error: provide one new absolute R8 rehearsal output directory" >&2
    exit 2
fi
if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: R8 rehearsal requires a clean worktree" >&2
    exit 1
fi

./scripts/ci/r8-compatibility.sh
./scripts/ci/r8-supply-chain.sh
./scripts/ci/r8-package.sh
./scripts/ci/r8-signature-fixtures.sh
./scripts/ci/release-candidate.sh "$1"

readonly output_root="$1"
readonly temporary="$(mktemp -d)"
trap 'rm -rf -- "$temporary"' EXIT
readonly archive="$(find "$output_root" -maxdepth 1 -type f -name 'hunteval-rc-*.tar.gz' -print -quit)"
test -n "$archive"
python3 scripts/r8_install.py install --archive "$archive" --destination "$temporary/installed"
python3 scripts/r8_install.py verify --root "$temporary/installed"
"$temporary/installed/bin/hunteval" --help >/dev/null
python3 scripts/r8_supply_chain.py verify --root "$output_root/evidence"

python3 scripts/r8_sign.py verify \
    --artifact "$output_root/release-manifest.json" \
    --signature-root "$output_root/signature" \
    --repository "${GITHUB_REPOSITORY:-ns2switch/HuntEval}" \
    --ref "${GITHUB_REF:-refs/heads/main}" --workflow r8-native-candidate \
    --at-epoch "$(git show -s --format=%ct HEAD)"
(
    cd "$output_root"
    find signature -type f -print0 | sort -z | xargs -0 sha256sum >SIGNATURE-SHA256SUMS
    sha256sum -c SIGNATURE-SHA256SUMS >signature-verification.txt
)

printf 'production_release_published=false\n' >"$output_root/R8_REHEARSAL_STATUS"
echo "R8 non-publishing release rehearsal passed: $output_root"
