#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/common.sh"
require_rust_toolchain

cargo test -p hunteval-domain --test schema_v06 --test science_v06
cargo test -p hunteval-fixture-tool --test contributor --test determinism --test schema
cargo test -p hunteval-runner --test cloud_fixtures --test dataset_review --test topology_equivalence
cargo test -p hunteval-statistics --test calibration --test statistical_policy
cargo test -p hunteval-evaluation --test topology_metrics
cargo test -p hunteval-reporting --test topology_reporting
cargo test -p hunteval-cli --test benchmark_validate
