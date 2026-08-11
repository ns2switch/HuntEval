#!/usr/bin/env python3
"""Safely install and verify an immutable HuntEval R8 tar candidate."""

from __future__ import annotations

import argparse
import os
import pathlib
import shutil
import tarfile
import tempfile
import unicodedata
import zipfile

MAX_MEMBERS = 4096
MAX_MEMBER_BYTES = 512 * 1024 * 1024
MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
WINDOWS_RESERVED = {
    "CON",
    "PRN",
    "AUX",
    "NUL",
    *(f"COM{number}" for number in range(1, 10)),
    *(f"LPT{number}" for number in range(1, 10)),
}


def unsafe_windows_part(part: str) -> bool:
    stem = part.rstrip(" .").split(".", maxsplit=1)[0].upper()
    return (
        part.endswith((" ", "."))
        or any(character in '<>"|?*' for character in part)
        or stem in WINDOWS_RESERVED
    )


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def relative_member(name: str) -> pathlib.PurePosixPath:
    path = pathlib.PurePosixPath(name)
    if (
        "\\" in name
        or ":" in name
        or any(unicodedata.category(character).startswith("C") for character in name)
        or path.is_absolute()
        or len(path.parts) < 2
        or path.parts[0] != "hunteval"
        or any(part in {"", ".", ".."} for part in path.parts)
        or any(unsafe_windows_part(part) for part in path.parts[1:])
    ):
        fail("archive contains an unsafe path")
    return pathlib.PurePosixPath(*path.parts[1:])


def install(archive: pathlib.Path, destination: pathlib.Path) -> None:
    if archive.is_symlink() or not archive.is_file():
        fail("archive must be a regular file")
    if not destination.is_absolute() or destination == pathlib.Path("/") or destination.exists():
        fail("destination must be a new absolute path")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = pathlib.Path(tempfile.mkdtemp(prefix=".hunteval-install-", dir=destination.parent))
    total = 0
    seen: set[pathlib.PurePosixPath] = set()
    try:
        if archive.name.endswith(".zip"):
            install_zip(archive, temporary, seen)
        elif archive.name.endswith(".tar.gz"):
            install_tar(archive, temporary, seen)
        else:
            fail("archive format is unsupported")
        verify(temporary, None)
        temporary.replace(destination)
    except BaseException:
        shutil.rmtree(temporary, ignore_errors=True)
        raise


def install_tar(
    archive: pathlib.Path,
    temporary: pathlib.Path,
    seen: set[pathlib.PurePosixPath],
) -> None:
    total = 0
    with tarfile.open(archive, mode="r:gz") as package:
            members = package.getmembers()
            if not members or len(members) > MAX_MEMBERS:
                fail("archive member inventory is empty or oversized")
            for member in members:
                if member.isdir() and member.name.rstrip("/") == "hunteval":
                    continue
                relative = relative_member(member.name)
                if relative in seen:
                    fail("archive contains duplicate paths")
                seen.add(relative)
                if member.issym() or member.islnk() or member.isdev() or member.isfifo():
                    fail("archive contains a special or linked member")
                if not (member.isdir() or member.isfile()):
                    fail("archive contains an unsupported member")
                if member.size < 0 or member.size > MAX_MEMBER_BYTES:
                    fail("archive member exceeds the byte limit")
                total += member.size
                if total > MAX_TOTAL_BYTES:
                    fail("archive exceeds the expanded byte limit")
                target = temporary.joinpath(*relative.parts)
                if member.isdir():
                    target.mkdir(mode=0o755, parents=True, exist_ok=True)
                    continue
                target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
                source = package.extractfile(member)
                if source is None:
                    fail("archive regular member cannot be read")
                flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
                if hasattr(os, "O_NOFOLLOW"):
                    flags |= os.O_NOFOLLOW
                descriptor = os.open(target, flags, 0o600)
                with source, os.fdopen(descriptor, "wb") as output:
                    shutil.copyfileobj(source, output, length=1024 * 1024)
                target.chmod(0o755 if relative.parts[0] == "bin" else 0o644)


def install_zip(
    archive: pathlib.Path,
    temporary: pathlib.Path,
    seen: set[pathlib.PurePosixPath],
) -> None:
    total = 0
    with zipfile.ZipFile(archive, mode="r") as package:
        members = package.infolist()
        if not members or len(members) > MAX_MEMBERS:
            fail("archive member inventory is empty or oversized")
        for member in members:
            if member.is_dir() and member.filename.rstrip("/") == "hunteval":
                continue
            relative = relative_member(member.filename)
            if relative in seen:
                fail("archive contains duplicate paths")
            seen.add(relative)
            unix_type = (member.external_attr >> 16) & 0o170000
            if unix_type == 0o120000 or member.flag_bits & 0x1:
                fail("archive contains a linked or encrypted member")
            if member.file_size < 0 or member.file_size > MAX_MEMBER_BYTES:
                fail("archive member exceeds the byte limit")
            total += member.file_size
            if total > MAX_TOTAL_BYTES:
                fail("archive exceeds the expanded byte limit")
            target = temporary.joinpath(*relative.parts)
            if member.is_dir():
                target.mkdir(mode=0o755, parents=True, exist_ok=True)
                continue
            target.parent.mkdir(mode=0o755, parents=True, exist_ok=True)
            flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
            if hasattr(os, "O_NOFOLLOW"):
                flags |= os.O_NOFOLLOW
            descriptor = os.open(target, flags, 0o600)
            with package.open(member, "r") as source, os.fdopen(descriptor, "wb") as output:
                shutil.copyfileobj(source, output, length=1024 * 1024)
            target.chmod(0o755 if relative.parts[0] == "bin" else 0o644)


def verify(root: pathlib.Path, target: str | None) -> None:
    windows_package = target == "x86_64-pc-windows-msvc" or (
        target is None and (root / "bin/hunteval.exe").is_file()
    )
    suffix = ".exe" if windows_package else ""
    required = [
        f"bin/hunteval{suffix}",
        "LICENSE",
        "README.md",
        "SECURITY.md",
        "schemas/v1.0/release-interface-inventory.schema.json",
    ]
    if root.is_symlink() or not root.is_dir():
        fail("installation root is not a regular directory")
    for name in required:
        path = root / name
        if path.is_symlink() or not path.is_file():
            fail(f"installation is missing required member: {name}")
    if not suffix and not os.access(root / "bin/hunteval", os.X_OK):
        fail("installed CLI is not executable")


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    installer = commands.add_parser("install")
    installer.add_argument("--archive", required=True)
    installer.add_argument("--destination", required=True)
    verifier = commands.add_parser("verify")
    verifier.add_argument("--root", required=True)
    verifier.add_argument("--target")
    args = parser.parse_args()
    if args.command == "install":
        install(pathlib.Path(args.archive), pathlib.Path(args.destination))
    else:
        verify(pathlib.Path(args.root), args.target)


if __name__ == "__main__":
    main()
