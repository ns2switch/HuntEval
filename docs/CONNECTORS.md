# Connector setup

HuntEval evaluates an external deployment through a bounded process protocol. The deployment can use a supported agent framework or a local MCP client; HuntEval remains responsible for scored-tool execution, budgets, validation, provenance, and hidden-ground-truth isolation.

## Choose a connector

| Deployment type | Connector | Current scope |
|---|---|---|
| CrewAI | Python `CrewAIAdapter` | supported R7 baseline |
| LangGraph | Python `LangGraphAdapter` | local contract and fixture conformance |
| AutoGen AgentChat | Python `AutoGenAdapter` | local contract and fixture conformance |
| Google ADK | Python `GoogleAdkAdapter` | local conformance; remote A2A disabled |
| Semantic Kernel | Python preview adapter | local preview conformance |
| Other local frameworks | MCP `stdio` session | bounded generic interoperability |
| Commercial platforms | Rust worker and fixture replay | offline preview; live certification pending |

Exact tested versions and remaining evidence are listed in the [connector support matrix](CONNECTOR_SUPPORT_MATRIX.md).

## Configure an agent-framework deployment

### 1. Install one adapter

Use Python 3.11 or newer and isolate each deployment's dependencies:

```bash
python3.11 -m venv .venv
. .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -e "./sdk/python[crewai]"
```

The optional groups are `crewai`, `langgraph`, `autogen`, `google-adk`, and `semantic-kernel`.

### 2. Create the deployment executable

The executable constructs the framework objects, creates a HuntEval adapter, and connects it to standard input and output with `DeploymentPeer`. Use the [CrewAI connector](CREWAI_CONNECTOR.md) as a complete implementation example.

The adapter receives only public run inputs. All scored tool calls must pass through the framework context's `managed_tool` method. The deployment returns structured tasks, actions, evidence, findings, and a final submission; arbitrary traces and private chain of thought are neither required nor recorded.

### 3. Register the process and topology

Create a deployment directory with `deployment.yaml`, its executable, and an explicit topology artifact. A minimal manifest follows the same shape as the included reference deployments:

```yaml
schema_version: "0.4"
id: my-crewai-deployment
kind: external_reference_process
architecture: supervisor_worker
agents:
  - id: supervisor
    role: orchestrator
  - id: investigator
    role: investigator
network_access: false
scored_tools: hunteval_managed_only
process:
  executable: bin/my-crewai-deployment
  arguments: []
  environment_allowlist: []
```

Use stable agent identifiers. Declare every role and topology change because they form part of the evaluated deployment identity. Do not place credentials in the manifest or run artifacts.

### 4. Run the deployment

```bash
target/debug/hunteval run \
  --episode datasets/aws/aws-iam-001 \
  --deployment path/to/my-crewai-deployment
```

Protocol violations, unknown fields, duplicate identities, invalid tool correlations, budget overruns, and attempts to bypass the managed-tool boundary fail closed.

## Connect an unsupported framework with MCP

The Python SDK includes bounded local MCP `stdio` interoperability. It exposes public episode reads and runner-mediated managed-tool requests without granting direct access to scored tools. Use this route when a framework can act as an MCP client but does not have a native HuntEval adapter.

MCP support is a protocol bridge, not a claim of native framework support. See [framework and MCP connectors](FRAMEWORK_CONNECTORS.md) for lifecycle and conformance details.

## Commercial platform previews

CrowdStrike Falcon, Google Security Operations, Microsoft Sentinel, Elastic Security, and Cortex XSIAM have finite read-only operation catalogs, deterministic fixture replay, and a disabled-by-default live worker foundation.

Start with the offline gate:

```bash
./scripts/ci/v072-commercial-connectors.sh
```

This command performs no DNS, socket, credential, provider, or tenant access. Live validation additionally requires externally enforced worker-bound egress, protected short-lived credentials, least-privilege platform configuration, and an authorized non-production tenant. It must remain disabled until those controls are independently supplied and reviewed.

Read the [commercial connector guide](COMMERCIAL_CONNECTORS.md) before configuring any live environment. Production scored execution and remote mutation are unavailable.
