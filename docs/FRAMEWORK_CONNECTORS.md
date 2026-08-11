# Framework connectors

HuntEval framework connectors translate observable framework lifecycle activity into the existing process-neutral deployment protocol. HuntEval remains responsible for scored-tool execution, budgets, protocol validation, provenance, and terminal state.

## Supported implementation surface

The Python SDK currently implements these dependency-free structural adapters:

| Adapter | Status | Entry point |
|---|---|---|
| CrewAI | supported R7 baseline | `CrewAIAdapter` |
| LangGraph | implemented; local exact-package and fixture conformance | `LangGraphAdapter` |
| AutoGen AgentChat | implemented; local exact-package and fixture conformance | `AutoGenAdapter` |
| Google ADK | implemented against `Runner.run`; local exact-package and fixture conformance | `GoogleAdkAdapter` |
| Semantic Kernel | preview; local exact-package and fixture conformance | `SemanticKernelPreviewAdapter` |
| generic MCP client | implemented, local fixture conformance only | `McpSession` |

`implemented` does not mean release-complete. The exact upstream versions pass the local isolated Python 3.11 public-surface harness. Protected execution, provider-backed smoke tests where appropriate, full scored paired benchmark evidence, migration/rollback rehearsal against a published package, protected-branch configuration, and completion evidence remain required by `V071_FRAMEWORK_CONNECTOR_PLAN.md`.

The base SDK has no mandatory dependency on any framework. Applications supply an object satisfying the documented structural protocol and install the framework version selected by the future support matrix.

Exact candidate packages are isolated behind the `autogen`, `crewai`, `google-adk`, `langgraph`, and `semantic-kernel` optional dependency groups. `scripts/ci/v071-upstream-frameworks.sh` installs each group into its own Python 3.11 environment and verifies the public callable surface used by the adapter. Passing that surface check does not replace lifecycle, provider, benchmark, or security conformance.

## Common lifecycle

`FrameworkContext` exposes only bounded observable HuntEval operations:

- task creation, delegation, reassignment, start, completion, failure, and cancellation;
- operational messages;
- runner-mediated managed tools with strict action correlation;
- structured evidence, findings, and reviews;
- public run inputs and structured final submission.

Framework callbacks cannot retrieve evaluator-only fields or ground truth through this API. Missing framework observations remain unavailable.

The focused gate also runs one equivalent supervisor-worker lifecycle through CrewAI, LangGraph, AutoGen, Google ADK, and Semantic Kernel. It fixes the public run identity, objective, seed, agent budget, topology, managed tool, and expected protocol sequence. This proves connector-level control preservation only; it does not measure provider behavior, investigation quality, framework overhead, or scored benchmark performance.

## MCP interoperability

`McpSession` is a bounded MCP request processor attached to one `FrameworkContext`. A framework host starts the local MCP `stdio` service inside its HuntEval deployment process and forwards the resulting structured final submission through the normal adapter lifecycle.

The current implementation pins MCP revision `2025-11-25` and exposes only:

- `initialize` and `notifications/initialized`;
- `ping`;
- `tools/list`;
- `tools/call` for the finite `hunteval.*` catalog.

Sampling, elicitation, roots, server-provided prompts, arbitrary resources, subscriptions, dynamic tools, custom transports, legacy HTTP+SSE, and Streamable HTTP are unavailable. Remote transport requires the separate v0.7.2 network and authentication policy.

Generic MCP compatibility is not native framework support. The client must supply stable observable agent, task, action, evidence, and finding identities. Framework-specific topology or utilization metrics remain unavailable when the client cannot supply them authoritatively.

## Security properties

- The adapter never executes a scored tool directly.
- Every MCP session is bound to one HuntEval run context.
- Tool names and MCP capabilities use a fixed allowlist.
- Duplicate messages, invalid lifecycle transitions, unknown tools, unsupported methods, malformed JSON-RPC, oversized frames, and unstructured submissions fail closed.
- Client text and managed-tool results remain untrusted.
- No connector requests or records private chain of thought or hidden framework state.

## Local verification

```bash
./scripts/ci/v071-framework-connectors.sh
```

The gate requires no framework package, model-provider credential, external network access, or commercial tenant.
