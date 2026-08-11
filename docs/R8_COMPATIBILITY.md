# HuntEval compatibility matrix

Compatibility does not grant runtime authority or certify deployment quality.

| Combination | Status | Components | Reason or limitation |
|---|---|---|---|
| commercial-preview | unavailable | connector.commercial-v0.7.2@0.7.2 preview | external_conformance_pending |
| crewai-retained | retained | connector.crewai-r7@R7 compatibility baseline, sdk.python-r7@0.1.0 | none |
| framework-pack-preview | preview | connector.framework-pack-v0.7.1@0.7.1 | release_closure_pending |
| linux-core-supported | supported | cli.hunteval-r7@R7 command surface, platform.linux-x86_64@x86_64-unknown-linux-gnu with Bubblewrap, protocol.deployment-jsonl@0.3, schema.contract-families@0.3 through 0.9 | none |
| macos-aarch64-package-preview | preview | platform.macos-aarch64@aarch64-apple-darwin | native_ci_pending |
| macos-x86_64-package-preview | preview | platform.macos-x86_64@x86_64-apple-darwin | native_ci_pending |
| mcp-preview | preview | connector.mcp-v0.7.1@MCP 2025-11-25 | release_closure_pending |
| windows-x86_64-package-preview | preview | platform.windows-x86_64@x86_64-pc-windows-msvc | native_ci_pending |

## Semantics

- `supported` and `retained` combinations reference only interfaces eligible in the exact freeze manifest.
- A native package preview proves no scored-execution, sandbox, or cross-topology capability. macOS and Windows remain unavailable for scored execution until an explicit future sandbox contract and its native gates exist.
- `preview` and `unavailable` combinations are not v1.0 stability claims and always state a rejection reason and limitation.
- Compatibility proves contract agreement only. It does not authorize capabilities, network access, scored execution, signing, or publication.
- Missing combinations are unavailable; the matrix never infers compatibility from similar versions or component names.
- Rollback retains the prior matrix and readers and never broadens support implicitly.

## Migration and rejection

The normative migration inventory is `examples/contracts/v1.0/migration-inventory.json`. Declared `adapt_in_memory` edges preserve source bytes and produce a content-addressed receipt for the exact normalized target bytes. `read_as_is` retains immutable readers. Undeclared, future-major, downgrade, private/public conversion, lossy, or ambiguous transitions are rejected without modifying the source.
