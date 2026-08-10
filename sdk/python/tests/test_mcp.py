import io
import json
import unittest
from datetime import datetime, timezone

from hunteval_sdk import MCP_PROTOCOL_REVISION, DeploymentPeer, FrameworkContext, McpSession
from hunteval_sdk.framework import MessageClock


def runner_message(message_id: str, message_type: str, **values: object) -> dict[str, object]:
    return {
        "protocol_version": "0.3",
        "message_id": message_id,
        "run_id": "run-mcp-1",
        "timestamp": "2026-08-10T12:00:00Z",
        "type": message_type,
        **values,
    }


def request(identity: object, method: str, params: object | None = None) -> dict[str, object]:
    value: dict[str, object] = {"jsonrpc": "2.0", "id": identity, "method": method}
    if params is not None:
        value["params"] = params
    return value


def call(identity: int, name: str, arguments: dict[str, object]) -> dict[str, object]:
    return request(identity, "tools/call", {"name": name, "arguments": arguments})


def final_submission() -> dict[str, object]:
    return {
        "status": "confirmed_malicious_activity",
        "summary": "Observable activity found.",
        "finding_ids": [],
        "malicious_event_ids": ["evt-1"],
        "malicious_entity_ids": [],
        "attack_path": ["evt-1"],
        "attack_techniques": [],
        "confidence": 0.8,
        "limitations": [],
    }


class McpTests(unittest.TestCase):
    def _session(self) -> tuple[McpSession, io.StringIO]:
        incoming = runner_message(
            "runner-tool",
            "tool_result",
            caused_by_message_id="mcp-000003",
            action_id="action-1",
            tool="duckdb_sql",
            outcome="success",
            event_ids=["evt-1"],
            result={"row_count": 1},
        )
        output = io.StringIO()
        peer = DeploymentPeer(io.StringIO(json.dumps(incoming) + "\n"), output)
        peer.registered = True
        context = FrameworkContext(
            peer,
            MessageClock(
                "run-mcp-1",
                datetime(2026, 8, 10, 12, 0, tzinfo=timezone.utc),
                "mcp",
            ),
            {
                "objective": "Find suspicious activity",
                "tables": ["public_events"],
                "seed": 7,
                "limits": {"max_agents": 2},
            },
            "mcp",
        )
        return McpSession(context), output

    def _initialize(self, session: McpSession) -> None:
        response = session.handle(
            request(
                1,
                "initialize",
                {
                    "protocolVersion": MCP_PROTOCOL_REVISION,
                    "capabilities": {},
                    "clientInfo": {"name": "unsupported-framework", "version": "1.0"},
                },
            )
        )
        self.assertEqual(response["result"]["protocolVersion"], MCP_PROTOCOL_REVISION)
        self.assertIsNone(
            session.handle(
                {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}}
            )
        )

    def test_mcp_client_completes_runner_mediated_lifecycle(self) -> None:
        session, protocol_output = self._session()
        self._initialize(session)
        tools = session.handle(request(2, "tools/list", {}))["result"]["tools"]
        self.assertEqual(len(tools), 8)
        self.assertFalse(any(tool["name"].startswith("sampling") for tool in tools))

        responses = [
            session.handle(
                call(
                    3,
                    "hunteval.task.create",
                    {
                        "agent_id": "agent-1",
                        "task_id": "task-1",
                        "objective": "Inspect public events",
                    },
                )
            ),
            session.handle(
                call(
                    4,
                    "hunteval.task.start",
                    {"agent_id": "agent-1", "task_id": "task-1"},
                )
            ),
            session.handle(
                call(
                    5,
                    "hunteval.tool.call",
                    {
                        "agent_id": "agent-1",
                        "task_id": "task-1",
                        "action_id": "action-1",
                        "tool": "duckdb_sql",
                        "purpose": "Inspect public events",
                        "arguments": {"query": "SELECT event_id FROM public_events"},
                    },
                )
            ),
            session.handle(
                call(
                    6,
                    "hunteval.task.complete",
                    {"agent_id": "agent-1", "task_id": "task-1"},
                )
            ),
            session.handle(
                call(7, "hunteval.submission.set", {"submission": final_submission()})
            ),
        ]
        self.assertTrue(all(response["result"]["isError"] is False for response in responses))
        self.assertEqual(session.take_submission()["malicious_event_ids"], ["evt-1"])
        emitted = [json.loads(line) for line in protocol_output.getvalue().splitlines()]
        self.assertEqual(
            [item["type"] for item in emitted],
            ["task_created", "task_started", "tool_request", "task_completed"],
        )

    def test_unsupported_capabilities_methods_and_duplicate_ids_fail_closed(self) -> None:
        session, _ = self._session()
        response = session.handle(request(1, "tools/list", {}))
        self.assertEqual(response["error"]["code"], -32001)
        bad_init = session.handle(
            request(
                2,
                "initialize",
                {
                    "protocolVersion": MCP_PROTOCOL_REVISION,
                    "capabilities": {"roots": {}},
                    "clientInfo": {"name": "client", "version": "1"},
                },
            )
        )
        self.assertEqual(bad_init["error"]["code"], -32602)

        session, _ = self._session()
        self._initialize(session)
        unsupported = session.handle(request(9, "roots/list", {}))
        self.assertEqual(unsupported["error"]["code"], -32601)
        duplicate = session.handle(request(9, "ping", {}))
        self.assertEqual(duplicate["error"]["code"], -32600)
        undeclared = session.handle(call(10, "filesystem.read", {}))
        self.assertEqual(undeclared["error"]["code"], -32602)
        self.assertIsNone(
            session.handle({"jsonrpc": "2.0", "method": "notifications/cancelled"})
        )

    def test_stdio_rejects_invalid_json_and_batches_without_executing(self) -> None:
        session, protocol_output = self._session()
        source = io.StringIO('{invalid}\n[{"jsonrpc":"2.0"}]\n')
        output = io.StringIO()
        session.serve(source, output)
        responses = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(responses[0]["error"]["code"], -32700)
        self.assertEqual(responses[1]["error"]["code"], -32600)
        self.assertEqual(protocol_output.getvalue(), "")

    def test_submission_cannot_be_replaced(self) -> None:
        session, _ = self._session()
        self._initialize(session)
        first = session.handle(
            call(2, "hunteval.submission.set", {"submission": final_submission()})
        )
        second = session.handle(
            call(3, "hunteval.submission.set", {"submission": final_submission()})
        )
        self.assertFalse(first["result"]["isError"])
        self.assertEqual(second["error"]["code"], -32001)

    def test_mcp_declares_observable_multi_agent_delegation(self) -> None:
        session, protocol_output = self._session()
        self._initialize(session)
        requests = [
            call(
                2,
                "hunteval.task.create",
                {
                    "agent_id": "supervisor",
                    "task_id": "task-specialist",
                    "objective": "Inspect public activity",
                },
            ),
            call(
                3,
                "hunteval.task.delegate",
                {
                    "agent_id": "supervisor",
                    "task_id": "task-specialist",
                    "target_agent_id": "specialist",
                },
            ),
            call(
                4,
                "hunteval.task.start",
                {"agent_id": "specialist", "task_id": "task-specialist"},
            ),
            call(
                5,
                "hunteval.task.complete",
                {"agent_id": "specialist", "task_id": "task-specialist"},
            ),
        ]
        self.assertTrue(all("result" in session.handle(item) for item in requests))
        emitted = [json.loads(line) for line in protocol_output.getvalue().splitlines()]
        self.assertEqual(
            [item["type"] for item in emitted],
            ["task_created", "task_delegated", "task_started", "task_completed"],
        )
        self.assertEqual(emitted[1]["target_agent_id"], "specialist")


if __name__ == "__main__":
    unittest.main()
