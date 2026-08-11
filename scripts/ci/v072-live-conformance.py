#!/usr/bin/env python3
"""Run one protected live-read-only worker call and emit a bounded attestation."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import resource
import signal
import stat
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any

MAX_COMMAND_BYTES = 1_048_576
MAX_SECRET_BYTES = 64 * 1024
MAX_WORKER_OUTPUT_BYTES = 65 * 1024 * 1024
SAFE_REASON = re.compile(r"^[a-z][a-z0-9_]{0,63}$")
ENFORCEMENT_KINDS = {"egress_proxy", "host_firewall", "network_namespace"}


@dataclass(frozen=True)
class WorkerExecution:
    returncode: int
    stdout: bytes
    stderr: bytes


def regular_file(path: Path, maximum: int) -> bytes:
    metadata = path.lstat()
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise SystemExit(f"unsafe input file: {path}")
    if metadata.st_size == 0 or metadata.st_size > maximum:
        raise SystemExit(f"input file outside size bound: {path}")
    return path.read_bytes()


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def require_object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SystemExit(f"{label} must be an object")
    return value


def validate_command(command: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any]]:
    if set(command) != {"policy", "request", "target"}:
        raise SystemExit("worker command fields are invalid")
    policy = require_object(command["policy"], "policy")
    envelope = require_object(command["request"], "request")
    request = require_object(envelope.get("request"), "commercial request")
    if policy.get("mode") != "live_read_only":
        raise SystemExit("live conformance requires live_read_only mode")
    if policy.get("platform") != request.get("platform"):
        raise SystemExit("policy and request platform differ")
    if not isinstance(policy.get("origin"), str) or not policy["origin"].startswith("https://"):
        raise SystemExit("policy origin is invalid")
    return policy, envelope


def validate_enforcement(
    value: dict[str, Any], policy: dict[str, Any], worker_sha256: str
) -> None:
    if set(value) != {
        "allowed_origins",
        "enforcement",
        "schema_version",
        "worker_sha256",
    }:
        raise SystemExit("egress enforcement fields are invalid")
    if value["schema_version"] != "0.1" or value["enforcement"] not in ENFORCEMENT_KINDS:
        raise SystemExit("egress enforcement is unsupported")
    if value["worker_sha256"] != worker_sha256:
        raise SystemExit("egress enforcement is bound to another worker")
    if value["allowed_origins"] != [policy["origin"]]:
        raise SystemExit("egress enforcement does not bind the exact origin")


def safe_reason(value: Any) -> str:
    if not isinstance(value, str) or SAFE_REASON.fullmatch(value) is None:
        raise SystemExit("worker returned an unsafe reason code")
    return value


def summarize_response(value: dict[str, Any]) -> tuple[str, str | None, int, bool]:
    status = value.get("status")
    if status == "failure":
        return "failed", safe_reason(value.get("reason_code")), 0, False
    if status != "completed" or set(value) != {"response", "status"}:
        raise SystemExit("worker response envelope is invalid")
    response = require_object(value["response"], "gateway response")
    gateway_status = response.get("status")
    if gateway_status == "error":
        return "failed", safe_reason(response.get("reason_code")), 0, False
    if gateway_status != "success":
        raise SystemExit("gateway response status is invalid")
    result = require_object(response.get("result"), "gateway result")
    records = result.get("records")
    if not isinstance(records, list):
        raise SystemExit("gateway result records are invalid")
    return "passed", None, len(records), bool(result.get("truncated", False))


def run_supervised_worker(
    worker: Path, input_bytes: bytes, timeout: float
) -> WorkerExecution:
    """Run one worker in a bounded process group with file-size limits."""
    output_limit = MAX_WORKER_OUTPUT_BYTES + 1
    error_limit = 64 * 1024 + 1

    def constrain_output() -> None:
        resource.setrlimit(resource.RLIMIT_FSIZE, (output_limit, output_limit))

    with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        process = subprocess.Popen(
            [str(worker.resolve())],
            stdin=subprocess.PIPE,
            stdout=stdout,
            stderr=stderr,
            env={},
            start_new_session=True,
            preexec_fn=constrain_output,
        )
        try:
            process.communicate(input=input_bytes, timeout=timeout)
        except subprocess.TimeoutExpired as error:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise SystemExit("worker process tree timed out") from error
        stdout.seek(0)
        stderr.seek(0)
        output = stdout.read(output_limit)
        errors = stderr.read(error_limit)
    if len(output) >= output_limit or len(errors) >= error_limit:
        raise SystemExit("worker output exceeded its conformance bound")
    return WorkerExecution(process.returncode, output, errors)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--command", type=Path, required=True)
    parser.add_argument("--secret", type=Path, required=True)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--egress-enforcement", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    arguments = parser.parse_args()

    if arguments.output.exists() or arguments.output.is_symlink():
        raise SystemExit("output must not already exist")
    command_bytes = regular_file(arguments.command, MAX_COMMAND_BYTES)
    secret = regular_file(arguments.secret, MAX_SECRET_BYTES)
    worker_bytes = regular_file(arguments.worker, 512 * 1024 * 1024)
    enforcement_bytes = regular_file(arguments.egress_enforcement, 64 * 1024)
    if arguments.secret.stat().st_mode & 0o077:
        raise SystemExit("secret file must not be accessible by group or other users")

    try:
        command = require_object(json.loads(command_bytes), "worker command")
        enforcement = require_object(json.loads(enforcement_bytes), "egress enforcement")
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SystemExit("conformance input is not valid JSON") from error
    policy, envelope = validate_command(command)
    worker_sha256 = digest(worker_bytes)
    validate_enforcement(enforcement, policy, worker_sha256)

    timeout = int(policy.get("timeout_ms", 0)) / 1000 + 10
    if timeout < 10 or timeout > 310:
        raise SystemExit("worker timeout is outside the supported range")
    completed = run_supervised_worker(
        arguments.worker,
        command_bytes + b"\n" + secret,
        timeout,
    )
    if secret in completed.stdout or secret in completed.stderr:
        raise SystemExit("worker output contains secret material")
    if completed.returncode != 0:
        raise SystemExit("worker process failed")
    try:
        response = require_object(json.loads(completed.stdout), "worker response")
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise SystemExit("worker response is not valid JSON") from error
    status, reason_code, record_count, truncated = summarize_response(response)
    request = require_object(envelope["request"], "commercial request")
    attestation = {
        "schema_version": "0.1",
        "status": status,
        "reason_code": reason_code,
        "platform": request.get("platform"),
        "operation": request.get("operation"),
        "tenant_alias": request.get("tenant_alias"),
        "region": request.get("region"),
        "request_id": envelope.get("request_id"),
        "action_id": envelope.get("action_id"),
        "record_count": record_count,
        "truncated": truncated,
        "command_sha256": digest(canonical(command)),
        "policy_sha256": digest(canonical(policy)),
        "target_sha256": digest(canonical(command["target"])),
        "worker_sha256": worker_sha256,
        "egress_enforcement_sha256": digest(canonical(enforcement)),
        "raw_response_persisted": False,
        "secret_persisted": False,
        "production_scored": False,
    }
    arguments.output.parent.mkdir(parents=True, exist_ok=True)
    with arguments.output.open("x", encoding="utf-8") as output:
        json.dump(attestation, output, sort_keys=True, separators=(",", ":"))
        output.write("\n")
    print(f"live conformance attestation: {arguments.output}")
    if status != "passed":
        raise SystemExit("live conformance did not pass")


if __name__ == "__main__":
    main()
