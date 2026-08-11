#!/usr/bin/env python3
"""Positive and adversarial archive fixtures for the R8 installer."""

import importlib.util
import io
import pathlib
import tarfile
import tempfile
import zipfile


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("r8_install", ROOT / "scripts/r8_install.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("cannot load R8 installer")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def archive(path: pathlib.Path, unsafe: bool = False) -> None:
    members = {
        "hunteval/bin/hunteval": b"#!/bin/sh\nexit 0\n",
        "hunteval/LICENSE": b"Apache-2.0\n",
        "hunteval/README.md": b"HuntEval\n",
        "hunteval/SECURITY.md": b"Security\n",
        "hunteval/schemas/v1.0/release-interface-inventory.schema.json": b"{}\n",
    }
    if unsafe:
        members["hunteval/../../escape"] = b"escape"
    with tarfile.open(path, "w:gz") as output:
        root = tarfile.TarInfo("hunteval/")
        root.type = tarfile.DIRTYPE
        root.mode = 0o777
        output.addfile(root)
        for name, content in members.items():
            info = tarfile.TarInfo(name)
            info.size = len(content)
            info.mode = 0o777
            output.addfile(info, io.BytesIO(content))


with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    candidate = temporary / "candidate.tar.gz"
    archive(candidate)
    installed = temporary / "installed"
    MODULE.install(candidate, installed)
    MODULE.verify(installed, "x86_64-unknown-linux-gnu")
    if (installed / "LICENSE").stat().st_mode & 0o777 != 0o644:
        raise SystemExit("installed data permissions are not normalized")
    if (installed / "bin/hunteval").stat().st_mode & 0o777 != 0o755:
        raise SystemExit("installed executable permissions are not normalized")

with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    hostile = temporary / "hostile.tar.gz"
    archive(hostile, unsafe=True)
    try:
        MODULE.install(hostile, temporary / "installed")
    except SystemExit:
        pass
    else:
        raise SystemExit("archive traversal fixture was accepted")

with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    candidate = temporary / "candidate.zip"
    members = {
        "hunteval/bin/hunteval.exe": b"MZ fixture",
        "hunteval/LICENSE": b"Apache-2.0\n",
        "hunteval/README.md": b"HuntEval\n",
        "hunteval/SECURITY.md": b"Security\n",
        "hunteval/schemas/v1.0/release-interface-inventory.schema.json": b"{}\n",
    }
    with zipfile.ZipFile(candidate, "w") as output:
        for name, content in members.items():
            output.writestr(name, content)
    installed = temporary / "installed"
    MODULE.install(candidate, installed)
    MODULE.verify(installed, "x86_64-pc-windows-msvc")

with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    hostile = temporary / "hostile.zip"
    with zipfile.ZipFile(hostile, "w") as output:
        output.writestr("hunteval/..\\escape", b"escape")
        output.writestr("hunteval/CON", b"device")
    try:
        MODULE.install(hostile, temporary / "installed")
    except SystemExit:
        pass
    else:
        raise SystemExit("Windows-style ZIP traversal fixture was accepted")

with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    hostile = temporary / "device.zip"
    with zipfile.ZipFile(hostile, "w") as output:
        output.writestr("hunteval/CON", b"device")
    try:
        MODULE.install(hostile, temporary / "installed")
    except SystemExit:
        pass
    else:
        raise SystemExit("Windows reserved-device fixture was accepted")

print("R8 installer fixtures pass")
