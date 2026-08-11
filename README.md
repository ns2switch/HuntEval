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

The optional pure Python contract SDK requires Python 3.11 or newer and can be installed locally with:

```bash
python3 -m pip install ./sdk/python
```

It provides typed public models, digest-verifying artifact readers, and a bounded deployment-protocol peer. It does not execute benchmarks, evaluate results, or call scored tools directly.

## Connect your agent deployment

The Python SDK provides one framework-neutral lifecycle and adapters for common agent frameworks:

| Integration | Current status |
|---|---|
| CrewAI | supported R7 baseline |
| LangGraph | implemented with local exact-package and fixture conformance |
| AutoGen AgentChat | implemented with local exact-package and fixture conformance |
| Google ADK | implemented with local exact-package conformance; remote A2A is disabled |
| Semantic Kernel | preview with local exact-package and fixture conformance |
| MCP | bounded local `stdio` interoperability |

Adapters receive only public run inputs and translate observable tasks, delegation, managed-tool requests, evidence, findings, and the final submission into HuntEval's process protocol. HuntEval retains control of scored tools, budgets, validation, and provenance. Framework scratchpads and private chain of thought are neither requested nor recorded.

Start with the [framework and MCP connector guide](docs/FRAMEWORK_CONNECTORS.md). The pinned upstream versions pass isolated local Python 3.11 public-surface checks; protected CI, provider and full scored benchmark evidence remain pending.

HuntEval also includes deterministic offline connector previews for CrowdStrike Falcon, Google Security Operations, Microsoft Sentinel, Elastic Security, and Cortex XSIAM. These previews replay content-addressed synthetic fixtures and perform no DNS, network, credential, or tenant access. A fail-closed sanitizer can convert explicitly reviewed private recording fields into revalidated synthetic fixtures without retaining source values. See the [commercial connector guide](docs/COMMERCIAL_CONNECTORS.md) for the supported read-only operation families and current limitations.

Validate the included R7 analytical corpus and extension examples:

```bash
target/debug/hunteval knowledge validate examples/contracts/v0.9/analytical-corpus-manifest.json
target/debug/hunteval knowledge build \
  examples/contracts/v0.9/analytical-corpus-manifest.json \
  --root .
target/debug/hunteval knowledge query \
  examples/contracts/v0.9/analytical-corpus-manifest.json \
  examples/contracts/v0.9/analytical-query.json \
  --root . \
  --audit /tmp/hunteval-retrieval-audit.jsonl
target/debug/hunteval extension validate \
  examples/contracts/v0.9/extension-manifest.json \
  --policy examples/contracts/v0.9/extension-capability-policy.json
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

R2 through R7 are complete. HuntEval provides the offline benchmark loop, topology science, evidence-backed diagnosis, controlled improvements, artifact-grounded local search, stable extension contracts, a Python SDK, and a CrewAI connector.

The local v0.7.1 framework/MCP implementation and the v0.7.2 commercial gateway, worker, HTTPS transport, vendor mappings, recording sanitizer, and offline safety foundation now exist and have dedicated CI gates. They are not release-complete: protected upstream-framework execution, full scored paired benchmarks, migration rehearsal against a published package, externally enforced live egress, platform token issuance, authorized non-production tenant evidence, and protected closure evidence remain pending. No commercial connector is currently certified for live or production-scored execution. HuntEval never adopts a suggested change automatically.

R8 closure is in progress. R8-00 through R8-05 are complete, all 17 protected checks passed on candidate revision `47cf61d`, and the immutable `v0.8.0-rc.5` rehearsal built, installed, signed, and verified native packages on Linux x86_64, macOS Intel, macOS Apple Silicon, and Windows x86_64. Only Linux claims scored execution; the other platforms remain package previews. The pending v0.7.1/v0.7.2 interfaces remain outside the stable freeze. R8 is not complete until independent security and reproducibility reviews pass and a final candidate binds those review records.

- [Roadmap through v1.0](docs/ROADMAP.md)
- [R5 completion evidence](docs/R5_COMPLETION_EVIDENCE.md)
- [R6 implementation plan](docs/R6_IMPLEMENTATION_PLAN.md)
- [Controlled improvement use case](docs/USE_CASE_CONTROLLED_IMPROVEMENT.md)
- [R6 completion evidence](docs/R6_COMPLETION_EVIDENCE.md)
- [R7 implementation plan](docs/R7_IMPLEMENTATION_PLAN.md)
- [Knowledge and extensions use case](docs/USE_CASE_KNOWLEDGE_EXTENSIONS.md)
- [CrewAI connector](docs/CREWAI_CONNECTOR.md)
- [R7 completion evidence](docs/R7_COMPLETION_EVIDENCE.md)
- [Pre-R8 connector implementation plan](docs/PRE_R8_CONNECTOR_IMPLEMENTATION_PLAN.md)
- [v0.7.1 framework connector plan](docs/V071_FRAMEWORK_CONNECTOR_PLAN.md)
- [Framework and MCP connector guide](docs/FRAMEWORK_CONNECTORS.md)
- [Connector support matrix](docs/CONNECTOR_SUPPORT_MATRIX.md)
- [v0.7.1 local implementation evidence](docs/V071_IMPLEMENTATION_EVIDENCE.md)
- [v0.7.2 commercial platform connector plan](docs/V072_COMMERCIAL_CONNECTOR_PLAN.md)
- [Commercial connector preview guide](docs/COMMERCIAL_CONNECTORS.md)
- [v0.7.2 local implementation evidence](docs/V072_IMPLEMENTATION_EVIDENCE.md)
- [R8 release-candidate implementation plan](docs/R8_IMPLEMENTATION_PLAN.md)
- [R8 compatibility matrix](docs/R8_COMPATIBILITY.md)
- [R8 candidate operations](docs/R8_OPERATIONS.md)
- [R8 candidate evidence](docs/R8_CANDIDATE_EVIDENCE.md)
- [Official benchmark card](docs/OFFICIAL_BENCHMARK_CARD.md)

## Development

Run the same authoritative quality gates used by GitHub Actions:

```bash
./scripts/ci/quality.sh
./scripts/ci/security.sh
HUNTEVAL_SKIP_FUZZ_SMOKE=1 ./scripts/ci/r3-adversarial.sh
./scripts/ci/r4-science.sh
./scripts/ci/r5-diagnosis.sh
./scripts/ci/r6-improvement.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/v072-commercial-connectors.sh
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
