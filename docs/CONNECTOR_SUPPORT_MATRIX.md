# Connector support matrix

This matrix distinguishes implemented structural compatibility from exact upstream support and live commercial conformance. Only evidence in this repository is authoritative.

## Agent frameworks

| Framework | Candidate upstream version observed 2026-08-10 | Local evidence | Status |
|---|---:|---|---|
| CrewAI | 1.15.5 | R7 regression fixture, common lifecycle, and isolated Python 3.11 public-surface check | supported R7 baseline; full protected matrix pending |
| LangGraph | 1.2.10 | structural adapter, paired deterministic fixture, and isolated Python 3.11 public-surface check | implemented; protected package matrix pending |
| AutoGen AgentChat | 0.7.5 | async structural adapter, paired deterministic fixture, and isolated Python 3.11 public-surface check | implemented; protected package matrix pending |
| Google ADK | 2.6.3 | local structural adapter, paired deterministic fixture, and isolated Python 3.11 public-surface check | implemented; protected package matrix and remote A2A conformance pending |
| Semantic Kernel | 1.44.1 | preview structural adapter, paired deterministic fixture, and isolated Python 3.11 public-surface check | preview; protected package matrix pending |
| Generic MCP client | protocol revision 2025-11-25 | single-agent, multi-agent, malformed-input, lifecycle, and replay fixtures | implemented local interoperability; not native framework support |

Candidate versions are discovery inputs, not support claims. A framework becomes supported only when that exact package passes isolated installation, public-API, lifecycle, package, and benchmark conformance on a recorded revision.

## Commercial platforms

| Platform | Offline catalog and replay | Live read-only | Production scored or mutation |
|---|---|---|---|
| CrowdStrike Falcon | implemented | worker path implemented; externally enforced tenant conformance required | unavailable |
| Google Security Operations | implemented | worker path implemented; token issuance and tenant conformance required | unavailable |
| Microsoft Sentinel | implemented | worker path implemented; feasibility and tenant conformance required | unavailable |
| Elastic Security | implemented | worker path implemented; feasibility and tenant conformance required | unavailable |
| Cortex XSIAM | implemented | worker path implemented; feasibility and tenant conformance required | unavailable |

The Rust commercial boundary validates exact HTTPS origins, finite operations, public resolved addresses, opaque secret references, request limits, and response limits. An HTTPS transport and one-call worker exist, but the support matrix does not enable them without external egress enforcement, protected short-lived token delivery, and passing authorized live attestations.
