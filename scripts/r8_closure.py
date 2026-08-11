#!/usr/bin/env python3
"""Fail closed unless an R8 evidence index proves every release gate."""

import argparse
import json
import pathlib


REQUIRED_CHECKS = {
    "Policy",
    "Quality",
    "Tests",
    "Security",
    "Adversarial protocol",
    "End-to-end",
    "Documentation",
    "Benchmark science",
    "Evidence-backed diagnosis",
    "Controlled improvement",
    "Knowledge and extensions",
    "Framework connectors",
    "Upstream framework conformance",
    "Commercial connector replay",
    "R8 compatibility",
    "R8 supply chain",
    "Package",
}
MILESTONES = {f"r8-{number:02d}" for number in range(12)}
NATIVE_TARGETS = {
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
}


def fail(reasons: list[str]) -> None:
    for reason in reasons:
        print(f"blocked: {reason}")
    raise SystemExit(1)


def load(path: pathlib.Path) -> dict:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 1024 * 1024:
        raise SystemExit("error: unsafe or oversized R8 evidence index")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError):
        raise SystemExit("error: malformed R8 evidence index") from None
    if not isinstance(value, dict):
        raise SystemExit("error: R8 evidence index must be an object")
    return value


def verify(value: dict) -> None:
    reasons: list[str] = []
    expected = {
        "schema_version",
        "revision",
        "pre_r8_dependency",
        "milestones",
        "security_review",
        "reproducibility_review",
        "github",
        "candidate_rehearsal",
        "native_targets",
        "production_published",
        "artifacts",
        "known_limitations",
    }
    if set(value) != expected or value.get("schema_version") != "1.0":
        raise SystemExit("error: R8 evidence index has unknown, missing, or unsupported fields")
    revision = value.get("revision")
    if not isinstance(revision, str) or len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
        raise SystemExit("error: R8 evidence revision is invalid")
    if value.get("pre_r8_dependency") not in {"satisfied", "revised"}:
        reasons.append("pre-R8 dependency is not satisfied or explicitly revised")
    milestones = value.get("milestones")
    if not isinstance(milestones, dict) or set(milestones) != MILESTONES:
        raise SystemExit("error: R8 milestone inventory is incomplete")
    for milestone, status in sorted(milestones.items()):
        if status != "complete":
            reasons.append(f"{milestone} is {status}, not complete")
    if value.get("security_review") != "passed":
        reasons.append("independent security review has not passed")
    if value.get("reproducibility_review") != "passed":
        reasons.append("independent reproducibility review has not passed")
    github = value.get("github")
    if not isinstance(github, dict) or set(github) != {"status", "run_url", "required_checks"}:
        raise SystemExit("error: GitHub evidence has an invalid shape")
    if github.get("status") != "passed" or not github.get("run_url"):
        reasons.append("protected GitHub evidence has not passed")
    checks = github.get("required_checks")
    if not isinstance(checks, list) or not REQUIRED_CHECKS.issubset(set(checks)):
        reasons.append("one or more required protected checks are absent")
    if value.get("candidate_rehearsal") != "passed":
        reasons.append("immutable non-publishing candidate rehearsal has not passed")
    native_targets = value.get("native_targets")
    if not isinstance(native_targets, dict) or set(native_targets) != NATIVE_TARGETS:
        raise SystemExit("error: native target evidence inventory is incomplete")
    for target, evidence in sorted(native_targets.items()):
        if not isinstance(evidence, dict) or set(evidence) != {"status", "run_url", "artifact_hashes"}:
            raise SystemExit(f"error: malformed native target evidence: {target}")
        hashes = evidence.get("artifact_hashes")
        if evidence.get("status") != "passed" or not evidence.get("run_url"):
            reasons.append(f"native target evidence has not passed: {target}")
        if not isinstance(hashes, list) or len(hashes) < 3:
            reasons.append(f"native target artifact evidence is incomplete: {target}")
        elif any(
            not isinstance(item, str)
            or len(item) != 64
            or any(character not in "0123456789abcdef" for character in item)
            for item in hashes
        ):
            raise SystemExit(f"error: malformed native target artifact identity: {target}")
    if value.get("production_published") is not False:
        raise SystemExit("error: R8 cannot claim production publication")
    artifacts = value.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        reasons.append("no content-addressed closure artifacts are recorded")
    elif any(
        not isinstance(item, str)
        or len(item) != 64
        or any(character not in "0123456789abcdef" for character in item)
        for item in artifacts
    ):
        raise SystemExit("error: closure artifact identity is malformed")
    limitations = value.get("known_limitations")
    if not isinstance(limitations, list) or not limitations:
        raise SystemExit("error: known limitations are absent")
    if reasons:
        fail(reasons)
    print("R8 closure evidence satisfies every declared gate")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("evidence_index")
    args = parser.parse_args()
    verify(load(pathlib.Path(args.evidence_index)))


if __name__ == "__main__":
    main()
