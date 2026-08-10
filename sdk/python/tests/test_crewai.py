import io
import json
import unittest

from hunteval_sdk import CrewAIAdapter, CrewAIAdapterConfig, DeploymentPeer


def runner_message(message_id: str, message_type: str, **values: object) -> dict[str, object]:
    return {
        "protocol_version": "0.3", "message_id": message_id, "run_id": "run-crew-1",
        "timestamp": "2026-08-10T12:00:00Z", "type": message_type, **values,
    }


class FakeCrew:
    def __init__(self, context: object) -> None:
        self.context = context

    def kickoff(self, *, inputs: object) -> dict[str, object]:
        self.context.create_task("supervisor", "task-1", "Investigate observable activity")
        self.context.delegate_task("supervisor", "task-1", "investigator")
        self.context.start_task("investigator", "task-1")
        result = self.context.managed_tool(
            agent_id="investigator", task_id="task-1", action_id="action-1",
            tool="duckdb_sql", purpose="Inspect public events",
            arguments={"query": "SELECT event_id FROM aws_cloudtrail"},
        )
        self.context.complete_task("investigator", "task-1")
        assert inputs["seed"] == 7
        assert result["event_ids"] == ["evt-1"]
        return {
            "status": "confirmed_malicious_activity", "summary": "Observable activity found.",
            "finding_ids": [], "malicious_event_ids": ["evt-1"],
            "malicious_entity_ids": [], "attack_path": ["evt-1"],
            "attack_techniques": [], "confidence": 0.8, "limitations": [],
        }


class CrewAIConnectorTests(unittest.TestCase):
    def test_crew_uses_runner_mediated_tool_and_structured_submission(self) -> None:
        incoming = [
            runner_message(
                "runner-start", "run_started", supported_minimum="0.3",
                supported_maximum="0.3", episode_id="episode-1",
                objective="Find suspicious activity", tables=["aws_cloudtrail"],
                limits={"max_agents": 2}, seed=7,
            ),
            runner_message(
                "runner-accepted", "registration_accepted",
                caused_by_message_id="crewai-000001", selected_protocol_version="0.3",
            ),
            runner_message(
                "runner-tool", "tool_result", caused_by_message_id="crewai-000005",
                action_id="action-1", tool="duckdb_sql", outcome="success",
                event_ids=["evt-1"], result={"row_count": 1},
            ),
            runner_message(
                "runner-end", "run_terminated", caused_by_message_id="crewai-000007",
                status="completed",
            ),
        ]
        source = io.StringIO("".join(json.dumps(item) + "\n" for item in incoming))
        output = io.StringIO()
        config = CrewAIAdapterConfig(
            deployment={"id": "crewai-test", "agents": [
                {"id": "supervisor"}, {"id": "investigator"},
            ]},
            coordinator_agent_id="supervisor",
        )
        CrewAIAdapter(config, FakeCrew).run(DeploymentPeer(source, output))
        emitted = [json.loads(line) for line in output.getvalue().splitlines()]
        self.assertEqual(
            [item["type"] for item in emitted],
            ["register_deployment", "task_created", "task_delegated", "task_started",
             "tool_request", "task_completed", "final_submission"],
        )
        self.assertEqual(emitted[4]["tool"], "duckdb_sql")
        self.assertEqual(emitted[-1]["submission"]["malicious_event_ids"], ["evt-1"])

    def test_unstructured_crew_output_fails_closed(self) -> None:
        class InvalidCrew:
            def kickoff(self, *, inputs: object) -> str:
                return "unstructured answer"

        incoming = [
            runner_message(
                "runner-start", "run_started", supported_minimum="0.3",
                supported_maximum="0.3", objective="test", tables=[],
                limits={"max_agents": 1}, seed=1,
            ),
            runner_message(
                "runner-accepted", "registration_accepted",
                caused_by_message_id="crewai-000001", selected_protocol_version="0.3",
            ),
        ]
        peer = DeploymentPeer(
            io.StringIO("".join(json.dumps(item) + "\n" for item in incoming)), io.StringIO()
        )
        config = CrewAIAdapterConfig(
            deployment={"id": "crewai-test", "agents": [{"id": "agent-1"}]},
            coordinator_agent_id="agent-1",
        )
        with self.assertRaisesRegex(ValueError, "structured final submission"):
            CrewAIAdapter(config, lambda _: InvalidCrew()).run(peer)


if __name__ == "__main__":
    unittest.main()
