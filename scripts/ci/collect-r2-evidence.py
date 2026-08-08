#!/usr/bin/env python3
"""Create deterministic, path-free R2 evidence from a completed benchmark."""

import hashlib
import json
import pathlib
import subprocess
import sys


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def binary_digests(report: dict, name: str) -> list[str]:
    values = {
        artifact["sha256"]
        for cell in report["cells"]
        for artifact in cell["artifacts"]
        if artifact["artifact"] == name
    }
    if len(values) != 1:
        raise SystemExit(f"error: expected one {name} digest, found {len(values)}")
    return sorted(values)


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit("error: expected benchmark directory, manifest, and output paths")
    benchmark = pathlib.Path(sys.argv[1])
    manifest = pathlib.Path(sys.argv[2])
    output = pathlib.Path(sys.argv[3])
    definition = json.loads((benchmark / "benchmark-definition.json").read_text(encoding="utf-8"))
    report_path = benchmark / "benchmark-report.json"
    report = json.loads(report_path.read_text(encoding="utf-8"))
    evidence = {
        "schema_version": "0.1",
        "revision": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], text=True
        ).strip(),
        "benchmark_id": definition["id"],
        "benchmark_manifest_sha256": digest(manifest),
        "benchmark_definition_sha256": report["benchmark_definition_sha256"],
        "episode_package_sha256": {
            episode["id"]: episode["package_sha256"] for episode in definition["episodes"]
        },
        "deployment_configuration_sha256": {
            deployment["id"]: deployment["configuration_sha256"]
            for deployment in definition["deployments"]
        },
        "scoring_profile_sha256": report["scoring_profile_sha256"],
        "runner_binary_sha256": binary_digests(report, "runner_binary")[0],
        "managed_tool_binary_sha256": binary_digests(report, "managed_tool_binary")[0],
        "normalized_result_sha256": digest(report_path),
        "known_limitations": report["limitations"],
        "adr_status_changes": [],
    }
    output.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
