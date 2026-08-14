# HuntEval

Evaluate single-agent and multi-agent threat-hunting deployments under controlled, reproducible conditions.

HuntEval runs complete agent deployments against deterministic cloud-security episodes, keeps scored tools and hidden ground truth inside a trusted boundary, and produces auditable metrics and reports. It is framework-neutral, runs locally, and does not require a production cloud account or SIEM for the included examples.

## Try HuntEval in five minutes

The scored runner currently requires Linux, Rust `1.93.1`, a C/C++ build toolchain, and Bubblewrap.

```bash
sudo apt-get update
sudo apt-get install --yes bubblewrap build-essential pkg-config

git clone https://github.com/ns2switch/HuntEval.git
cd HuntEval
rustup toolchain install 1.93.1 --profile minimal --component clippy,rustfmt
cargo build --workspace
target/debug/hunteval system check --format json
```

Run the included two-agent investigation. It uses local fixtures, so no credentials are needed.

```bash
target/debug/hunteval run \
  --episode datasets/aws/aws-iam-001 \
  --deployment deployments/two-agent-scripted

target/debug/hunteval trajectory inspect runs/latest/trajectory.jsonl
target/debug/hunteval report generate runs/latest --format html
```

The run records the deployment topology, observable agent actions, managed-tool results, evidence, findings, metrics, and exact artifact hashes. Ground truth is available only to the trusted evaluator.

See the full [installation guide](docs/INSTALLATION.md) for host requirements, Python SDK setup, troubleshooting, and platform support.

## Connect your agents

HuntEval provides a Python SDK and bounded adapters for:

- CrewAI;
- LangGraph;
- AutoGen AgentChat;
- Google ADK;
- Semantic Kernel preview;
- local MCP `stdio` clients.

Create an isolated Python environment and install the adapter you need:

```bash
python3.11 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e "./sdk/python[crewai]"
```

Replace `crewai` with `langgraph`, `autogen`, `google-adk`, or `semantic-kernel` as needed. Your deployment executable uses the SDK adapter to exchange the public episode, observable actions, managed-tool requests, evidence, and final submission with HuntEval. HuntEval continues to enforce budgets, protocol validation, provenance, and scored-tool execution.

Start with the [connector setup guide](docs/CONNECTORS.md) or the complete [CrewAI example](docs/CREWAI_CONNECTOR.md).

### Commercial security platforms

Offline, deterministic connector previews are available for CrowdStrike Falcon, Google Security Operations, Microsoft Sentinel, Elastic Security, and Cortex XSIAM. Live read-only workers exist behind fail-closed security controls, but no commercial connector is currently certified for production or scored live execution.

The [commercial connector guide](docs/COMMERCIAL_CONNECTORS.md) explains the available operations, fixture replay, credential boundary, and external controls still required for live validation.

## Compare deployment designs

The included benchmark compares single-agent, supervisor/worker, and supervisor/specialist deployments across AWS, Azure, and Google Cloud episodes:

```bash
target/debug/hunteval benchmark validate examples/cloud-mvp-benchmark.yaml
target/debug/hunteval benchmark run examples/cloud-mvp-benchmark.yaml \
  --output runs/cloud-mvp \
  --jobs 2

target/debug/hunteval benchmark compare runs/cloud-mvp \
  --left single-agent-scripted \
  --right two-agent-scripted

target/debug/hunteval report generate runs/cloud-mvp --format html
```

HuntEval keeps investigation quality, coordination overhead, resilience, and resource consumption visible as separate measurements. Missing or unverifiable metrics are never silently converted to zero.

## Common use cases

- [Compare complete agent deployments](docs/USE_CASE_CLOUD_DEPLOYMENT_COMPARISON.md)
- [Diagnose evidence-backed failures](docs/USE_CASE_EVIDENCE_BACKED_DIAGNOSIS.md)
- [Validate a controlled improvement](docs/USE_CASE_CONTROLLED_IMPROVEMENT.md)
- [Use local knowledge and extensions](docs/USE_CASE_KNOWLEDGE_EXTENSIONS.md)

## Documentation

- [Installation](docs/INSTALLATION.md)
- [Connector setup](docs/CONNECTORS.md)
- [Benchmark CLI](docs/BENCHMARK_CLI.md)
- [Metrics and ranking](docs/METRICS_AND_RANKING.md)
- [Platform compatibility](docs/R8_COMPATIBILITY.md)
- [Project roadmap](docs/ROADMAP.md)

## Current status

R2 through R7 are complete. R8 release-candidate closure is in progress. Linux is the only platform that currently claims scored execution; macOS and Windows packages remain installation and inspection previews.

Framework connectors have local contract and fixture conformance. Provider-backed scored benchmark evidence is still pending. Commercial integrations are offline previews until their external egress, credential, non-production tenant, and protected validation requirements are satisfied.

HuntEval is an evaluation framework, not a SOC product, autonomous offensive agent, production SIEM connector, or system for collecting private chain of thought.

## Security and license

Security reports are handled according to [SECURITY.md](SECURITY.md). HuntEval is licensed under the [Apache License 2.0](LICENSE).
