from __future__ import annotations

import hashlib
import json
import os
import stat
from pathlib import Path
from dataclasses import dataclass
from typing import Any, Literal

from ._validation import ContractError, digest

MAX_ARTIFACT_BYTES = 16 * 1024 * 1024
PublicArtifactKind = Literal["manifest", "protocol_message", "run", "report"]


@dataclass(frozen=True, slots=True)
class VerifiedPublicArtifact:
    kind: PublicArtifactKind
    schema_version: str
    sha256: str
    value: dict[str, Any]


def read_verified_json(root: Path, relative_path: str, expected_sha256: str) -> dict[str, Any]:
    """Read one bounded, root-confined public JSON artifact by exact digest."""
    expected = digest(expected_sha256)
    if Path(relative_path).is_absolute() or ".." in Path(relative_path).parts:
        raise ContractError("artifact path is unsafe")
    safe_root = root.resolve(strict=True)
    candidate = safe_root.joinpath(relative_path)
    current = safe_root
    for component in Path(relative_path).parts:
        current = current / component
        if current.is_symlink():
            raise ContractError("artifact symlinks are not supported")
    resolved = candidate.resolve(strict=True)
    if not resolved.is_relative_to(safe_root) or not resolved.is_file():
        raise ContractError("artifact is outside the authorized root")
    before = resolved.stat(follow_symlinks=False)
    if not stat.S_ISREG(before.st_mode) or before.st_size > MAX_ARTIFACT_BYTES:
        raise ContractError("artifact exceeds the byte limit")
    flags = os.O_RDONLY | getattr(os, "O_CLOEXEC", 0) | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(resolved, flags)
    try:
        opened = os.fstat(descriptor)
        if (opened.st_dev, opened.st_ino, opened.st_size) != (
            before.st_dev, before.st_ino, before.st_size
        ):
            raise ContractError("artifact changed while it was opened")
        chunks: list[bytes] = []
        remaining = MAX_ARTIFACT_BYTES + 1
        while remaining > 0:
            chunk = os.read(descriptor, min(65_536, remaining))
            if not chunk:
                break
            chunks.append(chunk)
            remaining -= len(chunk)
        raw = b"".join(chunks)
        if len(raw) > MAX_ARTIFACT_BYTES:
            raise ContractError("artifact exceeds the byte limit")
    finally:
        os.close(descriptor)
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ContractError("artifact digest does not match")
    value = json.loads(raw, object_pairs_hook=_unique_object)
    if not isinstance(value, dict):
        raise ContractError("artifact root must be an object")
    return value


def read_public_artifact(
    root: Path,
    relative_path: str,
    expected_sha256: str,
    kind: PublicArtifactKind,
) -> VerifiedPublicArtifact:
    value = read_verified_json(root, relative_path, expected_sha256)
    version = value.get("schema_version", value.get("protocol_version"))
    if not isinstance(version, str) or not version:
        raise ContractError("public artifact has no supported contract version")
    if _contains_private_field(value):
        raise ContractError("public artifact contains a prohibited private field")
    return VerifiedPublicArtifact(kind, version, expected_sha256, value)


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ContractError("artifact contains a duplicate JSON field")
        value[key] = item
    return value


def _contains_private_field(value: Any) -> bool:
    forbidden = {
        "ground_truth", "hidden_test", "hidden_test_results", "reference_query", "evaluator_only"
    }
    if isinstance(value, dict):
        return any(key.lower() in forbidden or _contains_private_field(item) for key, item in value.items())
    if isinstance(value, list):
        return any(_contains_private_field(item) for item in value)
    return False
