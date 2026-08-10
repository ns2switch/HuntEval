# HuntEval

HuntEval helps you evaluate whether a single-agent or multi-agent threat-hunting deployment actually works, how much it costs, and where it fails.

It runs reproducible cloud-security investigations, records observable actions and evidence, compares complete deployments under equivalent conditions, and produces auditable JSON and static HTML reports.

HuntEval is open source, framework-neutral, and designed to run locally without access to a production cloud account or SIEM.

## What is HuntEval for?

Use HuntEval when you want to:

- compare a single agent with supervisor/worker or specialist architectures;
- measure investigation quality, evidence quality, coordination, efficiency, resilience, and stability;
- understand duplicate work, missed evidence, task bottlenecks, and supported failure patterns;
- test deployment changes against the same episodes, seeds, budgets, tools, and scoring policy;
- reproduce and verify results from exact datasets, configurations, instructions, schemas, and binaries;
- build and review deterministic AWS, Azure, and Google Cloud threat-hunting benchmarks.

HuntEval evaluates the complete deployment rather than an isolated model. Agents, roles, topology, coordination, managed tools, memory boundaries, configuration, and final investigative output are all part of the evaluated system.

## Example use case

Suppose you have three implementations of the same threat-hunting assistant:

1. one generalist agent;
2. a supervisor and one investigator;
3. a supervisor coordinating specialist agents.

HuntEval can run all three against the same cloud episodes and paired seeds while keeping budgets, tool policy, scoring profile, and other declared controls equivalent.

```text
cloud telemetry in Parquet
  -> sandboxed deployment investigation
  -> HuntEval-managed read-only DuckDB queries
  -> evidence and findings with provenance
  -> hidden-ground-truth evaluation
  -> metric vectors and paired comparison
  -> verifiable JSON and static HTML reports
```

The result shows both investigation quality and operational trade-offs. A deployment can recover more malicious activity while also using more tool calls, creating duplicate tasks, or increasing coordination overhead. HuntEval keeps those dimensions visible instead of hiding them behind a universal score.

See the complete [cloud deployment comparison use case](docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md) for a guided example.

## Installation

### Requirements

- Linux for scored execution;
- Rust `1.93.1` through `rustup`;
- a C/C++ build toolchain;
- [Bubblewrap](https://github.com/containers/bubblewrap) at `/usr/bin/bwrap`.

HuntEval fails closed when the required filesystem, process, resource, or network-isolation capabilities are unavailable.

On Ubuntu or Debian:

```bash
sudo apt-get update
sudo apt-get install --yes bubblewrap build-essential pkg-config
```

Clone and build the project:

```bash
git clone https://github.com/ns2switch/HuntEval.git
cd HuntEval
rustup toolchain install 1.93.1 --profile minimal --component clippy,rustfmt
cargo build --workspace
```

Verify that the host supports scored execution:

```bash
target/debug/hunteval system check --format json
```

## Run your first investigation

The repository includes deterministic fixtures and reference deployments, so you can run an offline investigation immediately:

```bash
target/debug/hunteval run \
  --episode datasets/aws/aws-iam-001 \
  --deployment deployments/two-agent-scripted
```

Inspect the recorded trajectory and generate an HTML report:

```bash
target/debug/hunteval trajectory inspect runs/latest/trajectory.jsonl
target/debug/hunteval report generate runs/latest --format html
```

The run records the deployment configuration, managed tool requests and results, tasks, evidence, findings, metrics, and exact artifact hashes. Hidden ground truth is loaded only by the trusted evaluator and is never delivered to the deployment.

## Compare complete deployments

Validate and run the included cloud benchmark:

```bash
target/debug/hunteval benchmark validate examples/cloud-mvp-benchmark.yaml
target/debug/hunteval benchmark run examples/cloud-mvp-benchmark.yaml \
  --output runs/cloud-mvp \
  --jobs 2
```

The current example matrix contains three reference deployments, eighteen AWS/Azure/GCP episodes, and two paired seeds: 108 reproducible runs.

Check progress, compare two deployments, and generate reports:

```bash
target/debug/hunteval benchmark status runs/cloud-mvp --format json
target/debug/hunteval benchmark compare runs/cloud-mvp \
  --left single-agent-scripted \
  --right two-agent-scripted
target/debug/hunteval report generate runs/cloud-mvp --format json
target/debug/hunteval report generate runs/cloud-mvp --format html
target/debug/hunteval report verify runs/cloud-mvp --format json
```

Interrupted benchmark matrices can be resumed without replacing prior attempt history. Missing, failed, non-comparable, or unverifiable results remain explicit and are never silently converted to zero.

## Diagnose a result

HuntEval can classify supported failure patterns from verified observable artifacts and group recurrent failures across a benchmark:

```bash
target/debug/hunteval diagnose benchmark runs/cloud-mvp \
  --output runs/cloud-mvp-diagnosis
target/debug/hunteval diagnose verify runs/cloud-mvp-diagnosis
```

Diagnostic claims cite exact runs, events, actions, tasks, evidence, findings, metrics, or controlled experiments. HuntEval does not request private chain of thought, and a recommendation remains an unvalidated hypothesis until a controlled experiment passes and a human approves it.

See the [evidence-backed diagnosis use case](docs/USE_CASE_EVIDENCE_BACKED_DIAGNOSIS.md) for more detail.

## What HuntEval does not do

HuntEval is not:

- a production SIEM connector or incident-response platform;
- an autonomous offensive agent;
- tied to a specific LLM provider, agent framework, or topology;
- a system for collecting private chain of thought;
- a hosted leaderboard with one universal score;
- an autonomous prompt-adoption system.

Scored execution against production SIEMs, unrestricted deployment network access, distributed execution, Kubernetes, and autonomous adoption of generated changes remain outside the pre-v1.0 scope.

## Project status

R2 through R5 are complete. They provide the local benchmark loop, hardened sandbox and protocol, benchmark-science and topology experiments, and evidence-backed diagnosis. R6, the controlled improvement workflow, has started with its schema 0.8 contract freeze; the runtime artifact registry is the next implementation milestone.

- [Roadmap through v1.0](docs/ROADMAP.md)
- [R5 completion evidence](docs/R5_COMPLETION_EVIDENCE.md)
- [R6 implementation plan](docs/R6_IMPLEMENTATION_PLAN.md)

## Development

Run the same authoritative quality gates used by GitHub Actions:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/e2e.sh
```

All Rust implementations must preserve the security boundaries and Clean Architecture dependency direction, use typed errors, remain readable, and keep source files cohesive and reviewable.

See [CONTRIBUTING.md](CONTRIBUTING.md) to add deployments, datasets, tests, or documentation.

## Documentation

- [Technical specification](docs/SPECIFICATION.md)
- [Architecture decisions](docs/ADR.md)
- [Contracts and JSONL protocol](docs/CONTRACTS.md)
- [Metrics, scoring, and ranking](docs/METRICS_AND_RANKING.md)
- [Benchmark CLI reference](docs/BENCHMARK_CLI.md)
- [Threat model](docs/THREAT_MODEL.md)
- [Roadmap](docs/ROADMAP.md)

## License

HuntEval is licensed under the [Apache License 2.0](LICENSE).
