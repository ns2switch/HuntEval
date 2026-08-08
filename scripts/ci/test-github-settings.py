#!/usr/bin/env python3
"""Exercise positive and fail-closed GitHub settings verification fixtures."""

import json
import pathlib
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/ci/verify-github-settings.py"
CHECKS = [
    "Policy",
    "Quality",
    "Tests",
    "Security",
    "End-to-end",
    "Documentation",
    "Package",
]


def run(protection: dict, rulesets: list[dict]) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as directory:
        root = pathlib.Path(directory)
        protection_path = root / "protection.json"
        rulesets_path = root / "rulesets.json"
        protection_path.write_text(json.dumps(protection), encoding="utf-8")
        rulesets_path.write_text(json.dumps(rulesets), encoding="utf-8")
        return subprocess.run(
            [str(CHECKER), str(protection_path), str(rulesets_path)],
            check=False,
            capture_output=True,
            text=True,
        )


valid = {
    "required_status_checks": {"contexts": CHECKS, "strict": True},
    "required_pull_request_reviews": {
        "required_approving_review_count": 1,
        "require_code_owner_reviews": True,
        "dismiss_stale_reviews": True,
    },
    "enforce_admins": {"enabled": True},
    "allow_force_pushes": {"enabled": False},
    "allow_deletions": {"enabled": False},
    "required_conversation_resolution": {"enabled": True},
}
tag_rules = [{"target": "tag", "enforcement": "active"}]
if run(valid, tag_rules).returncode != 0:
    raise SystemExit("valid GitHub settings fixture was rejected")

invalid = dict(valid)
invalid["allow_force_pushes"] = {"enabled": True}
if run(invalid, tag_rules).returncode == 0:
    raise SystemExit("unsafe GitHub settings fixture was accepted")

print("GitHub settings verification fixtures pass")
