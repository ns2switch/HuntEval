"""Bounded local MCP interoperability adapter for HuntEval frameworks."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import IO, Any, Mapping

from .framework import FrameworkContext, normalize_submission
from .mcp_catalog import tool_catalog
from .protocol import ProtocolError

MCP_PROTOCOL_REVISION = "2025-11-25"
MAX_MCP_LINE_BYTES = 1_048_576
MAX_MCP_REQUESTS = 4_096


class McpProtocolError(ValueError):
    """A safe MCP request failed validation or HuntEval policy."""

    def __init__(self, code: int, message: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass(slots=True)
class McpSession:
    """One MCP client session bound to one HuntEval framework context."""

    context: FrameworkContext
    initialized: bool = False
    client_ready: bool = False
    requests: int = 0
    messages: int = 0
    seen_request_ids: set[str] = field(default_factory=set)
    _submission: dict[str, Any] | None = None

    def handle(self, message: Any) -> dict[str, Any] | None:
        """Handle one decoded JSON-RPC message and return an optional response."""
        try:
            return self._handle(message)
        except McpProtocolError as error:
            if isinstance(message, Mapping) and "id" not in message:
                return None
            return _error(_safe_id(message), error.code, str(error))
        except (ProtocolError, ValueError, TypeError):
            if isinstance(message, Mapping) and "id" not in message:
                return None
            return _error(_safe_id(message), -32001, "HuntEval rejected the MCP request")

    def serve(self, input_stream: IO[str], output_stream: IO[str]) -> None:
        """Serve newline-delimited MCP JSON-RPC over supervised local stdio."""
        while True:
            line = input_stream.readline(MAX_MCP_LINE_BYTES + 1)
            if not line:
                return
            if len(line.encode("utf-8")) > MAX_MCP_LINE_BYTES or not line.endswith("\n"):
                self._write(output_stream, _error(None, -32700, "MCP frame is oversized or incomplete"))
                return
            try:
                message = json.loads(line)
            except json.JSONDecodeError:
                self._write(output_stream, _error(None, -32700, "MCP frame is not valid JSON"))
                continue
            response = self.handle(message)
            if response is not None:
                self._write(output_stream, response)

    def take_submission(self) -> dict[str, Any]:
        """Return the one structured submission created by the MCP client."""
        if self._submission is None:
            raise McpProtocolError(-32001, "MCP client did not set a final submission")
        return dict(self._submission)

    def _handle(self, message: Any) -> dict[str, Any] | None:
        if not isinstance(message, dict) or message.get("jsonrpc") != "2.0":
            raise McpProtocolError(-32600, "MCP request must be a JSON-RPC 2.0 object")
        if self.messages >= MAX_MCP_REQUESTS:
            raise McpProtocolError(-32001, "MCP message limit is exhausted")
        self.messages += 1
        if set(message) - {"jsonrpc", "id", "method", "params"}:
            raise McpProtocolError(-32600, "MCP request contains unknown fields")
        method = message.get("method")
        if not isinstance(method, str) or not method or len(method.encode("utf-8")) > 256:
            raise McpProtocolError(-32600, "MCP method is missing or oversized")
        request_id = message.get("id")
        if request_id is None:
            return self._notification(method, message.get("params", {}))
        identity = _request_identity(request_id)
        if identity in self.seen_request_ids:
            raise McpProtocolError(-32600, "MCP request identity is duplicated")
        self.seen_request_ids.add(identity)
        self.requests += 1
        result = self._request(method, message.get("params", {}))
        return {"jsonrpc": "2.0", "id": request_id, "result": result}

    def _notification(self, method: str, params: Any) -> None:
        if method == "notifications/initialized":
            _mapping(params, set())
            if not self.initialized or self.client_ready:
                raise McpProtocolError(-32600, "MCP initialized notification is out of order")
            self.client_ready = True
            return None
        raise McpProtocolError(-32601, "MCP notification is not supported")

    def _request(self, method: str, params: Any) -> dict[str, Any]:
        if method == "initialize":
            return self._initialize(params)
        if not self.client_ready:
            raise McpProtocolError(-32001, "MCP session is not initialized")
        if method == "ping":
            _mapping(params, set())
            return {}
        if method == "tools/list":
            _mapping(params, set())
            return {"tools": tool_catalog()}
        if method == "tools/call":
            return self._call_tool(params)
        raise McpProtocolError(-32601, "MCP method is not supported")

    def _initialize(self, params: Any) -> dict[str, Any]:
        value = _mapping(params, {"protocolVersion", "capabilities", "clientInfo"})
        if self.initialized or value.get("protocolVersion") != MCP_PROTOCOL_REVISION:
            raise McpProtocolError(-32602, "MCP protocol revision is unsupported")
        capabilities = _mapping(value.get("capabilities"), set())
        client_info = _mapping(value.get("clientInfo"), {"name", "version"})
        _bounded_string(client_info.get("name"), "MCP client name", 128)
        _bounded_string(client_info.get("version"), "MCP client version", 64)
        if capabilities:
            raise McpProtocolError(-32602, "MCP client capabilities are unsupported")
        self.initialized = True
        return {
            "protocolVersion": MCP_PROTOCOL_REVISION,
            "capabilities": {"tools": {"listChanged": False}},
            "serverInfo": {"name": "hunteval", "version": "0.1.0"},
        }

    def _call_tool(self, params: Any) -> dict[str, Any]:
        value = _mapping(params, {"name", "arguments"})
        name = _bounded_string(value.get("name"), "MCP tool name", 128)
        arguments = _mapping(value.get("arguments"), None)
        result = self._dispatch(name, arguments)
        return {
            "content": [{"type": "text", "text": json.dumps(result, separators=(",", ":"))}],
            "structuredContent": result,
            "isError": False,
        }

    def _dispatch(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        if name == "hunteval.task.create":
            value = _mapping(arguments, {"agent_id", "task_id", "objective"})
            self.context.create_task(value["agent_id"], value["task_id"], value["objective"])
            return {"accepted": True}
        if name == "hunteval.task.delegate":
            value = _mapping(arguments, {"agent_id", "task_id", "target_agent_id"})
            self.context.delegate_task(
                value["agent_id"], value["task_id"], value["target_agent_id"]
            )
            return {"accepted": True}
        if name == "hunteval.task.start":
            value = _mapping(arguments, {"agent_id", "task_id"})
            self.context.start_task(value["agent_id"], value["task_id"])
            return {"accepted": True}
        if name == "hunteval.task.complete":
            value = _mapping(arguments, {"agent_id", "task_id"})
            self.context.complete_task(value["agent_id"], value["task_id"])
            return {"accepted": True}
        if name == "hunteval.tool.call":
            required = {"agent_id", "task_id", "action_id", "tool", "purpose", "arguments"}
            value = _mapping(arguments, required)
            result = self.context.managed_tool(
                agent_id=value["agent_id"],
                task_id=value["task_id"],
                action_id=value["action_id"],
                tool=value["tool"],
                purpose=value["purpose"],
                arguments=_mapping(value["arguments"], None),
            )
            return dict(result)
        if name == "hunteval.evidence.share":
            value = _mapping(arguments, {"agent_id", "task_id", "evidence"})
            self.context.share_evidence(
                value["agent_id"], value["task_id"], _mapping(value["evidence"], None)
            )
            return {"accepted": True}
        if name == "hunteval.finding.propose":
            value = _mapping(arguments, {"agent_id", "task_id", "finding"})
            self.context.propose_finding(
                value["agent_id"], value["task_id"], _mapping(value["finding"], None)
            )
            return {"accepted": True}
        if name == "hunteval.submission.set":
            value = _mapping(arguments, {"submission"})
            if self._submission is not None:
                raise McpProtocolError(-32001, "MCP final submission is already set")
            self._submission = normalize_submission(_mapping(value["submission"], None))
            return {"accepted": True}
        raise McpProtocolError(-32602, "MCP tool is not declared")

    @staticmethod
    def _write(output_stream: IO[str], response: dict[str, Any]) -> None:
        encoded = json.dumps(response, separators=(",", ":"), ensure_ascii=False)
        if len(encoded.encode("utf-8")) + 1 > MAX_MCP_LINE_BYTES:
            encoded = json.dumps(
                _error(response.get("id"), -32603, "MCP response exceeds the byte limit"),
                separators=(",", ":"),
            )
        output_stream.write(encoded + "\n")
        output_stream.flush()


def _mapping(value: Any, keys: set[str] | None) -> dict[str, Any]:
    if not isinstance(value, Mapping) or len(value) > 4_096:
        raise McpProtocolError(-32602, "MCP parameters must be a bounded object")
    result = dict(value)
    if keys is not None and set(result) != keys:
        raise McpProtocolError(-32602, "MCP parameter fields do not match the contract")
    return result


def _bounded_string(value: Any, name: str, maximum: int) -> str:
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > maximum:
        raise McpProtocolError(-32602, f"{name} is missing or oversized")
    return value


def _request_identity(value: Any) -> str:
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        raise McpProtocolError(-32600, "MCP request identity must be a string or integer")
    text = str(value)
    if not text or len(text.encode("utf-8")) > 128:
        raise McpProtocolError(-32600, "MCP request identity is empty or oversized")
    return f"{type(value).__name__}:{text}"


def _safe_id(message: Any) -> str | int | None:
    if not isinstance(message, Mapping):
        return None
    value = message.get("id")
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        return None
    return value


def _error(identity: str | int | None, code: int, message: str) -> dict[str, Any]:
    return {
        "jsonrpc": "2.0",
        "id": identity,
        "error": {"code": code, "message": message},
    }
