import io
import json
import unittest

from hunteval_sdk import (
    AutoGenAdapter,
    DeploymentPeer,
    FrameworkAdapterConfig,
    GoogleAdkAdapter,
    LangGraphAdapter,
    SemanticKernelPreviewAdapter,
)


def runner_message(message_id: str, message_type: str, **values: object) -> dict[str, object]:
    return {
        "protocol_version": "0.3",
        "message_id": message_id,
        "run_id": "run-framework-1",
        "timestamp": "2026-08-10T12:00:00Z",
        "type": message_type,
        **values,
    }


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


def exercise_context(context: object) -> dict[str, object]:
    context.create_task("agent-1", "task-1", "Inspect public events")
    context.start_task("agent-1", "task-1")
    result = context.managed_tool(
        agent_id="agent-1",
        task_id="task-1",
        action_id="action-1",
        tool="duckdb_sql",
        purpose="Inspect public events",
        arguments={"query": "SELECT event_id FROM public_events"},
    )
    if result["event_ids"] != ["evt-1"]:
        raise AssertionError("unexpected fixture tool result")
    context.complete_task("agent-1", "task-1")
    return submission()


class FakeLangGraph:
    def __init__(self, context: object) -> None:
        self.context = context

    def invoke(self, input: object, config: object) -> dict[str, object]:
        if input["seed"] != 7 or not config["configurable"]["hunteval_run_id"]:
            raise AssertionError("LangGraph public input mapping is invalid")
        return exercise_context(self.context)


class FakeAutoGen:
    def __init__(self, context: object) -> None:
        self.context = context

    async def run(self, *, task: str) -> dict[str, object]:
        if task != "Find suspicious activity":
            raise AssertionError("AutoGen task mapping is invalid")
        return exercise_context(self.context)


class FakeRunner:
    def __init__(self, context: object) -> None:
        self.context = context

    def run(
        self,
        *,
        user_id: str,
        session_id: str,
        new_message: object,
        state_delta: object = None,
        run_config: object = None,
    ) -> object:
        if (
            user_id != "agent-1"
            or session_id != "run-framework-1"
            or new_message != {"text": "Find suspicious activity"}
            or state_delta is not None
            or run_config is not None
        ):
            raise AssertionError("ADK public Runner mapping is invalid")
        exercise_context(self.context)
        return iter([{"author": "agent-1", "final": True}])


class FakeOrchestration:
    def __init__(self, context: object) -> None:
        self.context = context

    def invoke(self, *, inputs: object) -> dict[str, object]:
        if inputs["run_id"] != "run-framework-1":
            raise AssertionError("Semantic Kernel public input mapping is invalid")
        return exercise_context(self.context)


class FrameworkConnectorTests(unittest.TestCase):
    def _peer(self, prefix: str) -> tuple[DeploymentPeer, io.StringIO]:
        incoming = [
            runner_message(
                "runner-start",
                "run_started",
                supported_minimum="0.3",
                supported_maximum="0.3",
                episode_id="episode-1",
                objective="Find suspicious activity",
                tables=["public_events"],
                limits={"max_agents": 1},
                seed=7,
            ),
            runner_message(
                "runner-accepted",
                "registration_accepted",
                caused_by_message_id=f"{prefix}-000001",
                selected_protocol_version="0.3",
            ),
            runner_message(
                "runner-tool",
                "tool_result",
                caused_by_message_id=f"{prefix}-000004",
                action_id="action-1",
                tool="duckdb_sql",
                outcome="success",
                event_ids=["evt-1"],
                result={"row_count": 1},
            ),
            runner_message(
                "runner-end",
                "run_terminated",
                caused_by_message_id=f"{prefix}-000006",
                status="completed",
            ),
        ]
        output = io.StringIO()
        source = io.StringIO("".join(json.dumps(item) + "\n" for item in incoming))
        return DeploymentPeer(source, output), output

    def _config(self, prefix: str) -> FrameworkAdapterConfig:
        return FrameworkAdapterConfig(
            {"id": f"{prefix}-deployment", "agents": [{"id": "agent-1"}]},
            "agent-1",
            prefix,
        )

    def test_native_connectors_share_the_runner_mediated_lifecycle(self) -> None:
        cases = [
            (
                "langgraph",
                lambda config: LangGraphAdapter(config, FakeLangGraph),
            ),
            (
                "autogen",
                lambda config: AutoGenAdapter(
                    config, lambda context, _: FakeAutoGen(context)
                ),
            ),
            (
                "google-adk",
                lambda config: GoogleAdkAdapter(
                    config,
                    FakeRunner,
                    lambda objective: {"text": objective},
                    lambda _context, events: submission()
                    if events == ({"author": "agent-1", "final": True},)
                    else {},
                ),
            ),
            (
                "semantic-kernel",
                lambda config: SemanticKernelPreviewAdapter(
                    config, FakeOrchestration
                ),
            ),
        ]
        normalized_sequences: list[list[str]] = []
        for prefix, build in cases:
            with self.subTest(prefix=prefix):
                peer, output = self._peer(prefix)
                build(self._config(prefix)).run(peer)
                emitted = [json.loads(line) for line in output.getvalue().splitlines()]
                self.assertEqual(
                    [item["type"] for item in emitted],
                    [
                        "register_deployment",
                        "task_created",
                        "task_started",
                        "tool_request",
                        "task_completed",
                        "final_submission",
                    ],
                )
                normalized_sequences.append([item["type"] for item in emitted])
        self.assertTrue(
            all(sequence == normalized_sequences[0] for sequence in normalized_sequences)
        )

    def test_context_rejects_duplicate_task_before_framework_invocation_continues(self) -> None:
        class DuplicateGraph:
            def __init__(self, context: object) -> None:
                self.context = context

            def invoke(self, input: object, config: object) -> dict[str, object]:
                self.context.create_task("agent-1", "task-1", "First")
                self.context.create_task("agent-1", "task-1", "Duplicate")
                return submission()

        peer, _ = self._peer("langgraph")
        with self.assertRaisesRegex(ValueError, "duplicated"):
            LangGraphAdapter(self._config("langgraph"), DuplicateGraph).run(peer)

    def test_semantic_kernel_cannot_claim_stable_support(self) -> None:
        peer, _ = self._peer("semantic-kernel")
        adapter = SemanticKernelPreviewAdapter(
            self._config("semantic-kernel"), FakeOrchestration, support_status="supported"
        )
        with self.assertRaisesRegex(ValueError, "remain preview"):
            adapter.run(peer)


if __name__ == "__main__":
    unittest.main()
