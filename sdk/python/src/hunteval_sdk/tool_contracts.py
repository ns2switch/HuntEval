from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

from ._validation import ContractError, exact_keys, identifier

MAX_MESSAGE_BYTES = 1_048_576


def _bounded(value: dict[str, Any]) -> None:
    encoded = json.dumps(value, separators=(",", ":"), ensure_ascii=False).encode()
    if len(encoded) > MAX_MESSAGE_BYTES:
        raise ContractError("managed-tool adapter message exceeds its byte limit")


@dataclass(frozen=True, slots=True)
class ManagedToolAdapterRequest:
    request_id: str
    tool: str
    arguments: Any

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ManagedToolAdapterRequest:
        exact_keys(value, {"schema_version", "request_id", "tool", "arguments"})
        if value["schema_version"] != "0.9":
            raise ContractError("managed-tool request version is unsupported")
        _bounded(value)
        return cls(identifier(value["request_id"]), identifier(value["tool"]), value["arguments"])

    def to_dict(self) -> dict[str, Any]:
        value = {
            "schema_version": "0.9",
            "request_id": self.request_id,
            "tool": self.tool,
            "arguments": self.arguments,
        }
        _bounded(value)
        return value


@dataclass(frozen=True, slots=True)
class ManagedToolAdapterResponse:
    status: str
    request_id: str
    result: Any | None
    reason_code: str | None

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ManagedToolAdapterResponse:
        status = value.get("status")
        if status == "success":
            exact_keys(value, {"status", "schema_version", "request_id", "result"})
            result, reason = value["result"], None
        elif status == "error":
            exact_keys(value, {"status", "schema_version", "request_id", "reason_code"})
            result, reason = None, identifier(value["reason_code"])
        else:
            raise ContractError("managed-tool response status is unsupported")
        if value["schema_version"] != "0.9":
            raise ContractError("managed-tool response version is unsupported")
        _bounded(value)
        return cls(status, identifier(value["request_id"]), result, reason)
