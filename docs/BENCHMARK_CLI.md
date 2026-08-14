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

The R8 expanded corpus remains a pre-promotion candidate while independent episode review and the additive official-pack contract decision are pending. It can be validated and exercised without changing the historical official candidate:

```bash
target/debug/hunteval benchmark validate examples/cloud-expanded-benchmark.yaml
target/debug/hunteval benchmark run examples/cloud-expanded-benchmark.yaml \
  --output runs/cloud-expanded-r8 \
  --jobs 2
```

This resolves 54 episodes across three deployments and paired seeds 11 and 29, for 324 explicit cells. A successful local run does not make the expanded corpus release-eligible.

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

## Host capability, deployment conformance, and run integrity

Before scored execution, verify the supported Linux isolation boundary:

```bash
target/debug/hunteval system check --format json
```

The command executes safe namespace, mount, network, process-tree, and resource probes. It returns nonzero if any required capability is unavailable; scored execution does not downgrade to an unsandboxed path.

Validate an external protocol peer offline with synthetic public inputs:

```bash
target/debug/hunteval deployment conformance ./deployment-bin \
  --format json -- --deployment-specific-argument
```

Conformance checks protocol registration, HuntEval-managed tool mediation, terminal submission, and transcript integrity through the production sandbox. It is not a benchmark score or an investigation-quality certification.

Verify a stored run without a model provider or private ground truth:

```bash
target/debug/hunteval run verify runs/cloud-mvp/runs/<run-id> --format json
```

The verifier checks safe files, manifest compatibility, exact digests, trajectory replay, submission equivalence, execution policy, and normalized result consistency. JSON output is one deterministic line suitable for JSONL collection. Completed valid runs return zero; partial, invalid, tampered, and unsupported runs return nonzero. Public output explicitly reports `private_evaluation: not_checked`.

Repository and release scans use an explicit file inventory:

```bash
./scripts/ci/secret-scan.sh
```

Matches never print the candidate value. Findings or an incomplete scan fail closed.

## Evidence-backed diagnosis

Generate a content-addressed diagnostic bundle only from a verified stored run or benchmark:

```bash
target/debug/hunteval diagnose run runs/cloud-mvp/runs/<run-id> \
  --output artifacts/run-diagnosis
target/debug/hunteval diagnose benchmark runs/cloud-mvp \
  --output artifacts/benchmark-diagnosis
target/debug/hunteval diagnose verify artifacts/benchmark-diagnosis --format json
```

Generation refuses an existing destination, a destination inside the source, symbolic-link inputs or output parents, oversized artifacts, digest drift, malformed replay, and unsupported schema versions. The bundle retains bounded copies of every public source referenced by a displayed claim, including run trajectories and completed-cell results; it never copies evaluator-only inputs. Verification checks the bundle identity, unique safe inventory, media types, exact sizes and hashes, canonical taxonomy and classifier-registry hashes, typed diagnosis documents, report validation, and every report-owned artifact and claim-source digest. Exit code `0` means the offline bundle verified; malformed, incomplete, stale, or tampered bundles return `1`.

Run reports separate observation, classification, unvalidated hypothesis, and available bottleneck measurements. Benchmark reports group recurrence only within exact deployment/configuration cohorts and retain every excluded cell. Recurrence is descriptive, and controlled contribution remains experimental and topology-dependent. No diagnosis command changes a deployment artifact or affects the authoritative raw metric vector.

## Controlled improvement

R6 validates one explicitly registered candidate variable against an equivalent paired benchmark matrix. The CLI refuses stale candidates, ineligible equivalence results, unresolved pairs, links, oversized inputs, and hidden-test selection controls.

```bash
target/debug/hunteval improvement validate \
  --experiment artifacts/improvement-experiment.json \
  --equivalence artifacts/improvement-equivalence.json \
  --candidate-artifact artifacts/candidate-instruction.json \
  --benchmark-manifest examples/cloud-mvp-benchmark.yaml

target/debug/hunteval improvement run \
  --experiment artifacts/improvement-experiment.json \
  --equivalence artifacts/improvement-equivalence.json \
  --candidate-artifact artifacts/candidate-instruction.json \
  --benchmark-manifest examples/cloud-mvp-benchmark.yaml \
  --output runs/controlled-improvement --jobs 2

target/debug/hunteval improvement resume runs/controlled-improvement \
  --experiment artifacts/improvement-experiment.json \
  --equivalence artifacts/improvement-equivalence.json \
  --candidate-artifact artifacts/candidate-instruction.json \
  --retry interrupted

target/debug/hunteval improvement status runs/controlled-improvement --format json
target/debug/hunteval improvement verify artifacts/improvement-bundle --format json
```

Execution delegates every cell to the canonical benchmark service, preserving its sandbox, managed-tool mediation, budgets, attempt history, failures, resume behavior, scoring profile, and raw metric vector. The improvement decision is constraint-first: missing or unverifiable measurements cannot satisfy a hard constraint and are never imputed.

There is deliberately no CLI operation that edits or adopts deployment instructions. A suggestion is a separate non-authoritative artifact. `validated` requires a passing controlled decision, `approved` requires an explicit human-decision artifact, and `adopted` records a separately confirmed external action. Changing any bound candidate or control digest invalidates the prior validation.
