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
                        "source": "registry+https://github.com/rust-lang/crates.io-index",
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
        network_used=False,
    )
    MODULE.build(arguments)
    MODULE.verify_directory(output)
    provenance = json.loads((output / "build-provenance.json").read_text(encoding="utf-8"))
    if provenance["network_used"] is not False:
        raise SystemExit("isolated supply-chain fixture did not preserve network provenance")
    sbom = json.loads((output / "sbom.spdx.json").read_text(encoding="utf-8"))
    if sbom["packages"][0]["externalRefs"] != [
        {
            "referenceCategory": "PACKAGE-MANAGER",
            "referenceLocator": "pkg:cargo/hunteval-test@1.0.0",
            "referenceType": "purl",
        }
    ]:
        raise SystemExit("SBOM package is not discoverable through its Cargo purl")
    malformed_sbom = json.loads(json.dumps(sbom))
    malformed_sbom["packages"][0]["externalRefs"][0]["referenceLocator"] = (
        "pkg:cargo/substituted@9.9.9"
    )
    expect_failure("substituted SBOM purl", lambda: MODULE.verify_sbom(malformed_sbom))

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
