#!/usr/bin/env python3
"""Prove that incomplete R8 evidence blocks closure and complete evidence passes."""

import importlib.util
import json
import pathlib


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("r8_closure", ROOT / "scripts/r8_closure.py")
if SPEC is None or SPEC.loader is None:
    raise SystemExit("cannot load R8 closure verifier")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

pending = json.loads(
    (ROOT / "examples/contracts/v1.0/r8-evidence-index.json").read_text(encoding="utf-8")
)
try:
    MODULE.verify(pending)
except SystemExit:
    pass
else:
    raise SystemExit("pending R8 evidence was accepted as complete")

complete = dict(pending)
complete["pre_r8_dependency"] = "revised"
complete["milestones"] = {name: "complete" for name in MODULE.MILESTONES}
complete["security_review"] = "passed"
complete["reproducibility_review"] = "passed"
complete["github"] = {
    "status": "passed",
    "run_url": "https://github.com/ns2switch/HuntEval/actions/runs/1",
    "required_checks": sorted(MODULE.REQUIRED_CHECKS),
}
complete["candidate_rehearsal"] = "passed"
complete["native_targets"] = {
    target: {
        "status": "passed",
        "run_url": "https://github.com/ns2switch/HuntEval/actions/runs/1",
        "artifact_hashes": [character * 64 for character in "abc"],
    }
    for target in MODULE.NATIVE_TARGETS
}
complete["artifacts"] = ["a" * 64]
MODULE.verify(complete)

malformed = dict(complete)
malformed["unexpected"] = True
try:
    MODULE.verify(malformed)
except SystemExit:
    pass
else:
    raise SystemExit("unknown R8 evidence field was accepted")

print("R8 closure fail-closed fixtures pass")
