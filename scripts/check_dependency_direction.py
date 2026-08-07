#!/usr/bin/env python3
"""Reject forbidden dependencies between first-party HuntEval crates."""

from __future__ import annotations

import json
import pathlib
import sys


ALLOWED_LOCAL_DEPENDENCIES: dict[str, set[str]] = {
    "hunteval-domain": set(),
    "hunteval-protocol": {"hunteval-domain"},
    "hunteval-reference-deployment": {"hunteval-domain", "hunteval-protocol"},
    "hunteval-duckdb": {"hunteval-domain"},
    "hunteval-evaluation": {"hunteval-domain"},
    "hunteval-statistics": {"hunteval-domain"},
    "hunteval-resilience": {"hunteval-domain"},
    "hunteval-knowledge": {"hunteval-domain"},
    "hunteval-reporting": {"hunteval-domain", "hunteval-statistics"},
    "hunteval-runner": {
        "hunteval-domain",
        "hunteval-duckdb",
        "hunteval-evaluation",
        "hunteval-knowledge",
        "hunteval-protocol",
        "hunteval-reporting",
        "hunteval-resilience",
        "hunteval-statistics",
    },
    "hunteval-cli": {"hunteval-runner"},
    "hunteval-fixture-tool": {"hunteval-domain"},
}

REFERENCE_DEPENDENCIES = {
    "clap",
    "hunteval-domain",
    "hunteval-protocol",
    "serde_json",
    "thiserror",
}


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_dependency_direction.py <cargo-metadata.json>", file=sys.stderr)
        return 2

    metadata_path = pathlib.Path(sys.argv[1])
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    local_names = {package["name"] for package in metadata["packages"]}
    violations: list[str] = []

    for package in metadata["packages"]:
        package_name = package["name"]
        allowed = ALLOWED_LOCAL_DEPENDENCIES.get(package_name)
        if allowed is None:
            violations.append(f"unknown first-party crate: {package_name}")
            continue

        local_dependencies = {
            dependency["name"]
            for dependency in package["dependencies"]
            if dependency["name"] in local_names
            and not (
                package_name == "hunteval-reference-deployment"
                and dependency["kind"] == "dev"
            )
        }
        forbidden = sorted(local_dependencies - allowed)
        if forbidden:
            violations.append(
                f"{package_name} has forbidden local dependencies: {', '.join(forbidden)}"
            )
        if package_name == "hunteval-reference-deployment":
            dependencies = {
                dependency["name"]
                for dependency in package["dependencies"]
                if dependency["kind"] != "dev"
            }
            unexpected = sorted(dependencies - REFERENCE_DEPENDENCIES)
            if unexpected:
                violations.append(
                    "hunteval-reference-deployment has unexpected dependencies: "
                    + ", ".join(unexpected)
                )

    if violations:
        for violation in violations:
            print(f"error: {violation}", file=sys.stderr)
        return 1

    print("dependency direction is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
