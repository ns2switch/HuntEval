#!/usr/bin/env python3
"""Build a bounded, evaluator-safe quality report for the expanded R8 corpus."""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
import pathlib
import statistics
import subprocess
import sys
from typing import Any


PROVIDERS = ("aws", "azure", "gcp")
TELEMETRY = {
    "aws": ("aws_cloudtrail", "cloudtrail.parquet"),
    "azure": ("azure_activity", "activity.parquet"),
    "gcp": ("gcp_audit", "audit.parquet"),
}


def load_json(path: pathlib.Path) -> Any:
    with path.open("r", encoding="utf-8") as stream:
        return json.load(stream)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def histogram(values: list[Any]) -> dict[str, int]:
    return dict(sorted(collections.Counter(str(value) for value in values).items()))


def reference_recovery(
    datasets: pathlib.Path, worker: pathlib.Path
) -> tuple[int, int]:
    passed = 0
    total = 0
    for provider in PROVIDERS:
        table, parquet = TELEMETRY[provider]
        for number in range(7, 19):
            total += 1
            episode = datasets / provider / f"{provider}-cloud-{number:03d}"
            truth = load_json(episode / "private/ground-truth.json")
            query = (episode / "private/reference-query.sql").read_text(encoding="utf-8")
            command = {
                "tables": [
                    {
                        "name": table,
                        "parquet_path": str(
                            (episode / "public/telemetry" / parquet).resolve()
                        ),
                    }
                ],
                "request": {
                    "query": query,
                    "parameters": [],
                    "limits": {
                        "timeout_ms": 2000,
                        "memory_limit_mb": 128,
                        "max_rows": 1000,
                        "max_output_bytes": 1048576,
                    },
                },
            }
            result = subprocess.run(
                [str(worker.resolve())],
                input=json.dumps(command).encode("utf-8"),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
                timeout=10,
            )
            if result.returncode != 0:
                raise RuntimeError("reference worker failed")
            response = json.loads(result.stdout)
            if response.get("status") != "success":
                raise RuntimeError("reference query was rejected")
            rows = response["result"]["rows"]
            recovered = sorted(
                row[0]["value"]
                for row in rows
                if row and row[0].get("type") == "string"
            )
            expected = sorted(truth["malicious_event_ids"])
            if recovered != expected or response["result"]["truncated"]:
                raise RuntimeError("reference recovery mismatch")
            passed += 1
    return passed, total


def metric_analysis(benchmark: pathlib.Path) -> dict[str, Any]:
    results = []
    for path in sorted((benchmark / "runs").glob("*/result.json")):
        results.append(load_json(path))
    if len(results) != 324:
        raise RuntimeError(f"expected 324 results, found {len(results)}")

    deployment_values: dict[str, dict[str, list[float]]] = collections.defaultdict(
        lambda: collections.defaultdict(list)
    )
    episode_vectors: dict[str, list[tuple[str, int, tuple[Any, ...]]]] = (
        collections.defaultdict(list)
    )
    for result in results:
        key = result["cell"]["key"]
        deployment = key["deployment"]["id"]
        episode = key["episode"]["id"]
        seed = key["seed"]
        vector = []
        for name, metric in sorted(result["metrics"].items()):
            value = metric["value"]
            if value is not None:
                deployment_values[deployment][name].append(float(value))
            if name != "measured_duration_utilization":
                vector.append((name, metric["applicability"], value))
        episode_vectors[episode].append((deployment, seed, tuple(vector)))

    variances = {}
    for deployment, metrics in sorted(deployment_values.items()):
        variances[deployment] = {
            name: {
                "observations": len(values),
                "mean": statistics.fmean(values),
                "population_variance": statistics.pvariance(values),
            }
            for name, values in sorted(metrics.items())
        }

    exact_groups: dict[str, list[str]] = collections.defaultdict(list)
    mean_vectors: dict[str, dict[str, float]] = {}
    for episode, vectors in episode_vectors.items():
        ordered = sorted(vectors)
        signature = hashlib.sha256(
            json.dumps(ordered, sort_keys=True, separators=(",", ":")).encode("utf-8")
        ).hexdigest()
        exact_groups[signature].append(episode)
        collected: dict[str, list[float]] = collections.defaultdict(list)
        for _, _, vector in ordered:
            for name, applicability, value in vector:
                if applicability == "applicable" and value is not None:
                    collected[name].append(float(value))
        mean_vectors[episode] = {
            name: statistics.fmean(values) for name, values in collected.items()
        }

    identical = [
        sorted(group) for group in exact_groups.values() if len(group) > 1
    ]
    identical.sort()
    near = []
    episode_ids = sorted(mean_vectors)
    for index, left in enumerate(episode_ids):
        for right in episode_ids[index + 1 :]:
            if any(left in group and right in group for group in identical):
                continue
            names = sorted(set(mean_vectors[left]) & set(mean_vectors[right]))
            if not names:
                continue
            distance = math.sqrt(
                sum(
                    (mean_vectors[left][name] - mean_vectors[right][name]) ** 2
                    for name in names
                )
                / len(names)
            )
            if distance <= 0.01:
                near.append({"episodes": [left, right], "rms_distance": distance})
            if len(near) >= 100:
                break
        if len(near) >= 100:
            break

    flagged = {
        episode for group in identical for episode in group
    } | {
        episode for pair in near for episode in pair["episodes"]
    }

    state = load_json(benchmark / "benchmark-state.json")
    statuses = histogram([cell["status"] for cell in state["cells"]])
    return {
        "expected_cells": 324,
        "observed_cells": len(results),
        "cell_statuses": statuses,
        "metric_variance_by_reference_deployment": variances,
        "identical_metric_vector_groups": identical,
        "near_identical_mean_metric_pairs": near,
        "distinct_reference_outcome_signatures": len(exact_groups),
        "potentially_redundant_episodes": sorted(flagged),
        "discrimination_findings": [
            "The scripted reference deployments produce identical raw metric outcomes for broad episode groups.",
            "All new malicious episodes currently share one reference-outcome signature because the frozen scripted baselines do not recover their evidence.",
            "This is recorded as a benchmark-review finding; reference deployments were not tuned to private answers.",
        ],
        "redundancy_interpretation": (
            "Flagged groups require individual review. No episode is removed "
            "automatically, and similar scripted-baseline outcomes do not prove "
            "equivalent investigation content."
        ),
    }


def build_report(
    inventory_path: pathlib.Path,
    datasets: pathlib.Path,
    benchmark: pathlib.Path,
    worker: pathlib.Path,
    elapsed_wall_seconds: float | None,
    matrix_jobs: int,
) -> dict[str, Any]:
    inventory = load_json(inventory_path)
    episodes = inventory["episodes"]
    summary = inventory["summary"]
    if summary["episode_count"] != 54 or summary["providers"] != {
        "aws": 18,
        "azure": 18,
        "gcp": 18,
    }:
        raise RuntimeError("corpus composition does not meet the R8 target")
    if summary.get("benign_episodes", 0) < 9:
        raise RuntimeError("benign coverage is below the R8 target")
    if summary["multi_stage_episodes"] < 18 or summary["cross_boundary_episodes"] < 9:
        raise RuntimeError("path or boundary coverage is below the R8 target")

    passed, total = reference_recovery(datasets, worker)
    if passed != total or total != 36:
        raise RuntimeError("expanded reference recovery is incomplete")

    categories = histogram([episode["category"] for episode in episodes])
    tables = collections.Counter()
    services = collections.Counter()
    techniques = collections.Counter()
    for episode in episodes:
        tables.update(episode["telemetry_tables"])
        services.update(episode["cloud_services"])
        techniques.update(episode.get("attack_techniques", []))

    return {
        "schema_version": "1.0",
        "corpus_id": "cloud-expanded-r8",
        "status": "pending_independent_review",
        "execution_evidence": {
            "benchmark_definition_sha256": digest(
                benchmark / "benchmark-definition.json"
            ),
            "benchmark_report_sha256": digest(
                benchmark / "benchmark-report.json"
            ),
            "matrix_elapsed_wall_seconds": elapsed_wall_seconds,
            "matrix_jobs": matrix_jobs,
            "measurement_environment": (
                "local_linux_x86_64"
                if elapsed_wall_seconds is not None
                else "not_recorded_by_analyzer"
            ),
            "exact_revision_ci_rerun_required": True,
        },
        "composition": {
            "episode_count": summary["episode_count"],
            "provider_balance": summary["providers"],
            "category_balance": categories,
            "difficulty_balance": summary["difficulty"],
            "benign_episodes": summary["benign_episodes"],
            "malicious_episodes": summary["episode_count"]
            - summary["benign_episodes"],
            "multi_stage_episodes": summary["multi_stage_episodes"],
            "cross_boundary_episodes": summary["cross_boundary_episodes"],
            "attack_path_length_distribution": histogram(
                [episode["attack_path_length"] for episode in episodes]
            ),
            "timeline_duration_minutes_distribution": histogram(
                [episode["investigation_duration_minutes"] for episode in episodes]
            ),
            "event_count_distribution": histogram(
                [episode["event_count"] for episode in episodes]
            ),
            "entity_count_distribution": histogram(
                [episode["malicious_entity_count"] for episode in episodes]
            ),
            "telemetry_table_coverage": dict(sorted(tables.items())),
            "cloud_service_coverage": dict(sorted(services.items())),
            "attack_technique_aggregate": dict(sorted(techniques.items())),
        },
        "reference_recovery": {
            "expanded_episodes_checked": total,
            "passed": passed,
            "rate": passed / total,
            "private_queries_published": False,
        },
        "reference_matrix": metric_analysis(benchmark),
        "limitations": [
            "The corpus is deterministic synthetic cloud control-plane data.",
            "It does not represent production SIEM execution or universal SOC performance.",
            "The three reference deployments are scripted baselines, not commercial-product benchmarks.",
            "Topology attribution remains topology-dependent.",
            "Unavailable metrics are not inferred or imputed.",
            "New episodes are not release-eligible before independent content-addressed review.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--inventory", type=pathlib.Path, required=True)
    parser.add_argument("--datasets", type=pathlib.Path, required=True)
    parser.add_argument("--benchmark", type=pathlib.Path, required=True)
    parser.add_argument("--worker", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    parser.add_argument("--elapsed-wall-seconds", type=float)
    parser.add_argument("--matrix-jobs", type=int, default=2)
    args = parser.parse_args()
    report = build_report(
        args.inventory,
        args.datasets,
        args.benchmark,
        args.worker,
        args.elapsed_wall_seconds,
        args.matrix_jobs,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        print(f"corpus quality failed: {error}", file=sys.stderr)
        sys.exit(1)
