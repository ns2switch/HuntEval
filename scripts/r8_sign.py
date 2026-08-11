#!/usr/bin/env python3
"""Create and verify detached SSH signatures under an explicit offline R8 policy."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import tempfile

MAX_ARTIFACT_BYTES = 512 * 1024 * 1024
NAMESPACE = "hunteval-release"


def fail(message: str) -> None:
    raise SystemExit(f"error: {message}")


def digest(path: pathlib.Path) -> str:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_ARTIFACT_BYTES:
        fail(f"unsafe or oversized file: {path}")
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_json(path: pathlib.Path) -> dict:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"invalid JSON policy or inventory: {type(error).__name__}")
    if not isinstance(value, dict):
        fail("JSON policy or inventory must be an object")
    return value


def policy(path: pathlib.Path) -> dict:
    value = load_json(path)
    required = {
        "schema_version",
        "signer_identity",
        "namespace",
        "public_key",
        "key_fingerprint",
        "repository",
        "ref",
        "workflow",
        "valid_from_epoch",
        "valid_until_epoch",
        "revoked_fingerprints",
    }
    if set(value) != required or value.get("schema_version") != "1.0":
        fail("signature policy has unknown or missing fields")
    if value.get("namespace") != NAMESPACE:
        fail("signature namespace is not supported")
    if not isinstance(value.get("signer_identity"), str) or not value["signer_identity"]:
        fail("signature identity is invalid")
    if not isinstance(value.get("public_key"), str) or "\n" in value["public_key"]:
        fail("signature public key is invalid")
    if not isinstance(value.get("revoked_fingerprints"), list):
        fail("signature revocation inventory is invalid")
    if not all(isinstance(value.get(field), str) and value[field] for field in ("repository", "ref", "workflow")):
        fail("signature workflow identity is invalid")
    if not all(isinstance(value.get(field), int) for field in ("valid_from_epoch", "valid_until_epoch")):
        fail("signature validity interval is invalid")
    if value["valid_from_epoch"] >= value["valid_until_epoch"]:
        fail("signature validity interval is empty")
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8") as public_key:
        public_key.write(value["public_key"] + "\n")
        public_key.flush()
        result = subprocess.run(
            ["ssh-keygen", "-lf", public_key.name, "-E", "sha256"],
            check=False,
            capture_output=True,
            text=True,
        )
    if result.returncode != 0 or value["key_fingerprint"] not in result.stdout:
        fail("signature policy fingerprint does not match its public key")
    return value


def sign(args: argparse.Namespace) -> None:
    artifact = pathlib.Path(args.artifact).resolve()
    key = pathlib.Path(args.key).resolve()
    policy_path = pathlib.Path(args.policy).resolve()
    output = pathlib.Path(args.output)
    if not output.is_absolute() or output == pathlib.Path("/") or output.exists():
        fail("signature output must be a new absolute directory")
    value = policy(policy_path)
    if key.is_symlink() or not key.is_file():
        fail("signing key must be a regular file")
    generated = pathlib.Path(str(artifact) + ".sig")
    if generated.exists() or generated.is_symlink():
        fail("signing sidecar path already exists")
    output.mkdir(mode=0o700, parents=False)
    signature = output / "artifact.sig"
    result = subprocess.run(
        ["ssh-keygen", "-Y", "sign", "-f", str(key), "-n", NAMESPACE, str(artifact)],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 or not generated.is_file():
        fail("detached signing failed")
    generated.replace(signature)
    (output / "signature-policy.json").write_bytes(policy_path.read_bytes())
    inventory = {
        "schema_version": "1.0",
        "artifact_sha256": digest(artifact),
        "signature_sha256": digest(signature),
        "policy_sha256": digest(output / "signature-policy.json"),
        "signer_identity": value["signer_identity"],
        "key_fingerprint": value["key_fingerprint"],
        "namespace": NAMESPACE,
    }
    (output / "signature-inventory.json").write_text(
        json.dumps(inventory, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def verify(args: argparse.Namespace) -> None:
    artifact = pathlib.Path(args.artifact).resolve()
    root = pathlib.Path(args.signature_root).resolve()
    value = policy(root / "signature-policy.json")
    inventory = load_json(root / "signature-inventory.json")
    expected_inventory = {
        "schema_version",
        "artifact_sha256",
        "signature_sha256",
        "policy_sha256",
        "signer_identity",
        "key_fingerprint",
        "namespace",
    }
    if set(inventory) != expected_inventory or inventory.get("schema_version") != "1.0":
        fail("signature inventory has unknown or missing fields")
    signature = root / "artifact.sig"
    if inventory.get("artifact_sha256") != digest(artifact):
        fail("signed artifact bytes changed")
    if inventory.get("signature_sha256") != digest(signature):
        fail("detached signature bytes changed")
    if inventory.get("policy_sha256") != digest(root / "signature-policy.json"):
        fail("signature policy bytes changed")
    if inventory.get("key_fingerprint") != value["key_fingerprint"]:
        fail("signature fingerprint substitution detected")
    if value["key_fingerprint"] in value["revoked_fingerprints"]:
        fail("signature key is revoked")
    if not value["valid_from_epoch"] <= args.at_epoch <= value["valid_until_epoch"]:
        fail("signature policy is not valid at the requested epoch")
    if (value["repository"], value["ref"], value["workflow"]) != (
        args.repository,
        args.ref,
        args.workflow,
    ):
        fail("signature workflow identity does not match policy")
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8") as allowed:
        allowed.write(f'{value["signer_identity"]} {value["public_key"]}\n')
        allowed.flush()
        with artifact.open("rb") as content:
            result = subprocess.run(
                [
                    "ssh-keygen",
                    "-Y",
                    "verify",
                    "-f",
                    allowed.name,
                    "-I",
                    value["signer_identity"],
                    "-n",
                    NAMESPACE,
                    "-s",
                    str(signature),
                ],
                stdin=content,
                check=False,
                capture_output=True,
            )
    if result.returncode != 0:
        fail("offline detached signature verification failed")


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    signer = commands.add_parser("sign")
    signer.add_argument("--artifact", required=True)
    signer.add_argument("--key", required=True)
    signer.add_argument("--policy", required=True)
    signer.add_argument("--output", required=True)
    verifier = commands.add_parser("verify")
    verifier.add_argument("--artifact", required=True)
    verifier.add_argument("--signature-root", required=True)
    verifier.add_argument("--repository", required=True)
    verifier.add_argument("--ref", required=True)
    verifier.add_argument("--workflow", required=True)
    verifier.add_argument("--at-epoch", required=True, type=int)
    args = parser.parse_args()
    if args.command == "sign":
        sign(args)
    else:
        verify(args)


if __name__ == "__main__":
    main()
