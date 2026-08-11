#!/usr/bin/env python3
"""Build and verify bounded deterministic R8 supply-chain evidence."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import stat
import sys

MAX_FILES = 4096
MAX_FILE_BYTES = 512 * 1024 * 1024
MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
FORBIDDEN_PARTS = {".git", ".env", "private", "target"}


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def read_json(path: pathlib.Path) -> object:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_FILE_BYTES:
        fail(f"unsafe or oversized JSON file: {path}")
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"invalid JSON file {path}: {type(error).__name__}")


def write_json(path: pathlib.Path, value: object) -> None:
    path.write_text(
        json.dumps(value, indent=2, sort_keys=True, ensure_ascii=True) + "\n",
        encoding="utf-8",
    )


def safe_relative(path: pathlib.PurePosixPath) -> bool:
    return bool(path.parts) and all(
        part not in {"", ".", ".."}
        and part.lower() not in FORBIDDEN_PARTS
        and not part.lower().startswith(".env")
        and "ground-truth" not in part.lower()
        and "ground_truth" not in part.lower()
        for part in path.parts
    )


def package_inventory(root: pathlib.Path, target: str) -> dict:
    if root.is_symlink() or not root.is_dir():
        fail("package root must be a regular directory")
    entries: list[dict] = []
    total = 0
    for path in sorted(root.rglob("*")):
        relative = pathlib.PurePosixPath(path.relative_to(root).as_posix())
        if not safe_relative(relative):
            fail(f"unsafe or private package path: {relative}")
        metadata = path.lstat()
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            if path.is_dir() and not path.is_symlink():
                continue
            fail(f"unsupported package member: {relative}")
        if metadata.st_nlink != 1:
            fail(f"hard-linked package member: {relative}")
        if metadata.st_size > MAX_FILE_BYTES:
            fail(f"oversized package member: {relative}")
        total += metadata.st_size
        if total > MAX_TOTAL_BYTES or len(entries) >= MAX_FILES:
            fail("package inventory exceeds its bounds")
        entries.append(
            {
                "mode": format(stat.S_IMODE(metadata.st_mode), "04o"),
                "path": str(relative),
                "sha256": digest_bytes(path.read_bytes()),
                "size": metadata.st_size,
            }
        )
    if not entries:
        fail("package root is empty")
    return {"schema_version": "1.0", "target": target, "files": entries}


def dependency_evidence(metadata: dict, revision: str, epoch: int) -> tuple[dict, dict, dict]:
    packages = metadata.get("packages")
    if not isinstance(packages, list) or not packages:
        fail("Cargo metadata contains no packages")
    dependencies = []
    licenses = []
    for package in sorted(packages, key=lambda item: (item.get("name", ""), item.get("version", ""))):
        name = package.get("name")
        version = package.get("version")
        license_expression = package.get("license")
        if not all(isinstance(value, str) and value for value in (name, version, license_expression)):
            fail("dependency has missing identity or license")
        source = package.get("source") or "workspace"
        dependencies.append({"name": name, "source": source, "version": version})
        licenses.append({"license": license_expression, "name": name, "version": version})
    created = datetime.datetime.fromtimestamp(epoch, datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
    namespace = f"https://hunteval.dev/spdx/{revision}"
    sbom = {
        "SPDXID": "SPDXRef-DOCUMENT",
        "creationInfo": {"created": created, "creators": ["Tool: hunteval-r8-supply-chain"]},
        "dataLicense": "CC0-1.0",
        "documentNamespace": namespace,
        "name": "HuntEval R8 candidate dependencies",
        "packages": [
            {
                "SPDXID": f"SPDXRef-Package-{index}",
                "downloadLocation": "NOASSERTION",
                "licenseConcluded": item["license"],
                "licenseDeclared": item["license"],
                "name": item["name"],
                "versionInfo": item["version"],
            }
            for index, item in enumerate(licenses, start=1)
        ],
        "spdxVersion": "SPDX-2.3",
    }
    return (
        {"schema_version": "1.0", "dependencies": dependencies},
        {"schema_version": "1.0", "licenses": licenses},
        sbom,
    )


def build(args: argparse.Namespace) -> None:
    package_root = pathlib.Path(args.package_root).resolve()
    output = pathlib.Path(args.output)
    if not output.is_absolute() or output == pathlib.Path("/") or output.exists():
        fail("output must be a new absolute directory")
    if len(args.revision) != 40 or any(character not in "0123456789abcdef" for character in args.revision):
        fail("revision must be a lowercase 40-character hexadecimal identity")
    metadata = read_json(pathlib.Path(args.metadata))
    if not isinstance(metadata, dict):
        fail("Cargo metadata has an unexpected shape")
    output.mkdir(mode=0o700, parents=False)
    inventory = package_inventory(package_root, args.target)
    dependencies, licenses, sbom = dependency_evidence(metadata, args.revision, args.epoch)
    materials = {
        "cargo_lock_sha256": digest_bytes(pathlib.Path("Cargo.lock").read_bytes()),
        "compatibility_matrix_sha256": digest_bytes(
            pathlib.Path("examples/contracts/v1.0/compatibility-matrix.json").read_bytes()
        ),
        "interface_inventory_sha256": digest_bytes(
            pathlib.Path("examples/contracts/v1.0/release-interface-inventory.json").read_bytes()
        ),
        "platform_target_matrix_sha256": digest_bytes(
            pathlib.Path("examples/contracts/v1.0/platform-target-matrix.json").read_bytes()
        ),
    }
    provenance = {
        "schema_version": "1.0",
        "builder": "scripts/r8_supply_chain.py",
        "revision": args.revision,
        "rust_toolchain": args.rust_toolchain,
        "source_date_epoch": args.epoch,
        "target": args.target,
        "materials": materials,
        "network_used": False,
    }
    artifacts = {
        "package-inventory.json": inventory,
        "dependency-report.json": dependencies,
        "license-report.json": licenses,
        "sbom.spdx.json": sbom,
        "build-provenance.json": provenance,
    }
    for name, value in artifacts.items():
        write_json(output / name, value)
    references = []
    for name in sorted(artifacts):
        path = output / name
        references.append({"path": name, "sha256": digest_bytes(path.read_bytes())})
    manifest = {
        "schema_version": "1.0",
        "release_status": "candidate",
        "production_published": False,
        "revision": args.revision,
        "target": args.target,
        "artifacts": references,
    }
    write_json(output / "release-manifest.json", manifest)
    verify_directory(output)


def verify_directory(root: pathlib.Path) -> None:
    if root.is_symlink() or not root.is_dir():
        fail("evidence root must be a regular directory")
    manifest = read_json(root / "release-manifest.json")
    if not isinstance(manifest, dict) or manifest.get("schema_version") != "1.0":
        fail("unsupported release manifest")
    if manifest.get("release_status") != "candidate" or manifest.get("production_published") is not False:
        fail("release manifest makes an unauthorized publication claim")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts or len(artifacts) > 64:
        fail("release manifest artifact inventory is invalid")
    seen: set[str] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict) or set(artifact) != {"path", "sha256"}:
            fail("malformed release manifest artifact")
        name = artifact.get("path")
        expected = artifact.get("sha256")
        if not isinstance(name, str) or not safe_relative(pathlib.PurePosixPath(name)) or name in seen:
            fail("duplicate or unsafe release artifact")
        if not isinstance(expected, str) or len(expected) != 64:
            fail("invalid release artifact digest")
        seen.add(name)
        path = root / name
        if path.is_symlink() or not path.is_file() or digest_bytes(path.read_bytes()) != expected:
            fail(f"release artifact verification failed: {name}")
    inventory = read_json(root / "package-inventory.json")
    if not isinstance(inventory, dict) or inventory.get("schema_version") != "1.0":
        fail("invalid package inventory")


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    build_parser = commands.add_parser("build")
    build_parser.add_argument("--package-root", required=True)
    build_parser.add_argument("--output", required=True)
    build_parser.add_argument("--metadata", required=True)
    build_parser.add_argument("--revision", required=True)
    build_parser.add_argument("--target", required=True)
    build_parser.add_argument("--rust-toolchain", required=True)
    build_parser.add_argument("--epoch", required=True, type=int)
    verify_parser = commands.add_parser("verify")
    verify_parser.add_argument("--root", required=True)
    args = parser.parse_args()
    if args.command == "build":
        build(args)
    else:
        verify_directory(pathlib.Path(args.root))


if __name__ == "__main__":
    main()
