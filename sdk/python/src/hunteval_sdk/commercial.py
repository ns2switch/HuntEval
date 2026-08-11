"""Offline-first commercial connector contracts and deterministic replay."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Any, Mapping

from ._validation import digest, identifier
from .commercial_catalog import operations_for

MAX_ARGUMENT_BYTES = 65_536
MAX_RECORDS = 10_000
FORBIDDEN_ARGUMENT_FIELDS = {
    "endpoint",
    "headers",
    "host",
    "method",
    "url",
}
FORBIDDEN_RESPONSE_FIELDS: set[str] = set()


class CommercialConnectorError(ValueError):
    """A commercial connector request failed bounded policy validation."""


@dataclass(frozen=True, slots=True)
class CommercialRequest:
    platform: str
    operation: str
    tenant_alias: str
    region: str
    arguments: Mapping[str, Any]

    def __post_init__(self) -> None:
        identifier(self.platform)
        identifier(self.operation)
        identifier(self.tenant_alias)
        identifier(self.region)
        if self.operation not in operations_for(self.platform):
            raise CommercialConnectorError("commercial operation is not read-only or supported")
        _bounded_json_object(self.arguments, FORBIDDEN_ARGUMENT_FIELDS)

    def canonical_bytes(self) -> bytes:
        value = {
            "platform": self.platform,
            "operation": self.operation,
            "tenant_alias": self.tenant_alias,
            "region": self.region,
            "arguments": dict(self.arguments),
        }
        return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")

    @property
    def sha256(self) -> str:
        return hashlib.sha256(self.canonical_bytes()).hexdigest()


@dataclass(frozen=True, slots=True)
class CommercialFixture:
    fixture_id: str
    request_sha256: str
    response: Mapping[str, Any]
    response_sha256: str

    def __post_init__(self) -> None:
        identifier(self.fixture_id)
        digest(self.request_sha256)
        digest(self.response_sha256)
        response = _bounded_json_object(self.response, FORBIDDEN_RESPONSE_FIELDS)
        actual = hashlib.sha256(_canonical(response)).hexdigest()
        if actual != self.response_sha256:
            raise CommercialConnectorError("commercial fixture response digest does not match")


@dataclass(frozen=True, slots=True)
class CommercialResult:
    platform: str
    operation: str
    tenant_alias: str
    region: str
    mode: str
    request_sha256: str
    response_sha256: str
    records: tuple[Mapping[str, Any], ...]
    truncated: bool
    more_available: bool

    def to_dict(self) -> dict[str, Any]:
        return {
            "platform": self.platform,
            "operation": self.operation,
            "tenant_alias": self.tenant_alias,
            "region": self.region,
            "mode": self.mode,
            "request_sha256": self.request_sha256,
            "response_sha256": self.response_sha256,
            "records": [dict(record) for record in self.records],
            "truncated": self.truncated,
            "more_available": self.more_available,
        }


@dataclass(frozen=True, slots=True)
class FixtureReplayConnector:
    """A network-free commercial connector over one exact fixture inventory."""

    platform: str
    fixtures: Mapping[str, CommercialFixture]

    def __post_init__(self) -> None:
        identifier(self.platform)
        operations_for(self.platform)
        if not self.fixtures or len(self.fixtures) > 4_096:
            raise CommercialConnectorError("commercial fixture inventory is empty or oversized")
        for request_hash, fixture in self.fixtures.items():
            if digest(request_hash) != fixture.request_sha256:
                raise CommercialConnectorError("commercial fixture inventory key is invalid")

    def execute(self, request: CommercialRequest) -> CommercialResult:
        if request.platform != self.platform:
            raise CommercialConnectorError("commercial request targets another platform")
        fixture = self.fixtures.get(request.sha256)
        if fixture is None:
            raise CommercialConnectorError("no exact offline fixture exists for the request")
        response = dict(fixture.response)
        if set(response) != {"records", "truncated", "more_available"}:
            raise CommercialConnectorError("commercial fixture response fields are unsupported")
        records = response["records"]
        if not isinstance(records, list) or len(records) > MAX_RECORDS:
            raise CommercialConnectorError("commercial fixture record count is invalid")
        normalized: list[Mapping[str, Any]] = []
        for record in records:
            normalized.append(_bounded_json_object(record, FORBIDDEN_RESPONSE_FIELDS))
        if not isinstance(response["truncated"], bool) \
                or not isinstance(response["more_available"], bool):
            raise CommercialConnectorError("commercial fixture pagination flags are invalid")
        return CommercialResult(
            platform=request.platform,
            operation=request.operation,
            tenant_alias=request.tenant_alias,
            region=request.region,
            mode="fixture_replay",
            request_sha256=request.sha256,
            response_sha256=fixture.response_sha256,
            records=tuple(normalized),
            truncated=response["truncated"],
            more_available=response["more_available"],
        )


def build_fixture(
    fixture_id: str, request: CommercialRequest, response: Mapping[str, Any]
) -> CommercialFixture:
    """Build a content-addressed synthetic fixture after bounded validation."""
    normalized = _bounded_json_object(response, FORBIDDEN_RESPONSE_FIELDS)
    return CommercialFixture(
        fixture_id=fixture_id,
        request_sha256=request.sha256,
        response=normalized,
        response_sha256=hashlib.sha256(_canonical(normalized)).hexdigest(),
    )


def _bounded_json_object(value: Any, forbidden: set[str]) -> dict[str, Any]:
    if not isinstance(value, Mapping) or len(value) > 4_096:
        raise CommercialConnectorError("commercial value must be a bounded object")
    result = dict(value)
    _validate_json(result, forbidden, 0)
    try:
        encoded = _canonical(result)
    except (TypeError, ValueError) as error:
        raise CommercialConnectorError("commercial value must contain finite JSON") from error
    if len(encoded) > MAX_ARGUMENT_BYTES:
        raise CommercialConnectorError("commercial value exceeds the byte limit")
    return result


def _validate_json(value: Any, forbidden: set[str], depth: int) -> None:
    if depth > 16:
        raise CommercialConnectorError("commercial value exceeds nesting limits")
    if isinstance(value, Mapping):
        if len(value) > 4_096:
            raise CommercialConnectorError("commercial object exceeds property limits")
        for key, nested in value.items():
            if not isinstance(key, str) or key.lower() in forbidden or _sensitive_key(key):
                raise CommercialConnectorError("commercial value contains a prohibited field")
            _validate_json(nested, forbidden, depth + 1)
        return
    if isinstance(value, list):
        if len(value) > MAX_RECORDS:
            raise CommercialConnectorError("commercial array exceeds item limits")
        for nested in value:
            _validate_json(nested, forbidden, depth + 1)
        return
    if value is None or isinstance(value, (str, int, float, bool)):
        return
    raise CommercialConnectorError("commercial value contains a non-JSON value")


def _canonical(value: Mapping[str, Any]) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), allow_nan=False
    ).encode("utf-8")


def _sensitive_key(value: str) -> bool:
    normalized = "".join(character for character in value.lower() if character.isalnum())
    return normalized in {
        "authorization",
        "bearer",
        "cookie",
        "credential",
        "password",
        "secret",
        "setcookie",
        "token",
        "accesstoken",
        "refreshtoken",
        "clientsecret",
        "apikey",
    } or normalized.endswith(("password", "secret", "token"))
