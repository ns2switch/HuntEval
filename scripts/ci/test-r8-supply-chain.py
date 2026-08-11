#!/usr/bin/env python3
"""Negative and tamper tests for the R8 supply-chain evidence builder."""

import argparse
import importlib.util
import json
import pathlib
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("r8_supply_chain", ROOT / "scripts/r8_supply_chain.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("cannot load R8 supply-chain module")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def expect_failure(label: str, operation) -> None:
    try:
        operation()
    except SystemExit:
        return
    raise SystemExit(f"unsafe R8 supply-chain fixture was accepted: {label}")


with tempfile.TemporaryDirectory() as directory:
    temporary = pathlib.Path(directory)
    package = temporary / "package"
    package.mkdir()
    (package / "hunteval").write_bytes(b"bounded package")
    metadata = temporary / "metadata.json"
    metadata.write_text(
        json.dumps(
            {
                "packages": [
                    {
                        "name": "hunteval-test",
                        "version": "1.0.0",
                        "license": "Apache-2.0",
                        "source": None,
                    }
                ]
            }
        ),
        encoding="utf-8",
    )
    output = temporary / "evidence"
    arguments = argparse.Namespace(
        package_root=str(package),
        output=str(output),
        metadata=str(metadata),
        revision="1" * 40,
        target="x86_64-unknown-linux-gnu",
        rust_toolchain="1.93.1",
        epoch=0,
    )
    MODULE.build(arguments)
    MODULE.verify_directory(output)

    inventory = output / "package-inventory.json"
    original = inventory.read_bytes()
    inventory.write_bytes(original + b" ")
    expect_failure("tampered inventory", lambda: MODULE.verify_directory(output))
    inventory.write_bytes(original)

    manifest = output / "release-manifest.json"
    value = json.loads(manifest.read_text(encoding="utf-8"))
    value["production_published"] = True
    manifest.write_text(json.dumps(value), encoding="utf-8")
    expect_failure("publication claim", lambda: MODULE.verify_directory(output))

with tempfile.TemporaryDirectory() as directory:
    root = pathlib.Path(directory)
    (root / "ground-truth.json").write_text("{}", encoding="utf-8")
    expect_failure("ground truth path", lambda: MODULE.package_inventory(root, "test"))

print("R8 supply-chain negative fixtures pass")
