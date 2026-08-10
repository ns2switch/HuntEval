#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"
python3 -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else "Python 3.11 or newer is required")'

seeded_failure package
require_rust_toolchain

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "x86_64" ]]; then
    echo "error: the current release-candidate target is x86_64 Linux only" >&2
    exit 2
fi

if [[ $# -ne 1 || "$1" != /* || "$1" == "/" || -e "$1" ]]; then
    echo "error: provide one new absolute output directory" >&2
    exit 2
fi
if [[ -n "$(git status --porcelain)" && "${HUNTEVAL_RELEASE_ALLOW_DIRTY:-}" != "1" ]]; then
    echo "error: release-candidate dry run requires a clean worktree" >&2
    exit 1
fi

readonly output_root="$1"
readonly revision="$(git rev-parse --verify HEAD)"
readonly short_revision="${revision:0:12}"
readonly archive_name="hunteval-rc-$short_revision-x86_64-unknown-linux-gnu.tar.gz"
readonly staging="$(mktemp -d)"
trap 'rm -rf -- "$staging"' EXIT
mkdir -p "$output_root" "$staging/hunteval/bin" "$staging/hunteval/schemas"

cargo build --workspace --release --locked
for binary in hunteval hunteval-duckdb-worker hunteval-reference-deployment hunteval-reference-tool hunteval-fixture-tool; do
    install -m 0755 "target/release/$binary" "$staging/hunteval/bin/$binary"
done
cp -R schemas/v0.3 schemas/v0.4 schemas/v0.5 schemas/v0.6 schemas/v0.7 schemas/v0.8 schemas/v0.9 "$staging/hunteval/schemas/"
cp -R taxonomies "$staging/hunteval/"
install -m 0644 LICENSE README.md SECURITY.md "$staging/hunteval/"

mapfile -d '' package_files < <(
    cd "$staging/hunteval"
    find . -type f -printf '%P\0' | sort -z
)
"$staging/hunteval/bin/hunteval" system secret-scan \
    --root "$staging/hunteval" --format json -- "${package_files[@]}" \
    >"$output_root/secret-scan.json"

tar --sort=name \
    --mtime='UTC 1970-01-01' \
    --owner=0 --group=0 --numeric-owner \
    -C "$staging" -cf - hunteval \
    | gzip -n >"$output_root/$archive_name"

python3 -m pip wheel --disable-pip-version-check --no-deps --no-build-isolation \
    --wheel-dir "$output_root" ./sdk/python
readonly sdk_wheel="$(find "$output_root" -maxdepth 1 -type f -name 'hunteval_sdk-*.whl' -print -quit)"
test -n "$sdk_wheel"
python3 scripts/ci/check-python-wheel.py "$sdk_wheel"

(
    cd "$output_root"
    sha256sum "$archive_name" "$(basename "$sdk_wheel")" secret-scan.json >SHA256SUMS
    sha256sum -c SHA256SUMS >verification.txt
)
{
    printf 'revision=%s\n' "$revision"
    printf 'rust_toolchain=%s\n' "$HUNTEVAL_RUST_VERSION"
    printf 'python_version=%s\n' "$(python3 -c 'import platform; print(platform.python_version())')"
    printf 'target=x86_64-unknown-linux-gnu\n'
    printf 'production_release_published=false\n'
} >"$output_root/release-metadata.txt"

echo "release-candidate dry-run artifacts: $output_root"
