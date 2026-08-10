# Connector support matrix

This matrix distinguishes implemented structural compatibility from exact upstream support and live commercial conformance. Only evidence in this repository is authoritative.

## Agent frameworks

| Framework | Candidate upstream version observed 2026-08-10 | Local evidence | Status |
|---|---:|---|---|
| CrewAI | 0.11.2 | R7 regression fixture and common lifecycle | supported R7 baseline; candidate version not yet package-conformant |
| LangGraph | 1.2.10 | structural adapter, paired deterministic fixture | implemented; upstream package conformance pending |
| AutoGen AgentChat | 0.7.5 | async structural adapter, paired deterministic fixture | implemented; upstream package conformance pending |
| Google ADK | 2.6.3 | local structural adapter, paired deterministic fixture | implemented; upstream package and remote A2A conformance pending |
| Semantic Kernel | 1.44.1 | preview structural adapter, paired deterministic fixture | preview; upstream package conformance pending |
| Generic MCP client | protocol revision 2025-11-25 | single-agent, multi-agent, malformed-input, lifecycle, and replay fixtures | implemented local interoperability; not native framework support |

Candidate versions are discovery inputs, not support claims. A framework becomes supported only when that exact package passes isolated installation, public-API, lifecycle, package, and benchmark conformance on a recorded revision.

## Commercial platforms

| Platform | Offline catalog and replay | Live read-only | Production scored or mutation |
|---|---|---|---|
| CrowdStrike Falcon | implemented | unavailable; authorized tenant evidence required | unavailable |
| Google Security Operations | implemented | unavailable; authorized tenant evidence required | unavailable |
| Microsoft Sentinel | implemented | unavailable; feasibility and tenant evidence required | unavailable |
| Elastic Security | implemented | unavailable; feasibility and tenant evidence required | unavailable |
| Cortex XSIAM | implemented | unavailable; feasibility and tenant evidence required | unavailable |

The Rust commercial boundary validates exact HTTPS origins, finite operations, public resolved addresses, opaque secret references, request limits, and response limits around an injected transport. No production HTTP transport or runtime secret resolver is enabled by the current support matrix.
