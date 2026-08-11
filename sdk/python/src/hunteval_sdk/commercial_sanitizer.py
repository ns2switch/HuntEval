"""Fail-closed conversion of reviewed tenant recordings into synthetic fixtures."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from typing import Any, Mapping

from ._validation import identifier
from .commercial import (
    MAX_RECORDS,
    CommercialConnectorError,
    CommercialFixture,
    CommercialRequest,
    build_fixture,
)

MAX_FIELDS = 512
MAX_DEPTH = 16
SAFE_RETAINED_LITERALS = frozenset(
    {
        "observed",
        "informational",
        "low",
        "medium",
        "high",
        "critical",
        "unknown",
    }
)


@dataclass(frozen=True, slots=True)
class RecordingSanitizationPolicy:
    """Explicit schema vocabulary and safe literals for one recording family."""

    policy_id: str
    allowed_fields: frozenset[str]
    retained_literals: frozenset[str] = frozenset()

    def __post_init__(self) -> None:
        identifier(self.policy_id)
        if not self.allowed_fields or len(self.allowed_fields) > MAX_FIELDS:
            raise CommercialConnectorError("sanitization field inventory is invalid")
        if any(not _safe_field(field) for field in self.allowed_fields):
            raise CommercialConnectorError("sanitization field inventory is unsafe")
        if len(self.retained_literals) > MAX_FIELDS or any(
            not isinstance(value, str) or value not in SAFE_RETAINED_LITERALS
            for value in self.retained_literals
        ):
            raise CommercialConnectorError("sanitization literal inventory is invalid")

    @property
    def sha256(self) -> str:
        value = {
            "policy_id": self.policy_id,
            "allowed_fields": sorted(self.allowed_fields),
            "retained_literals": sorted(self.retained_literals),
        }
        return hashlib.sha256(_canonical(value)).hexdigest()


@dataclass(frozen=True, slots=True)
class SanitizedRecording:
    """A synthetic fixture plus the policy identity used to derive it."""

    fixture: CommercialFixture
    policy_sha256: str


def sanitize_recording(
    fixture_id: str,
    request: CommercialRequest,
    recording: Mapping[str, Any],
    policy: RecordingSanitizationPolicy,
) -> SanitizedRecording:
    """Sanitize every data value and reject every undeclared field."""
    if not isinstance(recording, Mapping) or set(recording) != {
        "records",
        "truncated",
        "more_available",
    }:
        raise CommercialConnectorError("recording envelope is unsupported")
    records = recording["records"]
    if not isinstance(records, list) or len(records) > MAX_RECORDS:
        raise CommercialConnectorError("recording record count is invalid")
    if not isinstance(recording["truncated"], bool) or not isinstance(
        recording["more_available"], bool
    ):
        raise CommercialConnectorError("recording pagination flags are invalid")

    sanitized = {
        "records": [
            _sanitize(record, policy, f"/records/{index}", 0)
            for index, record in enumerate(records)
        ],
        "truncated": recording["truncated"],
        "more_available": recording["more_available"],
    }
    fixture = build_fixture(fixture_id, request, sanitized)
    return SanitizedRecording(fixture, policy.sha256)


def _sanitize(
    value: Any, policy: RecordingSanitizationPolicy, path: str, depth: int
) -> Any:
    if depth > MAX_DEPTH:
        raise CommercialConnectorError("recording exceeds sanitization nesting limits")
    if isinstance(value, Mapping):
        result: dict[str, Any] = {}
        if len(value) > MAX_FIELDS:
            raise CommercialConnectorError("recording object is oversized")
        for key, nested in value.items():
            if not isinstance(key, str) or key not in policy.allowed_fields:
                raise CommercialConnectorError("recording contains an undeclared field")
            result[key] = _sanitize(nested, policy, f"{path}/{key}", depth + 1)
        return result
    if isinstance(value, list):
        if len(value) > MAX_RECORDS:
            raise CommercialConnectorError("recording array is oversized")
        return [
            _sanitize(item, policy, f"{path}/{index}", depth + 1)
            for index, item in enumerate(value)
        ]
    if value is None or isinstance(value, bool):
        return value
    if isinstance(value, str):
        if value in policy.retained_literals:
            return value
        return f"synthetic-{_replacement_digest(policy, path)[:16]}"
    if isinstance(value, int):
        return int(_replacement_digest(policy, path)[:15], 16)
    if isinstance(value, float):
        if value != value or value in {float("inf"), float("-inf")}:
            raise CommercialConnectorError("recording contains a non-finite number")
        return int(_replacement_digest(policy, path)[:15], 16)
    raise CommercialConnectorError("recording contains a non-JSON value")


def _replacement_digest(policy: RecordingSanitizationPolicy, path: str) -> str:
    material = {
        "policy_sha256": policy.sha256,
        "path": path,
    }
    return hashlib.sha256(_canonical(material)).hexdigest()


def _safe_field(value: str) -> bool:
    if not value or len(value.encode("utf-8")) > 128:
        return False
    normalized = "".join(character for character in value.lower() if character.isalnum())
    return normalized not in {
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
    } and not normalized.endswith(("password", "secret", "token"))


def _canonical(value: Mapping[str, Any]) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
