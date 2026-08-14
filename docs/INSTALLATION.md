# Installation

This guide installs HuntEval from source and runs the included offline reference deployment. No cloud, model-provider, or SIEM credentials are required.

## Supported environments

Scored execution currently requires Linux because HuntEval relies on Bubblewrap for fail-closed filesystem, process, resource, and network isolation.

| Platform | Current support |
|---|---|
| Linux x86_64 | installation, inspection, and scored execution |
| macOS Intel | package preview; no scored execution claim |
| macOS Apple Silicon | package preview; no scored execution claim |
| Windows x86_64 | package preview; no scored execution claim |

See the [compatibility matrix](R8_COMPATIBILITY.md) for the exact release-candidate claims.

## Linux prerequisites

Install Rust `1.93.1` with `rustup`, a C/C++ toolchain, `pkg-config`, and Bubblewrap at `/usr/bin/bwrap`.

On Ubuntu or Debian:

```bash
sudo apt-get update
sudo apt-get install --yes bubblewrap build-essential pkg-config
rustup toolchain install 1.93.1 --profile minimal --component clippy,rustfmt
```

Clone and build HuntEval:

```bash
git clone https://github.com/ns2switch/HuntEval.git
cd HuntEval
cargo build --workspace
```

Confirm that the host can enforce the scored-execution boundary:

```bash
target/debug/hunteval system check --format json
```

HuntEval stops instead of weakening isolation when a required host capability is unavailable.

## First offline run

The repository includes deterministic cloud telemetry and scripted reference deployments:

```bash
target/debug/hunteval run \
  --episode datasets/aws/aws-iam-001 \
  --deployment deployments/two-agent-scripted
```

Inspect the observable trajectory and generate a static report:

```bash
target/debug/hunteval trajectory inspect runs/latest/trajectory.jsonl
target/debug/hunteval report generate runs/latest --format html
```

Run artifacts are written below `runs/`, which is intentionally excluded from Git.

## Python SDK

The optional SDK requires Python 3.11 or newer. Install the framework-neutral contracts without an agent framework:

```bash
python3.11 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e ./sdk/python
```

Install a framework adapter by selecting one optional dependency group:

```bash
python -m pip install -e "./sdk/python[crewai]"
python -m pip install -e "./sdk/python[langgraph]"
python -m pip install -e "./sdk/python[autogen]"
python -m pip install -e "./sdk/python[google-adk]"
python -m pip install -e "./sdk/python[semantic-kernel]"
```

Install only the adapter required by a deployment so framework and provider dependencies remain isolated. Continue with [connector setup](CONNECTORS.md).

## Troubleshooting

If `system check` fails, use its JSON result to identify the missing capability. Common causes are:

- Bubblewrap is missing or is not available at `/usr/bin/bwrap`;
- user namespaces are disabled by the host;
- the build toolchain required by DuckDB is missing;
- the filesystem does not support the required isolation behavior.

Do not bypass a failed host check for a scored run. macOS and Windows can inspect preview packages, but they do not currently provide an equivalent scored-execution boundary.
