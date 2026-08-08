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

## Comparative reports

Generate the normalized machine-readable report or the portable static HTML view from a benchmark artifact directory:

```bash
target/debug/hunteval report generate runs/cloud-mvp --format json
target/debug/hunteval report generate runs/cloud-mvp --format html
target/debug/hunteval report verify runs/cloud-mvp --format text
target/debug/hunteval report verify runs/cloud-mvp/benchmark-report.json --format json
```

Benchmark generation always writes `benchmark-report.json`, the deterministic source of truth. HTML generation additionally writes `benchmark-report.html`; it contains semantic HTML and inline static CSS, but no scripts or event handlers. Both views retain incomplete cells, missing observations, sample counts, inconclusive comparisons, constraint-first rankings, observable claim sources, limitations, and artifact hashes.

Input type is detected from validated artifacts rather than directory names. Reads are bounded, symbolic links and traversal are rejected, and output replacement is atomic. Verification revalidates the normalized contract and checks the exact SHA-256 digest of every artifact listed in the report. A stale, missing, oversized, or modified artifact makes verification return exit code 1. Run report generation remains compatible with directories containing a validated `result.json`; run verification checks its referenced artifact files, while benchmark verification additionally enforces journaled digests.
