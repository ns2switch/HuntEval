# v0.7.1 implementation evidence

## Status

v0.7.1 is implemented locally but not release-complete. R7 remains complete and unchanged.

## Implemented behavior

- one framework-neutral lifecycle with bounded task, delegation, communication, managed-tool, evidence, finding, and terminal operations;
- CrewAI regression migration;
- structural LangGraph and async AutoGen AgentChat adapters, a Google ADK adapter aligned to the documented `Runner.run` surface, and a Semantic Kernel preview adapter;
- MCP revision `2025-11-25` session processing over local newline-delimited `stdio` with a fixed eight-tool `hunteval.*` catalog;
- single-agent and observable multi-agent MCP fixtures;
- deterministic paired lifecycle fixtures across the four new native adapters;
- a paired supervisor-worker conformance matrix across CrewAI and the four new adapters with equivalent run identity, objective, seed, agent budget, topology, managed tool, and normalized protocol sequence;
- malformed input, duplicate identity, lifecycle, unsupported capability, unknown tool, structured-output, process-boundary, and replay tests;
- dedicated local and GitHub Actions `Framework connectors` gate;
- package inventory inclusion through the existing reproducible R7 wheel gate.
- isolated optional dependency groups pinned to the candidate upstream versions;
- an upstream public-surface conformance harness for CrewAI, LangGraph, AutoGen AgentChat, Google ADK, and Semantic Kernel.

## Local acceptance evidence

The following passed on the current worktree:

```text
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/quality.sh all
```

The dedicated gate currently runs eleven focused framework and MCP tests. The R7 gate runs the complete Python suite and verifies two independently built wheel inventories.

## Open release evidence

- passing protected Python 3.11 execution of the complete isolated upstream package matrix (local isolated Python 3.11 inspection passed CrewAI 1.15.5, LangGraph 1.2.10, AutoGen AgentChat 0.7.5, Google ADK 2.6.3, and Semantic Kernel 1.44.1; the initial CrewAI 0.11.2 candidate was rejected because its public `kickoff` surface was incompatible, and Semantic Kernel requires an explicit compatible protobuf range);
- provider-backed non-scored smoke evidence where appropriate;
- full scored paired benchmark matrices rather than the implemented deterministic lifecycle and topology-equivalence fixtures;
- protected-branch requirement and passing remote GitHub Actions evidence for the exact closure revision;
- migration and rollback rehearsal against the published package.

These omissions prevent a `complete` status. They do not alter R7 evidence.
