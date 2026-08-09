#!/usr/bin/env python3
"""Build controlled R4 topology inputs from a completed paired benchmark."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


def read_json(path: Path) -> dict:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 512 * 1024 * 1024:
        raise ValueError(f"unsafe input: {path}")
    return json.loads(path.read_bytes())


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical_digest(value: object) -> str:
    return digest_bytes(json.dumps(value, sort_keys=True, separators=(",", ":")).encode())


def tree_digest(root: Path) -> str:
    files = sorted(path for path in root.rglob("*") if path.is_file())
    if not files or len(files) > 10_000 or any(path.is_symlink() for path in files):
        raise ValueError(f"unsafe tree: {root}")
    hasher = hashlib.sha256()
    for path in files:
        relative = path.relative_to(root).as_posix().encode()
        data = path.read_bytes()
        hasher.update(len(relative).to_bytes(8, "big"))
        hasher.update(relative)
        hasher.update(len(data).to_bytes(8, "big"))
        hasher.update(data)
    return hasher.hexdigest()


def file_set_digest(paths: list[Path]) -> str:
    hasher = hashlib.sha256()
    for path in sorted(paths):
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"unsafe control artifact: {path}")
        data = path.read_bytes()
        label = path.as_posix().encode()
        hasher.update(len(label).to_bytes(8, "big"))
        hasher.update(label)
        hasher.update(len(data).to_bytes(8, "big"))
        hasher.update(data)
    return hasher.hexdigest()


def escape_pointer(value: str) -> str:
    return value.replace("~", "~0").replace("/", "~1")


def changed_paths(left: object, right: object, path: str = "") -> set[str]:
    if isinstance(left, dict) and isinstance(right, dict):
        changes: set[str] = set()
        for key in sorted(set(left) | set(right)):
            child = f"{path}/{escape_pointer(key)}"
            if key not in left or key not in right:
                changes.add(child)
            else:
                changes.update(changed_paths(left[key], right[key], child))
        return changes
    if isinstance(left, list) and isinstance(right, list):
        return set() if left == right else {path}
    return set() if left == right else {path}


def metric(cell: dict, name: str) -> float | None:
    value = cell["metrics"][name]["value"]
    if value is not None and not isinstance(value, (int, float)):
        raise ValueError(f"invalid metric {name}")
    return value


def observations_for(cells: list[dict]) -> dict[str, list[float | None]]:
    return {
        "duplicate_work": [metric(cell, "duplicate_tool_work") for cell in cells],
        "investigation_quality": [cell["aggregate_score"] for cell in cells],
        "tool_call_utilization": [metric(cell, "tool_call_utilization") for cell in cells],
        "verified_cost": [metric(cell, "verified_cost_utilization") for cell in cells],
    }


def write_json(path: Path, value: object) -> None:
    if path.exists() or path.is_symlink():
        raise ValueError(f"refusing to overwrite: {path}")
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> int:
    if len(sys.argv) != 8:
        raise ValueError(
            "usage: collect-r4-topology-experiment.py REPO BENCHMARK REPORT "
            "BASELINE_TOPOLOGY CANDIDATE_TOPOLOGY EXPERIMENT_OUT OBSERVATIONS_OUT"
        )
    repo, benchmark, report_path, baseline_path, candidate_path, experiment_out, observations_out = (
        Path(value).resolve() for value in sys.argv[1:]
    )
    definition = read_json(benchmark / "benchmark-definition.json")
    report = read_json(report_path)
    baseline_topology = read_json(baseline_path)
    candidate_topology = read_json(candidate_path)
    baseline_id = "single-agent-scripted"
    candidate_id = "supervisor-specialist-scripted"
    selected: dict[str, list[dict]] = {}
    for deployment in (baseline_id, candidate_id):
        cells = sorted(
            (cell for cell in report["cells"] if cell["deployment_id"] == deployment),
            key=lambda cell: (cell["episode_id"], cell["seed"]),
        )
        if not cells or any(cell["status"] != "completed" for cell in cells):
            raise ValueError(f"incomplete topology cells: {deployment}")
        selected[deployment] = cells
    baseline_keys = [(cell["episode_id"], cell["seed"]) for cell in selected[baseline_id]]
    candidate_keys = [(cell["episode_id"], cell["seed"]) for cell in selected[candidate_id]]
    if baseline_keys != candidate_keys:
        raise ValueError("topology matrices are not paired")

    episode_manifests = [
        repo / "datasets" / episode_id.split("-", 1)[0] / episode_id / "public/manifest.yaml"
        for episode_id, _ in baseline_keys
    ]
    model_assignments = sorted(
        {
            agent["model_assignment"]
            for topology in (baseline_topology, candidate_topology)
            for agent in topology["agents"]
        }
    )
    runtime_binaries = [
        repo / "target/debug/hunteval",
        repo / "target/debug/hunteval-duckdb-worker",
        repo / "target/debug/hunteval-reference-deployment",
    ]
    experiment = {
        "schema_version": "0.6",
        "id": "single-to-supervisor-specialist-v1",
        "baseline_topology_sha256": digest_bytes(baseline_path.read_bytes()),
        "candidate_topology_sha256": digest_bytes(candidate_path.read_bytes()),
        "changed_variables": sorted(changed_paths(baseline_topology, candidate_topology)),
        "control_hashes": {
            "episodes": canonical_digest(definition["episodes"]),
            "seeds": canonical_digest(definition["seeds"]),
            "budgets": file_set_digest(episode_manifests),
            "models": canonical_digest(model_assignments),
            "managed_tool_policy": digest_bytes(
                (repo / "crates/hunteval-duckdb/src/policy.rs").read_bytes()
            ),
            "scoring_profile": definition["scoring_profile"]["sha256"],
            "execution_policy": digest_bytes(
                (repo / "examples/contracts/v0.5/execution-policy.json").read_bytes()
            ),
            "schemas": tree_digest(repo / "schemas"),
            "binaries": file_set_digest(runtime_binaries),
        },
        "paired_cell_ids": sorted(
            cell["cell_id"]
            for deployment in (baseline_id, candidate_id)
            for cell in selected[deployment]
        ),
    }
    observations = {
        "baseline": observations_for(selected[baseline_id]),
        "candidate": observations_for(selected[candidate_id]),
    }
    write_json(experiment_out, experiment)
    write_json(observations_out, observations)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, TypeError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1) from None
