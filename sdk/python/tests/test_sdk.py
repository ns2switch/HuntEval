import hashlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from hunteval_sdk import (
    AnalyticalCorpus,
    AnalyticalQuery,
    AnalyticalResult,
    DeploymentPeer,
    ExtensionManifest,
    ExtensionCapabilityPolicy,
    ExtensionConformanceResult,
    ExtensionResolution,
    FinalSubmissionMessage,
    ManagedToolAdapterRequest,
    ManagedToolAdapterResponse,
    RegistrationMessage,
    RetrievalAuditEvent,
    read_public_artifact,
    read_verified_json,
)


class SdkTests(unittest.TestCase):
    def test_contract_examples_parse(self) -> None:
        root = Path(__file__).parents[3]
        corpus = json.loads((root / "examples/contracts/v0.9/analytical-corpus-manifest.json").read_text())
        extension = json.loads((root / "examples/contracts/v0.9/extension-manifest.json").read_text())
        self.assertEqual(AnalyticalCorpus.from_dict(corpus).id, "verified-history")
        self.assertEqual(ExtensionManifest.from_dict(extension).id, "local-query-tool")
        self.assertEqual(AnalyticalCorpus.from_dict(corpus).to_dict(), corpus)
        self.assertEqual(ExtensionManifest.from_dict(extension).to_dict(), extension)

    def test_r7_result_contracts_parse_strictly(self) -> None:
        root = Path(__file__).parents[3] / "examples/contracts/v0.9"
        fixtures = [
            ("analytical-query.json", AnalyticalQuery),
            ("analytical-result.json", AnalyticalResult),
            ("retrieval-audit-event.json", RetrievalAuditEvent),
            ("extension-capability-policy.json", ExtensionCapabilityPolicy),
            ("extension-resolution.json", ExtensionResolution),
            ("extension-conformance-result.json", ExtensionConformanceResult),
            ("managed-tool-adapter-request.json", ManagedToolAdapterRequest),
            ("managed-tool-adapter-response.json", ManagedToolAdapterResponse),
        ]
        for name, model in fixtures:
            value = json.loads((root / name).read_text())
            self.assertIsNotNone(model.from_dict(value))
            changed = dict(value)
            changed["unknown"] = True
            with self.assertRaises(ValueError):
                model.from_dict(changed)

    def test_verified_reader_rejects_digest_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "artifact.json"
            raw = b'{"value":1}'
            path.write_bytes(raw)
            value = read_verified_json(Path(directory), "artifact.json", hashlib.sha256(raw).hexdigest())
            self.assertEqual(value["value"], 1)
            with self.assertRaises(ValueError):
                read_verified_json(Path(directory), "artifact.json", "0" * 64)

    def test_protocol_peer_enforces_registration_and_terminal_state(self) -> None:
        output = io.StringIO()
        peer = DeploymentPeer(io.StringIO(""), output)
        registration = RegistrationMessage(
            "message-1", "run-1", "2026-08-10T00:00:00Z", {"id": "deployment"}
        ).to_dict()
        submission = FinalSubmissionMessage(
            "message-2", "run-1", "2026-08-10T00:00:01Z", "agent-1", {}
        ).to_dict()
        with self.assertRaises(ValueError):
            peer.send(submission)
        peer.send(registration)
        peer.send(submission)
        with self.assertRaises(ValueError):
            peer.send(registration)
        self.assertEqual(registration["type"], "register_deployment")
        self.assertEqual(submission["type"], "final_submission")

    def test_python_messages_match_normative_protocol_fixtures(self) -> None:
        root = Path(__file__).parents[3]
        transcript = json.loads((root / "examples/contracts/protocol-transcript.json").read_text())
        registration = transcript[1]
        self.assertEqual(
            RegistrationMessage(
                registration["message_id"],
                registration["run_id"],
                registration["timestamp"],
                registration["deployment"],
            ).to_dict(),
            registration,
        )
        submission = transcript[-2]
        self.assertEqual(
            FinalSubmissionMessage(
                submission["message_id"],
                submission["run_id"],
                submission["timestamp"],
                submission["agent_id"],
                submission["submission"],
            ).to_dict(),
            submission,
        )
        incoming = io.StringIO(json.dumps(transcript[0], separators=(",", ":")) + "\n")
        self.assertEqual(DeploymentPeer(incoming, io.StringIO()).receive(), transcript[0])

    def test_public_reader_preserves_type_and_rejects_nested_private_fields(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "run.json"
            raw = b'{"schema_version":"0.8","run_id":"run-1"}'
            path.write_bytes(raw)
            artifact = read_public_artifact(Path(directory), "run.json", hashlib.sha256(raw).hexdigest(), "run")
            self.assertEqual(artifact.kind, "run")
            private = b'{"schema_version":"0.8","nested":{"ground_truth":"secret"}}'
            path.write_bytes(private)
            with self.assertRaises(ValueError):
                read_public_artifact(
                    Path(directory), "run.json", hashlib.sha256(private).hexdigest(), "run"
                )

    def test_verified_reader_rejects_symlinked_path_components(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            actual = root / "actual"
            actual.mkdir()
            raw = b'{"value":1}'
            (actual / "artifact.json").write_bytes(raw)
            (root / "linked").symlink_to(actual, target_is_directory=True)
            with self.assertRaises(ValueError):
                read_verified_json(root, "linked/artifact.json", hashlib.sha256(raw).hexdigest())

    def test_compatibility_index_binds_schema_and_example_inventory(self) -> None:
        root = Path(__file__).parents[3]
        paths = sorted(
            [
                path
                for directory in (root / "schemas/v0.9", root / "examples/contracts/v0.9")
                for path in directory.iterdir()
                if path.is_file() and path.name != "sdk-compatibility-index.json"
            ]
            + [root / "examples/contracts/protocol/compatibility-manifest.json"]
        )
        inventory = "".join(
            f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(root)}\n"
            for path in paths
        ).encode()
        index = json.loads((root / "examples/contracts/v0.9/sdk-compatibility-index.json").read_text())
        self.assertEqual(index["fixture_sha256"], hashlib.sha256(inventory).hexdigest())


if __name__ == "__main__":
    unittest.main()
