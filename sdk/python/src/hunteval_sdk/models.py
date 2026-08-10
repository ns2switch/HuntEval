from __future__ import annotations

from dataclasses import dataclass
import re
from typing import Any, Literal

from ._validation import ContractError, digest, exact_keys, identifier, positive_int

SourceKind = Literal["run", "benchmark", "report", "topology", "diagnosis", "improvement", "document"]
CorpusScope = Literal["evaluator_analytics", "deployment_visible"]
ExtensionKind = Literal["managed_tool", "deployment_adapter"]
VERSION = re.compile(r"^[0-9]+\.[0-9]+$")
CAPABILITIES = {
    "public_episode_read", "managed_tool_request", "process_spawn", "local_read_only_data"
}


@dataclass(frozen=True, slots=True)
class CorpusSource:
    id: str
    kind: SourceKind
    path: str
    artifact_sha256: str
    verified: bool = True

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> CorpusSource:
        exact_keys(value, {"id", "kind", "path", "artifact_sha256", "verified"})
        kinds = {"run", "benchmark", "report", "topology", "diagnosis", "improvement", "document"}
        if value["kind"] not in kinds or value["verified"] is not True:
            raise ContractError("source kind or verification status is invalid")
        path = value["path"]
        if not isinstance(path, str) or not path or path.startswith("/") or ".." in path.split("/"):
            raise ContractError("source path is unsafe")
        return cls(identifier(value["id"]), value["kind"], path, digest(value["artifact_sha256"]))


@dataclass(frozen=True, slots=True)
class AnalyticalCorpus:
    id: str
    scope: CorpusScope
    sources: tuple[CorpusSource, ...]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> AnalyticalCorpus:
        exact_keys(value, {"schema_version", "id", "scope", "sources"})
        if value["schema_version"] != "0.9" or value["scope"] not in {
            "evaluator_analytics", "deployment_visible"
        }:
            raise ContractError("corpus version or scope is unsupported")
        raw_sources = value["sources"]
        if not isinstance(raw_sources, list) or not 1 <= len(raw_sources) <= 10_000:
            raise ContractError("source count is outside the supported bound")
        sources = tuple(CorpusSource.from_dict(item) for item in raw_sources if isinstance(item, dict))
        if len(sources) != len(raw_sources) or len({item.id for item in sources}) != len(sources):
            raise ContractError("source inventory is malformed or duplicated")
        if value["scope"] == "deployment_visible" and any(item.kind != "document" for item in sources):
            raise ContractError("deployment-visible corpora can contain documents only")
        return cls(identifier(value["id"]), value["scope"], sources)

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": "0.9",
            "id": self.id,
            "scope": self.scope,
            "sources": [
                {"id": item.id, "kind": item.kind, "path": item.path, "artifact_sha256": item.artifact_sha256, "verified": True}
                for item in self.sources
            ],
        }


@dataclass(frozen=True, slots=True)
class ExtensionManifest:
    id: str
    kind: ExtensionKind
    executable_sha256: str
    supported_versions: tuple[str, ...]
    requested_capabilities: tuple[str, ...]
    tools: tuple[str, ...]
    limits: dict[str, int]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ExtensionManifest:
        exact_keys(value, {"schema_version", "id", "kind", "executable_sha256", "supported_versions", "requested_capabilities", "network", "tools", "limits"})
        if value["schema_version"] != "0.9" or value["kind"] not in {"managed_tool", "deployment_adapter"} or value["network"] != "denied":
            raise ContractError("extension version, kind, or network policy is unsupported")
        versions = value["supported_versions"]
        capabilities = value["requested_capabilities"]
        tools = value["tools"]
        limits = value["limits"]
        if not isinstance(versions, list) or not 1 <= len(versions) <= 16:
            raise ContractError("supported version inventory is invalid")
        if (
            any(not isinstance(version, str) or VERSION.fullmatch(version) is None for version in versions)
            or len(set(versions)) != len(versions)
        ):
            raise ContractError("supported version inventory is malformed or duplicated")
        if (
            not isinstance(capabilities, list)
            or len(capabilities) > 4
            or any(item not in CAPABILITIES for item in capabilities)
            or len(set(capabilities)) != len(capabilities)
            or not isinstance(tools, list)
            or len(tools) > 64
            or not isinstance(limits, dict)
        ):
            raise ContractError("extension inventory is invalid")
        exact_keys(limits, {"wall_time_ms", "max_input_bytes", "max_output_bytes", "max_processes", "max_concurrency"})
        checked_limits = {name: positive_int(limit) for name, limit in limits.items()}
        checked_tools = tuple(identifier(tool) for tool in tools)
        if len(set(checked_tools)) != len(checked_tools):
            raise ContractError("extension tool inventory is duplicated")
        if value["kind"] == "managed_tool" and not checked_tools:
            raise ContractError("managed-tool adapter must declare a tool")
        if value["kind"] == "deployment_adapter" and checked_tools:
            raise ContractError("deployment adapter cannot declare tools")
        return cls(identifier(value["id"]), value["kind"], digest(value["executable_sha256"]), tuple(versions), tuple(capabilities), checked_tools, checked_limits)

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": "0.9", "id": self.id, "kind": self.kind,
            "executable_sha256": self.executable_sha256,
            "supported_versions": list(self.supported_versions),
            "requested_capabilities": list(self.requested_capabilities),
            "network": "denied", "tools": list(self.tools), "limits": dict(self.limits),
        }
