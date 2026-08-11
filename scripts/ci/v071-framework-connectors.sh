#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

seeded_failure v071-framework-connectors
python3 -c 'import sys; raise SystemExit(0 if sys.version_info >= (3, 11) else "Python 3.11 or newer is required")'

PYTHONPATH=sdk/python/src python3 -m unittest \
    sdk/python/tests/test_crewai.py \
    sdk/python/tests/test_framework_connectors.py \
    sdk/python/tests/test_framework_matrix.py \
    sdk/python/tests/test_mcp.py \
    -v
python3 -m compileall -q sdk/python/src sdk/python/tests

PYTHONPATH=sdk/python/src python3 - <<'PY'
import hunteval_sdk
from hunteval_sdk.mcp_catalog import tool_catalog

required = {
    "AutoGenAdapter",
    "CrewAIAdapter",
    "GoogleAdkAdapter",
    "LangGraphAdapter",
    "McpSession",
    "SemanticKernelPreviewAdapter",
}
missing = sorted(name for name in required if not hasattr(hunteval_sdk, name))
if missing:
    raise SystemExit(f"missing public connector exports: {missing}")

names = [tool["name"] for tool in tool_catalog()]
if len(names) != len(set(names)) or not names or any(not name.startswith("hunteval.") for name in names):
    raise SystemExit("MCP tool catalog is empty, duplicated, or outside the HuntEval namespace")
PY
