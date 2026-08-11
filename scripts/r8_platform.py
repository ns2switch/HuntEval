#!/usr/bin/env python3
"""Validate R8 native targets and their candidate evidence fail closed."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import platform

MATRIX = pathlib.Path("examples/contracts/v1.0/platform-target-matrix.json")
EXPECTED = {
    "x86_64-unknown-linux-gnu": ("linux", "x86_64", "tar.gz", ""),
    "x86_64-apple-darwin": ("macos", "x86_64", "tar.gz", ""),
    "aarch64-apple-darwin": ("macos", "aarch64", "tar.gz", ""),
    "x86_64-pc-windows-msvc": ("windows", "x86_64", "zip", ".exe"),
}


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def load_matrix() -> dict:
    try:
        value = json.loads(MATRIX.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"platform matrix is unreadable: {type(error).__name__}")
    if not isinstance(value, dict) or set(value) != {"schema_version", "targets"}:
        fail("platform matrix has an unexpected shape")
    if value["schema_version"] != "1.0" or not isinstance(value["targets"], list):
        fail("platform matrix version or targets are invalid")
    seen: set[str] = set()
    for item in value["targets"]:
        if not isinstance(item, dict) or item.get("target") not in EXPECTED:
            fail("platform matrix contains an unknown target")
        target = item["target"]
        if target in seen:
            fail("platform matrix contains a duplicate target")
        seen.add(target)
        expected = EXPECTED[target]
        actual = (item.get("os"), item.get("architecture"), item.get("archive_format"), item.get("binary_suffix"))
        if actual != expected or item.get("native_gate_required") is not True:
            fail(f"platform target has inconsistent native properties: {target}")
        scored = item.get("scored_execution")
        sandbox = item.get("sandbox_backend")
        support = item.get("support_level")
        if target == "x86_64-unknown-linux-gnu":
            if scored is not True or sandbox != "linux_bubblewrap" or support != "candidate":
                fail("Linux candidate properties are inconsistent")
        elif scored is not False or sandbox != "unavailable" or support != "preview":
            fail("preview targets cannot claim scored execution or a sandbox")
    if seen != set(EXPECTED):
        fail("platform matrix is incomplete")
    return value


def target_entry(target: str) -> dict:
    matrix = load_matrix()
    return next(item for item in matrix["targets"] if item["target"] == target)


def host_identity() -> tuple[str, str]:
    systems = {"Linux": "linux", "Darwin": "macos", "Windows": "windows"}
    machines = {"x86_64": "x86_64", "AMD64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}
    system = systems.get(platform.system())
    machine = machines.get(platform.machine())
    if system is None or machine is None:
        fail("host operating system or architecture is unsupported")
    return system, machine


def assert_host(target: str) -> dict:
    entry = target_entry(target)
    if (entry["os"], entry["architecture"]) != host_identity():
        fail(f"target {target} is not native to this host")
    return entry


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_candidates(root: pathlib.Path) -> None:
    load_matrix()
    if root.is_symlink() or not root.is_dir():
        fail("candidate root must be a regular directory")
    found = {path.name for path in root.iterdir() if path.is_dir() and not path.is_symlink()}
    if found != set(EXPECTED):
        fail("candidate root does not contain the exact native target set")
    for target in sorted(EXPECTED):
        candidate = root / target
        evidence_path = candidate / "native-platform-evidence.json"
        try:
            evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            fail(f"native evidence is unreadable for {target}: {type(error).__name__}")
        expected_keys = {
            "schema_version",
            "target",
            "runner",
            "matrix_sha256",
            "archive",
            "archive_sha256",
            "native_smoke_passed",
            "production_published",
        }
        if not isinstance(evidence, dict) or set(evidence) != expected_keys:
            fail(f"native evidence has an unexpected shape for {target}")
        if evidence.get("schema_version") != "1.0" or evidence.get("target") != target:
            fail(f"native evidence identity mismatch for {target}")
        if evidence.get("runner") != target_entry(target)["runner"]:
            fail(f"native evidence runner mismatch for {target}")
        if evidence.get("matrix_sha256") != digest(MATRIX):
            fail(f"native evidence matrix mismatch for {target}")
        if evidence.get("native_smoke_passed") is not True or evidence.get("production_published") is not False:
            fail(f"native evidence makes an invalid state claim for {target}")
        archive = evidence.get("archive")
        if not isinstance(archive, str) or pathlib.PurePosixPath(archive).name != archive:
            fail(f"unsafe native archive name for {target}")
        archive_path = candidate / archive
        if not archive_path.is_file() or digest(archive_path) != evidence.get("archive_sha256"):
            fail(f"native archive digest mismatch for {target}")
        try:
            manifest = json.loads((candidate / "release-manifest.json").read_text(encoding="utf-8"))
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            fail(f"candidate manifest is unreadable for {target}: {type(error).__name__}")
        if (
            not isinstance(manifest, dict)
            or set(manifest)
            != {
                "schema_version",
                "release_status",
                "production_published",
                "revision",
                "target",
                "artifacts",
            }
            or manifest.get("schema_version") != "1.0"
            or manifest.get("target") != target
            or manifest.get("release_status") != "candidate"
            or manifest.get("production_published") is not False
            or not isinstance(manifest.get("artifacts"), list)
            or not manifest["artifacts"]
            or len(manifest["artifacts"]) > 64
        ):
            fail(f"candidate manifest identity is invalid for {target}")
        revision = manifest.get("revision")
        if (
            not isinstance(revision, str)
            or len(revision) != 40
            or any(character not in "0123456789abcdef" for character in revision)
        ):
            fail(f"candidate manifest revision is invalid for {target}")
        references: dict[str, str] = {}
        for artifact in manifest["artifacts"]:
            if not isinstance(artifact, dict) or set(artifact) != {"path", "sha256"}:
                fail(f"candidate manifest contains a malformed reference for {target}")
            name = artifact["path"]
            relative = pathlib.PurePosixPath(name) if isinstance(name, str) else None
            if (
                relative is None
                or relative.is_absolute()
                or ".." in relative.parts
                or name in references
            ):
                fail(f"candidate manifest contains an unsafe reference for {target}")
            path = candidate.joinpath(*relative.parts)
            if not path.is_file() or digest(path) != artifact["sha256"]:
                fail(f"candidate manifest reference failed for {target}: {name}")
            references[name] = artifact["sha256"]
        if archive not in references or "native-platform-evidence.json" not in references:
            fail(f"candidate manifest omits native roots for {target}")


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("validate")
    host = commands.add_parser("assert-host")
    host.add_argument("--target", required=True)
    verify = commands.add_parser("verify-candidates")
    verify.add_argument("--root", required=True)
    args = parser.parse_args()
    if args.command == "validate":
        load_matrix()
    elif args.command == "assert-host":
        assert_host(args.target)
    else:
        verify_candidates(pathlib.Path(args.root))


if __name__ == "__main__":
    main()
