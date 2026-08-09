# Use case: evidence-backed diagnosis

## Goal

An evaluation engineer has completed a benchmark and wants to locate recurrent observable failures and coordination bottlenecks without exposing ground truth, inferring private reasoning, or changing the evaluated deployment.

## Workflow

Build HuntEval and verify the source benchmark first:

```bash
cargo build --workspace
target/debug/hunteval benchmark status runs/cloud-mvp --format json
target/debug/hunteval report verify runs/cloud-mvp --format json
```

Generate and independently verify the benchmark diagnosis:

```bash
target/debug/hunteval diagnose benchmark runs/cloud-mvp \
  --output artifacts/cloud-mvp-diagnosis
target/debug/hunteval diagnose verify artifacts/cloud-mvp-diagnosis --format json
```

For a single completed cell, use its stored run directory:

```bash
target/debug/hunteval diagnose run runs/cloud-mvp/runs/<run-id> \
  --output artifacts/<run-id>-diagnosis
target/debug/hunteval diagnose verify artifacts/<run-id>-diagnosis --format text
```

## Interpretation

`diagnostic-report.json` is the normalized view; the static HTML file is an escaped projection. Each classification cites exact typed sources and content hashes. The bounded bundle contains the referenced public source bytes, so verification remains offline; private evaluator material is not copied. Unsupported classifications appear only as omissions. The benchmark report exposes eligible and excluded cells so failed or missing cells are not treated as negative observations.

Bottleneck observations measure only runner-visible lifecycle intervals. An unavailable value means the required observation was absent; it is not zero. Recurrence identifies repeated patterns but does not establish causality. Agent or role contribution requires a controlled R4 topology experiment and remains explicitly experimental and topology-dependent.

The output is a review artifact. R5 does not edit deployment configuration, validate an improvement, reveal hidden-test results, or approve a recommendation.
