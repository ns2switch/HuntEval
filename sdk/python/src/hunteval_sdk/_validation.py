from __future__ import annotations

import re
from typing import Any

IDENTIFIER = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$")
SHA256 = re.compile(r"^[a-f0-9]{64}$")


class ContractError(ValueError):
    """A public contract failed bounded validation."""


def exact_keys(value: dict[str, Any], required: set[str]) -> None:
    if set(value) != required:
        raise ContractError("contract fields do not match the supported version")


def identifier(value: Any) -> str:
    if not isinstance(value, str) or IDENTIFIER.fullmatch(value) is None:
        raise ContractError("identifier is invalid")
    return value


def digest(value: Any) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise ContractError("SHA-256 digest is invalid")
    return value


def positive_int(value: Any) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ContractError("limit must be a positive integer")
    return value
