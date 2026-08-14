#!/usr/bin/env python3
"""Keep the Trivy CI/CD gates pinned, blocking, and evidence-producing."""

from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ACTION = "aquasecurity/trivy-action@ed142fd0673e97e23eac54620cfb913e5ce36c25"
VERSION = "version: v0.73.0"


def step(document: str, name: str) -> str:
    marker = f"      - name: {name}\n"
    if document.count(marker) != 1:
        raise SystemExit(f"Trivy workflow must contain exactly one {name!r} step")
    remainder = document.split(marker, 1)[1]
    return remainder.split("      - ", 1)[0]


def require(block: str, values: tuple[str, ...], context: str) -> None:
    missing = [value for value in values if value not in block]
    if missing:
        raise SystemExit(f"{context} is missing required Trivy policy: {missing}")
    if "continue-on-error:" in block:
        raise SystemExit(f"{context} cannot make Trivy advisory")


ci = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")
release = (ROOT / ".github/workflows/release-candidate.yml").read_text(
    encoding="utf-8"
)

source = step(ci, "Scan source dependencies and configuration with Trivy")
require(
    source,
    (
        ACTION,
        VERSION,
        "scan-type: fs",
        "scan-ref: .",
        "scanners: vuln,misconfig",
        "severity: HIGH,CRITICAL",
        "ignore-unfixed: 'false'",
        "exit-code: '1'",
        "format: json",
        "output: .ci-artifacts/trivy-source.json",
    ),
    "source scan",
)
require(ci, (".ci-artifacts/trivy-source.json",), "source evidence upload")

candidate = step(release, "Scan the native candidate with Trivy")
require(
    candidate,
    (
        ACTION,
        VERSION,
        "scan-type: sbom",
        "/evidence/sbom.spdx.json",
        "scanners: vuln",
        "vuln-type: os,library",
        "severity: HIGH,CRITICAL",
        "ignore-unfixed: 'false'",
        "exit-code: '1'",
        "format: json",
        "trivy-candidate-${{ matrix.target }}.json",
    ),
    "native candidate scan",
)
require(
    release,
    ("name: trivy-candidate-${{ matrix.target }}-${{ github.sha }}",),
    "candidate evidence upload",
)

if ci.count(ACTION) != 1 or release.count(ACTION) != 1:
    raise SystemExit("Trivy action invocation count changed unexpectedly")

print("Trivy workflow policy is pinned and fail-closed")
