#!/usr/bin/env python3
"""Fail closed unless GitHub repository settings satisfy the R2 policy."""

import json
import pathlib
import sys


REQUIRED_CHECKS = {
    "Policy",
    "Quality",
    "Tests",
    "Security",
    "End-to-end",
    "Documentation",
    "Package",
}


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def load(path: str) -> object:
    with pathlib.Path(path).open(encoding="utf-8") as handle:
        return json.load(handle)


def main() -> None:
    if len(sys.argv) != 3:
        fail("expected protection and ruleset JSON paths")
    protection = load(sys.argv[1])
    rulesets = load(sys.argv[2])
    if not isinstance(protection, dict) or not isinstance(rulesets, list):
        fail("GitHub settings response has an unexpected shape")
    status_checks = protection.get("required_status_checks") or {}
    checks = set(status_checks.get("contexts", []))
    checks.update(
        item.get("context")
        for item in status_checks.get("checks", [])
        if isinstance(item, dict) and item.get("context")
    )
    missing = REQUIRED_CHECKS.difference(checks)
    reviews = protection.get("required_pull_request_reviews") or {}
    if missing:
        fail(f"required checks are missing: {', '.join(sorted(missing))}")
    if not status_checks.get("strict"):
        fail("required checks do not require an up-to-date branch")
    if reviews.get("required_approving_review_count", 0) < 1:
        fail("at least one approving review is required")
    if not reviews.get("require_code_owner_reviews"):
        fail("CODEOWNER review is not required")
    if not reviews.get("dismiss_stale_reviews"):
        fail("stale approvals are not dismissed")
    if not protection.get("enforce_admins", {}).get("enabled"):
        fail("branch protection does not include administrators")
    if protection.get("allow_force_pushes", {}).get("enabled"):
        fail("force pushes are allowed")
    if protection.get("allow_deletions", {}).get("enabled"):
        fail("protected branch deletion is allowed")
    if not protection.get("required_conversation_resolution", {}).get("enabled"):
        fail("conversation resolution is not required")
    active_tag_rules = [
        item
        for item in rulesets
        if isinstance(item, dict)
        and item.get("target") == "tag"
        and item.get("enforcement") == "active"
    ]
    if not active_tag_rules:
        fail("no active protected-tag ruleset was found")
    print("GitHub branch and tag settings satisfy the committed R2 policy")


if __name__ == "__main__":
    main()
