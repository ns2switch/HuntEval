# Benchmark CLI

The benchmark commands execute a resolved deployment × episode × seed matrix through the trusted HuntEval runner. Build the complete workspace first so `hunteval`, `hunteval-reference-deployment`, and `hunteval-duckdb-worker` are sibling executables.

```bash
cargo build --workspace
target/debug/hunteval benchmark validate examples/cloud-mvp-benchmark.yaml
target/debug/hunteval benchmark run examples/cloud-mvp-benchmark.yaml \
  --output runs/cloud-mvp \
  --jobs 2
target/debug/hunteval benchmark status runs/cloud-mvp --format json
target/debug/hunteval benchmark compare runs/cloud-mvp \
  --left single-agent-scripted \
  --right two-agent-scripted
```

`benchmark run` requires a new output directory. It resolves and hashes every matrix input before scheduling, binds the exact deployment executable into cell identity, mediates scored SQL through the isolated DuckDB worker, and writes an append-only journal. `--jobs` is bounded to 1–256. `--fail-fast` stops scheduling new batches after a failed batch without discarding unrelated terminal outcomes.

An interrupted controller leaves its current attempt recoverable. Resume without retrying other failures with:

```bash
target/debug/hunteval benchmark resume runs/cloud-mvp --retry interrupted
```

The retry values are `none`, `interrupted`, and `failed`. Retries create new attempt identifiers and never overwrite prior attempt history. Changed manifests, artifacts, scoring data, or deployment executable bytes are rejected as configuration drift; use a new output directory for the new benchmark identity.

## Output and exit codes

Machine-readable command results are written to stdout and diagnostics to stderr. `benchmark status` defaults to text and accepts `--format json`.

| Code | Meaning |
|---|---|
| 0 | The command succeeded; comparisons are eligible. |
| 1 | Validation, execution, storage, or artifact verification failed; a run summary can also report failed cells. |
| 2 | Command-line arguments are invalid (assigned by Clap). |
| 3 | A comparison was evaluated and is ineligible. |

Status and comparison read the stored resolved definition and journal, so they remain available without executing deployments. Comparison eligibility requires every paired cell to be completed and every referenced normalized result to match its journaled digest.
