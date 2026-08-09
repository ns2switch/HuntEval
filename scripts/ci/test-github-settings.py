#!/usr/bin/env python3
"""Exercise positive and fail-closed GitHub settings verification fixtures."""

import json
import pathlib
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/ci/verify-github-settings.py"
CHECKS = [
    "Adversarial protocol",
    "Benchmark science",
    "Evidence-backed diagnosis",
    "Policy",
    "Quality",
    "Tests",
    "Security",
    "End-to-end",
    "Documentation",
    "Package",
]
TAG_PATTERN = "refs/tags/v*"


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
tag_rules = [
    {
        "target": "tag",
        "enforcement": "active",
        "bypass_actors": [
            {"actor_id": 12803838, "actor_type": "User", "bypass_mode": "always"}
        ],
        "conditions": {"ref_name": {"include": [TAG_PATTERN], "exclude": []}},
        "rules": [{"type": "creation"}],
    },
    {
        "target": "tag",
        "enforcement": "active",
        "bypass_actors": [],
        "conditions": {"ref_name": {"include": [TAG_PATTERN], "exclude": []}},
        "rules": [{"type": "update"}, {"type": "deletion"}],
    },
]
if run(valid, tag_rules).returncode != 0:
    raise SystemExit("valid GitHub settings fixture was rejected")

invalid = dict(valid)
invalid["allow_force_pushes"] = {"enabled": True}
if run(invalid, tag_rules).returncode == 0:
    raise SystemExit("unsafe GitHub settings fixture was accepted")

missing_check = dict(valid)
missing_check["required_status_checks"] = {
    "contexts": CHECKS[:-1],
    "strict": True,
}
if run(missing_check, tag_rules).returncode == 0:
    raise SystemExit("missing required check was accepted")

if run(valid, tag_rules[:1]).returncode == 0:
    raise SystemExit("release tags without immutable update/deletion rules were accepted")

if run(valid, tag_rules[1:]).returncode == 0:
    raise SystemExit("release tags without restricted creation were accepted")

invalid_creator = [dict(tag_rules[0]), tag_rules[1]]
invalid_creator[0]["bypass_actors"] = [
    {"actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always"}
]
if run(valid, invalid_creator).returncode == 0:
    raise SystemExit("non-maintainer release-tag creation bypass was accepted")

bypassable_immutability = [tag_rules[0], dict(tag_rules[1])]
bypassable_immutability[1]["bypass_actors"] = tag_rules[0]["bypass_actors"]
if run(valid, bypassable_immutability).returncode == 0:
    raise SystemExit("bypassable release-tag immutability was accepted")

print("GitHub settings verification fixtures pass")
