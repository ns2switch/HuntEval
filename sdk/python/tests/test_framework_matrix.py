import io
import json
import unittest

from hunteval_sdk import (
    AutoGenAdapter,
    CrewAIAdapter,
    CrewAIAdapterConfig,
    DeploymentPeer,
    FrameworkAdapterConfig,
    GoogleAdkAdapter,
    LangGraphAdapter,
    SemanticKernelPreviewAdapter,
)


def submission() -> dict[str, object]:
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


def exercise(context: object) -> dict[str, object]:
    context.create_task("supervisor", "task-1", "Inspect public events")
    context.delegate_task("supervisor", "task-1", "investigator")
    context.start_task("investigator", "task-1")
    context.managed_tool(
        agent_id="investigator",
        task_id="task-1",
        action_id="action-1",
        tool="duckdb_sql",
        purpose="Inspect public events",
        arguments={"query": "SELECT event_id FROM public_events"},
    )
    context.complete_task("investigator", "task-1")
    return submission()


def assert_controls(inputs: object) -> None:
    if inputs["seed"] != 7 or inputs["limits"] != {"max_agents": 2}:
        raise AssertionError("paired controls changed")


class MatrixCrew:
    def __init__(self, context: object) -> None:
        self.context = context

    def kickoff(self, *, inputs: object) -> dict[str, object]:
        assert_controls(inputs)
        return exercise(self.context)


class MatrixGraph:
    def __init__(self, context: object) -> None:
        self.context = context

    def invoke(self, input: object, config: object) -> dict[str, object]:
        assert_controls(input)
        return exercise(self.context)


class MatrixTeam:
    def __init__(self, context: object) -> None:
        self.context = context

    async def run(self, *, task: str) -> dict[str, object]:
        if task != "Find suspicious activity":
            raise AssertionError("paired objective changed")
        return exercise(self.context)


class MatrixRunner:
    def __init__(self, context: object) -> None:
        self.context = context

    def run(self, **values: object) -> object:
        if values["session_id"] != "run-framework-1":
            raise AssertionError("paired run identity changed")
        exercise(self.context)
        return iter([{"author": "supervisor", "final": True}])


class MatrixOrchestration:
    def __init__(self, context: object) -> None:
        self.context = context

    def invoke(self, *, inputs: object) -> dict[str, object]:
        assert_controls(inputs)
        return exercise(self.context)


class FrameworkMatrixTests(unittest.TestCase):
    def test_paired_supervisor_worker_matrix_preserves_declared_controls(self) -> None:
        cases = [
            (
                "crewai",
                lambda config: CrewAIAdapter(
                    CrewAIAdapterConfig(config.deployment, "supervisor"), MatrixCrew
                ),
            ),
            ("langgraph", lambda config: LangGraphAdapter(config, MatrixGraph)),
            (
                "autogen",
                lambda config: AutoGenAdapter(
                    config, lambda context, _: MatrixTeam(context)
                ),
            ),
            (
                "google-adk",
                lambda config: GoogleAdkAdapter(
                    config,
                    MatrixRunner,
                    lambda objective: {"text": objective},
                    lambda _context, _events: submission(),
                ),
            ),
            (
                "semantic-kernel",
                lambda config: SemanticKernelPreviewAdapter(
                    config, MatrixOrchestration
                ),
            ),
        ]
        expected = [
            "register_deployment",
            "task_created",
            "task_delegated",
            "task_started",
            "tool_request",
            "task_completed",
            "final_submission",
        ]
        transcripts: dict[str, list[str]] = {}
        for prefix, build in cases:
            with self.subTest(prefix=prefix):
                peer, output = self._peer(prefix)
                build(self._config(prefix)).run(peer)
                messages = [json.loads(line) for line in output.getvalue().splitlines()]
                transcripts[prefix] = [message["type"] for message in messages]
                self.assertEqual(transcripts[prefix], expected)
                self.assertEqual(
                    messages[0]["deployment"]["agents"],
                    [{"id": "supervisor"}, {"id": "investigator"}],
                )
                self.assertEqual(messages[4]["tool"], "duckdb_sql")
        self.assertEqual(len({tuple(value) for value in transcripts.values()}), 1)

    def _config(self, prefix: str) -> FrameworkAdapterConfig:
        return FrameworkAdapterConfig(
            {
                "id": f"{prefix}-supervisor-worker",
                "agents": [{"id": "supervisor"}, {"id": "investigator"}],
            },
            "supervisor",
            prefix,
        )

    def _peer(self, prefix: str) -> tuple[DeploymentPeer, io.StringIO]:
        values = [
            self._runner_message(
                "runner-start",
                "run_started",
                supported_minimum="0.3",
                supported_maximum="0.3",
                episode_id="episode-1",
                objective="Find suspicious activity",
                tables=["public_events"],
                limits={"max_agents": 2},
                seed=7,
            ),
            self._runner_message(
                "runner-accepted",
                "registration_accepted",
                caused_by_message_id=f"{prefix}-000001",
                selected_protocol_version="0.3",
            ),
            self._runner_message(
                "runner-tool",
                "tool_result",
                caused_by_message_id=f"{prefix}-000005",
                action_id="action-1",
                tool="duckdb_sql",
                outcome="success",
                event_ids=["evt-1"],
                result={"row_count": 1},
            ),
            self._runner_message(
                "runner-end",
                "run_terminated",
                caused_by_message_id=f"{prefix}-000007",
                status="completed",
            ),
        ]
        output = io.StringIO()
        source = io.StringIO("".join(json.dumps(item) + "\n" for item in values))
        return DeploymentPeer(source, output), output

    @staticmethod
    def _runner_message(
        message_id: str, message_type: str, **values: object
    ) -> dict[str, object]:
        return {
            "protocol_version": "0.3",
            "message_id": message_id,
            "run_id": "run-framework-1",
            "timestamp": "2026-08-10T12:00:00Z",
            "type": message_type,
            **values,
        }


if __name__ == "__main__":
    unittest.main()
