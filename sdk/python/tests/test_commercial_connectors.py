import unittest

from hunteval_sdk import CommercialRequest, FixtureReplayConnector, build_fixture
from hunteval_sdk.commercial_catalog import READ_ONLY_OPERATIONS


class CommercialConnectorTests(unittest.TestCase):
    def test_every_platform_replays_an_exact_read_only_fixture(self) -> None:
        for platform, operations in READ_ONLY_OPERATIONS.items():
            with self.subTest(platform=platform):
                operation = sorted(operations)[0]
                request = CommercialRequest(
                    platform,
                    operation,
                    "tenant-test",
                    "region-test",
                    {"time_range": "fixture-window", "limit": 2},
                )
                fixture = build_fixture(
                    f"{platform}-fixture",
                    request,
                    {
                        "records": [{"source_id": "record-1", "classification": "observed"}],
                        "truncated": False,
                        "more_available": False,
                    },
                )
                connector = FixtureReplayConnector(platform, {request.sha256: fixture})
                first = connector.execute(request).to_dict()
                second = connector.execute(request).to_dict()
                self.assertEqual(first, second)
                self.assertEqual(first["mode"], "fixture_replay")
                self.assertEqual(first["records"][0]["source_id"], "record-1")

    def test_mutation_unknown_platform_and_arbitrary_transport_fail_closed(self) -> None:
        invalid = [
            ("crowdstrike_falcon", "detection_update", {"limit": 1}),
            ("unknown_platform", "search", {"limit": 1}),
            ("google_secops", "udm_search", {"url": "https://attacker.invalid"}),
            ("elastic_security", "security_search", {"authorization": "secret"}),
            ("elastic_security", "security_search", {"filter": {"headers": {}}}),
        ]
        for platform, operation, arguments in invalid:
            with self.subTest(platform=platform, operation=operation):
                with self.assertRaises(ValueError):
                    CommercialRequest(
                        platform, operation, "tenant-test", "region-test", arguments
                    )

    def test_fixture_digest_and_request_identity_are_enforced(self) -> None:
        request = CommercialRequest(
            "crowdstrike_falcon",
            "detections_search",
            "tenant-test",
            "region-test",
            {"limit": 1},
        )
        fixture = build_fixture(
            "falcon-fixture",
            request,
            {"records": [], "truncated": False, "more_available": False},
        )
        changed = CommercialRequest(
            "crowdstrike_falcon",
            "detections_search",
            "tenant-test",
            "region-test",
            {"limit": 2},
        )
        connector = FixtureReplayConnector(request.platform, {request.sha256: fixture})
        with self.assertRaisesRegex(ValueError, "no exact offline fixture"):
            connector.execute(changed)
        with self.assertRaisesRegex(ValueError, "digest"):
            type(fixture)(
                fixture.fixture_id,
                fixture.request_sha256,
                fixture.response,
                "0" * 64,
            )

    def test_fixture_rejects_nested_secret_material(self) -> None:
        request = CommercialRequest(
            "google_secops", "udm_search", "tenant-test", "region-test", {"limit": 1}
        )
        with self.assertRaisesRegex(ValueError, "prohibited field"):
            build_fixture(
                "leaking-fixture",
                request,
                {
                    "records": [{"metadata": {"token": "must-not-persist"}}],
                    "truncated": False,
                    "more_available": False,
                },
            )


if __name__ == "__main__":
    unittest.main()
