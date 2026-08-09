#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure security
require_rust_toolchain
require_cargo_deny

readonly bubblewrap="${HUNTEVAL_BWRAP:-/usr/bin/bwrap}"
if [[ ! -x "$bubblewrap" ]]; then
    echo "error: executable Bubblewrap is required at $bubblewrap" >&2
    exit 1
fi
"$bubblewrap" --version

cargo deny check
cargo run --quiet -p hunteval-cli -- system check --format json
cargo test -p hunteval-domain --test workspace_policy
cargo test -p hunteval-sandbox
cargo test -p hunteval-runner --test isolation linux_backend_hides_private_root_and_network
cargo test -p hunteval-duckdb --test sql_policy
cargo test -p hunteval-duckdb --test worker_failures
cargo test -p hunteval-duckdb --test worker_isolation
cargo test -p hunteval-knowledge --test injection
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/secret-scan.sh
