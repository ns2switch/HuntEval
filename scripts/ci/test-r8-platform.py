#!/usr/bin/env python3
"""Positive and adversarial fixtures for the R8 native target matrix."""

import hashlib
import importlib.util
import json
import pathlib
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("r8_platform", ROOT / "scripts/r8_platform.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("cannot load R8 platform validator")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

matrix = MODULE.load_matrix()
if len(matrix["targets"]) != 4:
    raise SystemExit("native target matrix is incomplete")

with tempfile.TemporaryDirectory() as directory:
    root = pathlib.Path(directory)
    for target, (_, _, archive_format, _) in MODULE.EXPECTED.items():
        candidate = root / target
        candidate.mkdir()
        extension = ".tar.gz" if archive_format == "tar.gz" else ".zip"
        archive_name = f"hunteval-rc-fixture-{target}{extension}"
        archive = candidate / archive_name
        archive.write_bytes(target.encode("ascii"))
        evidence = {
            "schema_version": "1.0",
            "target": target,
            "runner": MODULE.target_entry(target)["runner"],
            "matrix_sha256": MODULE.digest(MODULE.MATRIX),
            "archive": archive_name,
            "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
            "native_smoke_passed": True,
            "production_published": False,
        }
        evidence_path = candidate / "native-platform-evidence.json"
        evidence_path.write_text(
            json.dumps(evidence), encoding="utf-8"
        )
        manifest = {
            "schema_version": "1.0",
            "release_status": "candidate",
            "production_published": False,
            "revision": "1" * 40,
            "target": target,
            "artifacts": [
                {"path": archive_name, "sha256": MODULE.digest(archive)},
                {"path": evidence_path.name, "sha256": MODULE.digest(evidence_path)},
            ],
        }
        (candidate / "release-manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )
    MODULE.verify_candidates(root)
    changed = root / "x86_64-pc-windows-msvc" / "hunteval-rc-fixture-x86_64-pc-windows-msvc.zip"
    changed.write_bytes(b"changed")
    try:
        MODULE.verify_candidates(root)
    except SystemExit:
        pass
    else:
        raise SystemExit("changed native archive was accepted")

print("R8 native platform fixtures pass")
