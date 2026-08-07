# Use case: compare two cloud threat-hunting deployments

## Goal

An evaluation engineer wants to compare a scripted single-agent deployment with a scripted two-agent deployment across the same AWS, Azure, and Google Cloud identity investigations. The comparison must use identical episodes and paired seeds, keep ground truth hidden from both deployments, and remain recoverable if the controller is interrupted.

This use case answers a concrete question:

> Can both deployment architectures complete the same 18 episode-and-seed pairs with verified, directly comparable artifacts?

It demonstrates benchmark execution and comparison mechanics. It does not establish that either scripted reference deployment represents production performance.

## Inputs

The repository provides everything needed for the example:

| Input | Purpose |
|---|---|
| `examples/cloud-mvp-benchmark.yaml` | Defines two deployments, nine episodes, two paired seeds, and the scoring profile. |
| `deployments/single-agent-scripted` | External single-agent reference deployment. |
| `deployments/two-agent-scripted` | External supervisor-and-investigator reference deployment. |
| `datasets/{aws,azure,gcp}` | Nine physically separated public/private episode packages. |
| `examples/scoring-profile-balanced.yaml` | Versioned metric weights used by every cell. |

The Cartesian matrix contains 36 cells:

```text
2 deployments × 9 episodes × 2 seeds = 36 cells
```

## 1. Build the trusted components

From the repository root:

```bash
cargo build --workspace
```

This creates sibling executables for the CLI, reference deployment, and managed DuckDB worker under `target/debug/`.

## 2. Validate the benchmark before running it

```bash
target/debug/hunteval benchmark validate \
  examples/cloud-mvp-benchmark.yaml
```

Expected stdout:

```json
{"benchmark_id":"cloud-mvp","run_cells":36}
```

Validation resolves every referenced artifact, rejects unsafe paths and duplicate dimensions, and derives stable cell identities without launching a deployment.

## 3. Execute the complete matrix

Choose a new output directory; benchmark execution will not overwrite an existing one.

```bash
target/debug/hunteval benchmark run \
  examples/cloud-mvp-benchmark.yaml \
  --output runs/cloud-mvp-use-case \
  --jobs 2
```

Expected successful summary:

```json
{"total":36,"completed":36,"failed":0,"pending":0,"non_comparable":0}
```

For each cell, HuntEval starts the selected deployment in the networkless Linux sandbox, exposes only public episode data, mediates its SQL request through the constrained DuckDB worker, validates the JSONL trajectory, evaluates the final submission privately, and records exact artifact hashes.

## 4. Inspect progress or recover an interruption

Status is available in human-readable or JSON form:

```bash
target/debug/hunteval benchmark status runs/cloud-mvp-use-case
target/debug/hunteval benchmark status \
  runs/cloud-mvp-use-case \
  --format json
```

If the controller is interrupted, resume its interrupted cell and all still-pending cells with:

```bash
target/debug/hunteval benchmark resume \
  runs/cloud-mvp-use-case \
  --retry interrupted
```

The interrupted attempt remains in the append-only history. The resumed execution receives a new attempt identifier. Use `--retry failed` only when failed cells should also receive a new attempt.

## 5. Verify comparison eligibility

```bash
target/debug/hunteval benchmark compare \
  runs/cloud-mvp-use-case \
  --left single-agent-scripted \
  --right two-agent-scripted
```

An eligible response has this shape:

```json
{
  "schema_version": "0.4",
  "comparison_id": "comparison:<sha256>",
  "status": "eligible",
  "cell_ids": ["cell:<sha256>", "cell:<sha256>"],
  "reasons": []
}
```

The real response lists all 36 paired cell identifiers. Before returning `eligible`, HuntEval requires every pair to be complete and verifies each normalized `result.json` against the digest recorded in the benchmark journal. A missing deployment, failed cell, unpaired seed, configuration drift, or modified result produces `ineligible` with reason codes and exit code `3`.

## 6. Understand the stored evidence

The output directory is self-describing:

```text
runs/cloud-mvp-use-case/
├── benchmark-controller.json   local resume configuration
├── benchmark-definition.json   resolved, content-addressed matrix
├── benchmark-events.jsonl      authoritative append-only history
├── benchmark-state.json        deterministic current-state projection
└── runs/
    └── run-cell:<digest>-1/
        ├── aggregate-score.json
        ├── manifest.json
        ├── metrics.json
        ├── result.json
        ├── submission.json
        └── trajectory.jsonl
```

`benchmark-events.jsonl` answers what happened to every attempt. `benchmark-state.json` answers what is currently complete, failed, or pending. Each per-cell `manifest.json` records reproducibility hashes, while `result.json` is the normalized artifact used for verified comparison eligibility.

## Expected decision

When the command reports 36 completed cells and an eligible comparison, the engineer may proceed to analyze paired metrics for the two deployment architectures. An ineligible comparison is not a low score: it means the available cells cannot support a valid like-for-like claim and the listed reason codes must be resolved first.

The complete command and exit-code reference is in [BENCHMARK_CLI.md](BENCHMARK_CLI.md).
