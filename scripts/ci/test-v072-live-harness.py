#!/usr/bin/env python3
"""Offline negative test for the protected commercial live harness."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def test_process_tree_timeout(root: Path) -> None:
    module_path = Path("scripts/ci/v072-live-conformance.py").resolve()
    specification = importlib.util.spec_from_file_location("v072_live", module_path)
    if specification is None or specification.loader is None:
        raise SystemExit("live harness module cannot be loaded")
    module = importlib.util.module_from_spec(specification)
    sys.modules[specification.name] = module
    specification.loader.exec_module(module)
    worker = root / "blocking-worker"
    worker.write_text(
        "#!/usr/bin/env python3\n"
        "import subprocess, time\n"
        "subprocess.Popen(['sleep', '30'])\n"
        "time.sleep(30)\n",
        encoding="utf-8",
    )
    os.chmod(worker, 0o700)
    try:
        module.run_supervised_worker(worker, b"input", 0.1)
    except SystemExit as error:
        if str(error) != "worker process tree timed out":
            raise
    else:
        raise SystemExit("process-tree timeout did not fail closed")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("worker path is required")
    worker = Path(sys.argv[1]).resolve()
    if not worker.is_file() or worker.is_symlink():
        raise SystemExit("worker is unavailable")
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        test_process_tree_timeout(root)
        command_path = root / "command.json"
        secret_path = root / "secret"
        enforcement_path = root / "egress.json"
        output_path = root / "attestation.json"
        command = {
            "policy": {
                "policy_version": "0.1",
                "mode": "live_read_only",
                "platform": "crowdstrike_falcon",
                "origin": "https://api.crowdstrike.invalid",
                "operations": ["detections_search"],
                "secret_reference": "fixture-read-only",
                "max_requests": 1,
                "max_response_bytes": 4096,
                "max_records": 10,
                "timeout_ms": 1000,
            },
            "target": {},
            "request": {
                "request_id": "request-1",
                "agent_id": "agent-1",
                "task_id": "task-1",
                "action_id": "action-1",
                "request": {
                    "platform": "crowdstrike_falcon",
                    "operation": "detections_search",
                    "tenant_alias": "tenant-test",
                    "region": "region-test",
                    "arguments": {"limit": 1},
                },
            },
        }
        command_path.write_text(json.dumps(command), encoding="utf-8")
        canary = b"canary-value-that-must-not-leak"
        secret_path.write_bytes(canary)
        os.chmod(secret_path, 0o600)
        worker_sha256 = hashlib.sha256(worker.read_bytes()).hexdigest()
        enforcement_path.write_text(
            json.dumps(
                {
                    "schema_version": "0.1",
                    "enforcement": "host_firewall",
                    "allowed_origins": ["https://api.crowdstrike.invalid"],
                    "worker_sha256": worker_sha256,
                }
            ),
            encoding="utf-8",
        )
        completed = subprocess.run(
            [
                sys.executable,
                "scripts/ci/v072-live-conformance.py",
                "--command",
                str(command_path),
                "--secret",
                str(secret_path),
                "--worker",
                str(worker),
                "--egress-enforcement",
                str(enforcement_path),
                "--output",
                str(output_path),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if completed.returncode == 0 or not output_path.is_file():
            raise SystemExit("negative live conformance did not fail closed")
        combined = completed.stdout + completed.stderr + output_path.read_bytes()
        if canary in combined:
            raise SystemExit("live conformance leaked the canary")
        attestation = json.loads(output_path.read_text(encoding="utf-8"))
        if attestation["status"] != "failed" or attestation["reason_code"] not in {
            "denied_address",
            "transport_failure",
        }:
            raise SystemExit("live conformance failure was not typed")
        print("commercial live harness negative path: passed")


if __name__ == "__main__":
    main()
