# v0.7.1 implementation evidence

## Status

v0.7.1 is implemented locally but not release-complete. R7 remains complete and unchanged.

## Implemented behavior

- one framework-neutral lifecycle with bounded task, delegation, communication, managed-tool, evidence, finding, and terminal operations;
- CrewAI regression migration;
- structural LangGraph, async AutoGen AgentChat, local Google ADK, and Semantic Kernel preview adapters;
- MCP revision `2025-11-25` session processing over local newline-delimited `stdio` with a fixed eight-tool `hunteval.*` catalog;
- single-agent and observable multi-agent MCP fixtures;
- deterministic paired lifecycle fixtures across the four new native adapters;
- malformed input, duplicate identity, lifecycle, unsupported capability, unknown tool, structured-output, process-boundary, and replay tests;
- dedicated local and GitHub Actions `Framework connectors` gate;
- package inventory inclusion through the existing reproducible R7 wheel gate.

## Local acceptance evidence

The following passed on the current worktree:

```text
./scripts/ci/v071-framework-connectors.sh
./scripts/ci/r7-extensions.sh
./scripts/ci/quality.sh all
```

The dedicated gate currently runs ten focused framework and MCP tests. The R7 gate runs the complete Python suite and verifies two independently built wheel inventories.

## Open release evidence

- isolated installation and public-API conformance against the exact candidate framework versions in `CONNECTOR_SUPPORT_MATRIX.md`;
- provider-backed non-scored smoke evidence where appropriate;
- full paired benchmark matrices rather than lifecycle-equivalence fixtures;
- protected-branch requirement and passing remote GitHub Actions evidence for the exact closure revision;
- migration and rollback rehearsal against the published package.

These omissions prevent a `complete` status. They do not alter R7 evidence.
