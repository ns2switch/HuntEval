from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Any

from ._validation import ContractError, digest, exact_keys, identifier, positive_int

SOURCE_KINDS = {
    "run", "benchmark", "report", "topology", "diagnosis", "improvement", "document"
}
SCOPES = {"evaluator_analytics", "deployment_visible"}
CAPABILITIES = {
    "public_episode_read", "managed_tool_request", "process_spawn", "local_read_only_data"
}
LIMIT_KEYS = {
    "wall_time_ms", "max_input_bytes", "max_output_bytes", "max_processes", "max_concurrency"
}


def _bounded_strings(value: Any, allowed: set[str], maximum: int) -> tuple[str, ...]:
    if not isinstance(value, list) or len(value) > maximum or any(
        not isinstance(item, str) or item not in allowed for item in value
    ):
        raise ContractError("string inventory is invalid")
    if len(set(value)) != len(value):
        raise ContractError("string inventory contains duplicates")
    return tuple(value)


def _limits(value: Any) -> dict[str, int]:
    if not isinstance(value, dict):
        raise ContractError("extension limits must be an object")
    exact_keys(value, LIMIT_KEYS)
    return {key: positive_int(limit) for key, limit in value.items()}


@dataclass(frozen=True, slots=True)
class AnalyticalQuery:
    index_sha256: str
    scope: str
    terms: tuple[str, ...]
    source_kinds: tuple[str, ...] | None
    max_results: int

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> AnalyticalQuery:
        exact_keys(
            value,
            {"schema_version", "index_sha256", "scope", "terms", "source_kinds", "max_results"},
        )
        if value["schema_version"] != "0.9" or value["scope"] not in SCOPES:
            raise ContractError("query version or scope is unsupported")
        terms = value["terms"]
        if not isinstance(terms, list) or not 1 <= len(terms) <= 16 or any(
            not isinstance(term, str) or not term.strip() or len(term) > 128 for term in terms
        ):
            raise ContractError("query terms are invalid")
        kinds = value["source_kinds"]
        parsed_kinds = None if kinds is None else _bounded_strings(kinds, SOURCE_KINDS, 7)
        maximum = positive_int(value["max_results"])
        if maximum > 100:
            raise ContractError("query result limit is invalid")
        return cls(
            digest(value["index_sha256"]), value["scope"], tuple(terms), parsed_kinds, maximum
        )


@dataclass(frozen=True, slots=True)
class AnalyticalMatch:
    source_id: str
    source_kind: str
    artifact_sha256: str
    field: str
    excerpt: str

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> AnalyticalMatch:
        exact_keys(value, {"source_id", "source_kind", "artifact_sha256", "field", "excerpt"})
        if value["source_kind"] not in SOURCE_KINDS:
            raise ContractError("analytical source kind is invalid")
        field, excerpt = value["field"], value["excerpt"]
        if not isinstance(field, str) or not field or len(field) > 128:
            raise ContractError("analytical result field is invalid")
        if not isinstance(excerpt, str) or len(excerpt.encode()) > 1_024:
            raise ContractError("analytical result excerpt is invalid")
        return cls(
            identifier(value["source_id"]),
            value["source_kind"],
            digest(value["artifact_sha256"]),
            field,
            excerpt,
        )


@dataclass(frozen=True, slots=True)
class AnalyticalResult:
    query_sha256: str
    index_sha256: str
    matches: tuple[AnalyticalMatch, ...]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> AnalyticalResult:
        exact_keys(value, {"schema_version", "query_sha256", "index_sha256", "matches"})
        if (
            value["schema_version"] != "0.9"
            or not isinstance(value["matches"], list)
            or len(value["matches"]) > 100
        ):
            raise ContractError("analytical result is invalid")
        matches = tuple(
            AnalyticalMatch.from_dict(item)
            for item in value["matches"]
            if isinstance(item, dict)
        )
        if len(matches) != len(value["matches"]):
            raise ContractError("analytical match inventory is invalid")
        return cls(digest(value["query_sha256"]), digest(value["index_sha256"]), matches)


@dataclass(frozen=True, slots=True)
class RetrievalAuditEvent:
    sequence: int
    recorded_at: str
    scope: str
    query_sha256: str
    result_sha256: str
    index_sha256: str
    latency_ms: int
    cost_microunits: int | None
    previous_event_sha256: str | None

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> RetrievalAuditEvent:
        exact_keys(
            value,
            {
                "schema_version", "sequence", "recorded_at", "scope", "query_sha256",
                "result_sha256", "index_sha256", "latency_ms", "cost_microunits",
                "previous_event_sha256",
            },
        )
        if value["schema_version"] != "0.9" or value["scope"] not in SCOPES:
            raise ContractError("audit version or scope is invalid")
        timestamp = value["recorded_at"]
        if not isinstance(timestamp, str) or not timestamp.endswith("Z"):
            raise ContractError("audit timestamp must be UTC")
        try:
            datetime.fromisoformat(timestamp.replace("Z", "+00:00"))
        except ValueError as error:
            raise ContractError("audit timestamp is invalid") from error
        latency = value["latency_ms"]
        cost = value["cost_microunits"]
        if not isinstance(latency, int) or isinstance(latency, bool) or latency < 0:
            raise ContractError("audit latency is invalid")
        if cost is not None and (
            not isinstance(cost, int) or isinstance(cost, bool) or cost < 0
        ):
            raise ContractError("audit cost is invalid")
        previous = value["previous_event_sha256"]
        return cls(
            positive_int(value["sequence"]),
            timestamp,
            value["scope"],
            digest(value["query_sha256"]),
            digest(value["result_sha256"]),
            digest(value["index_sha256"]),
            latency,
            cost,
            None if previous is None else digest(previous),
        )


@dataclass(frozen=True, slots=True)
class ExtensionCapabilityPolicy:
    policy_sha256: str
    allowed_capabilities: tuple[str, ...]
    maximum_limits: dict[str, int]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ExtensionCapabilityPolicy:
        exact_keys(
            value,
            {"schema_version", "policy_sha256", "allowed_capabilities", "network", "maximum_limits"},
        )
        if value["schema_version"] != "0.9" or value["network"] != "denied":
            raise ContractError("extension policy version or network rule is unsupported")
        return cls(
            digest(value["policy_sha256"]),
            _bounded_strings(value["allowed_capabilities"], CAPABILITIES, 4),
            _limits(value["maximum_limits"]),
        )


@dataclass(frozen=True, slots=True)
class ExtensionResolution:
    manifest_sha256: str
    policy_sha256: str
    granted_capabilities: tuple[str, ...]
    status: str
    reasons: tuple[str, ...]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ExtensionResolution:
        exact_keys(
            value,
            {
                "schema_version", "manifest_sha256", "policy_sha256",
                "granted_capabilities", "status", "reasons",
            },
        )
        if value["schema_version"] != "0.9" or value["status"] not in {"eligible", "rejected"}:
            raise ContractError("extension resolution version or status is invalid")
        reasons = value["reasons"]
        if (
            not isinstance(reasons, list)
            or len(reasons) > 32
            or any(not isinstance(item, str) for item in reasons)
        ):
            raise ContractError("extension resolution reasons are invalid")
        capabilities = _bounded_strings(value["granted_capabilities"], CAPABILITIES, 4)
        if value["status"] == "rejected" and capabilities:
            raise ContractError("a rejected extension cannot receive capabilities")
        return cls(
            digest(value["manifest_sha256"]),
            digest(value["policy_sha256"]),
            capabilities,
            value["status"],
            tuple(reasons),
        )


@dataclass(frozen=True, slots=True)
class ExtensionConformanceResult:
    manifest_sha256: str
    executable_sha256: str
    policy_sha256: str
    protocol_transcript_sha256: str | None
    status: str
    checks: tuple[str, ...]
    reasons: tuple[str, ...]

    @classmethod
    def from_dict(cls, value: dict[str, Any]) -> ExtensionConformanceResult:
        exact_keys(
            value,
            {
                "schema_version", "manifest_sha256", "executable_sha256", "policy_sha256",
                "protocol_transcript_sha256", "status", "checks", "reasons",
            },
        )
        if value["schema_version"] != "0.9" or value["status"] not in {
            "conformant", "rejected"
        }:
            raise ContractError("extension conformance version or status is invalid")
        checks = value["checks"]
        reasons = value["reasons"]
        if any(
            not isinstance(items, list)
            or len(items) > 32
            or len(set(items)) != len(items)
            or any(not isinstance(item, str) for item in items)
            for items in (checks, reasons)
        ):
            raise ContractError("extension conformance checks or reasons are invalid")
        transcript = value["protocol_transcript_sha256"]
        return cls(
            digest(value["manifest_sha256"]),
            digest(value["executable_sha256"]),
            digest(value["policy_sha256"]),
            None if transcript is None else digest(transcript),
            value["status"],
            tuple(checks),
            tuple(reasons),
        )
