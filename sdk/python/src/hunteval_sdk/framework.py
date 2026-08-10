"""Framework-neutral lifecycle for HuntEval deployment adapters."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
import asyncio
import inspect
from typing import Any, Awaitable, Mapping, cast

from ._validation import identifier
from .protocol import DeploymentPeer, FinalSubmissionMessage, ProtocolError, RegistrationMessage

MAX_TEXT_BYTES = 65_536
MAX_OBJECT_ITEMS = 4_096


@dataclass(frozen=True, slots=True)
class FrameworkAdapterConfig:
    """Framework-independent deployment metadata and message identity policy."""

    deployment: Mapping[str, Any]
    coordinator_agent_id: str
    message_prefix: str

    def __post_init__(self) -> None:
        identifier(self.coordinator_agent_id)
        identifier(self.message_prefix)
        if not isinstance(self.deployment, Mapping) or not self.deployment.get("agents"):
            raise ValueError("framework deployment metadata must declare agents")
        agents = self.deployment["agents"]
        if not isinstance(agents, list) or not any(
            isinstance(agent, Mapping) and agent.get("id") == self.coordinator_agent_id
            for agent in agents
        ):
            raise ValueError("framework coordinator must be a declared deployment agent")


@dataclass(slots=True)
class MessageClock:
    """Deterministic message identities and timestamps for one deployment run."""

    run_id: str
    initial: datetime
    prefix: str
    sequence: int = 0

    def next(self) -> tuple[str, str]:
        self.sequence += 1
        timestamp = self.initial + timedelta(microseconds=self.sequence)
        value = timestamp.astimezone(timezone.utc).isoformat(timespec="microseconds")
        return f"{self.prefix}-{self.sequence:06d}", value.replace("+00:00", "Z")


@dataclass(slots=True)
class FrameworkContext:
    """The only HuntEval authority exposed to a framework connector."""

    peer: DeploymentPeer
    clock: MessageClock
    run_started: Mapping[str, Any]
    framework: str
    _tasks: set[str] = field(default_factory=set)
    _started_tasks: set[str] = field(default_factory=set)
    _closed_tasks: set[str] = field(default_factory=set)
    _actions: set[str] = field(default_factory=set)
    _evidence: set[str] = field(default_factory=set)
    _findings: set[str] = field(default_factory=set)

    @property
    def kickoff_inputs(self) -> dict[str, Any]:
        """Return the bounded public inputs supplied by the HuntEval runner."""
        return {
            "run_id": self.clock.run_id,
            "objective": self.run_started["objective"],
            "tables": list(self.run_started.get("tables", [])),
            "seed": self.run_started["seed"],
            "limits": dict(self.run_started["limits"]),
        }

    def create_task(self, agent_id: str, task_id: str, objective: str) -> None:
        if task_id in self._tasks:
            raise ProtocolError("framework task identity is duplicated")
        _bounded_text(objective, "task objective", 4_096)
        self._send(
            "task_created",
            agent_id=identifier(agent_id),
            task={
                "id": identifier(task_id),
                "objective": objective,
                "priority": "normal",
                "dependencies": [],
                "required_capabilities": [],
                "parent_task_id": None,
            },
        )
        self._tasks.add(task_id)

    def delegate_task(self, agent_id: str, task_id: str, target_agent_id: str) -> None:
        self._require_pending(task_id)
        self._send(
            "task_delegated",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            target_agent_id=identifier(target_agent_id),
            reason_code=f"{self.framework}_assignment",
        )

    def reassign_task(self, agent_id: str, task_id: str, target_agent_id: str) -> None:
        self._require_pending(task_id)
        self._send(
            "task_reassigned",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            target_agent_id=identifier(target_agent_id),
        )

    def start_task(self, agent_id: str, task_id: str) -> None:
        self._require_pending(task_id)
        self._send(
            "task_started", agent_id=identifier(agent_id), task_id=identifier(task_id)
        )
        self._started_tasks.add(task_id)

    def complete_task(self, agent_id: str, task_id: str) -> None:
        self._require_started(task_id)
        self._send(
            "task_completed", agent_id=identifier(agent_id), task_id=identifier(task_id)
        )
        self._close_task(task_id)

    def fail_task(self, agent_id: str, task_id: str, reason_code: str) -> None:
        self._require_started(task_id)
        _bounded_text(reason_code, "task failure reason", 256)
        self._send(
            "task_failed",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            reason_code=reason_code,
        )
        self._close_task(task_id)

    def cancel_task(self, agent_id: str, task_id: str) -> None:
        if task_id not in self._tasks or task_id in self._closed_tasks:
            raise ProtocolError("only an open framework task can be cancelled")
        self._send(
            "task_cancelled", agent_id=identifier(agent_id), task_id=identifier(task_id)
        )
        self._close_task(task_id)

    def operational_message(
        self,
        agent_id: str,
        target_agent_id: str,
        message: str,
        *,
        reason_code: str,
        task_id: str | None = None,
    ) -> None:
        _bounded_text(message, "operational message", 8_192)
        _bounded_text(reason_code, "operational reason", 256)
        if task_id is not None and task_id not in self._tasks:
            raise ProtocolError("operational message references an unknown task")
        self._send(
            "operational_message",
            agent_id=identifier(agent_id),
            target_agent_id=identifier(target_agent_id),
            task_id=identifier(task_id) if task_id is not None else None,
            reason_code=reason_code,
            message=message,
        )

    def managed_tool(
        self,
        *,
        agent_id: str,
        task_id: str,
        action_id: str,
        tool: str,
        purpose: str,
        arguments: Mapping[str, Any],
    ) -> Mapping[str, Any]:
        """Invoke a scored tool through HuntEval and return its untrusted result."""
        self._require_started(task_id)
        if action_id in self._actions:
            raise ProtocolError("framework action identity is duplicated")
        _bounded_text(purpose, "tool purpose", 4_096)
        if not isinstance(arguments, Mapping) or len(arguments) > MAX_OBJECT_ITEMS:
            raise ProtocolError("managed tool arguments are invalid or oversized")
        message_id = self._send(
            "tool_request",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            action_id=identifier(action_id),
            tool=identifier(tool),
            purpose=purpose,
            arguments=dict(arguments),
        )
        self._actions.add(action_id)
        response = self.peer.receive()
        if (
            response.get("type") != "tool_result"
            or response.get("action_id") != action_id
            or response.get("caused_by_message_id") != message_id
        ):
            raise ProtocolError("managed tool response is missing or has wrong correlation")
        return response

    def share_evidence(
        self, agent_id: str, task_id: str, evidence: Mapping[str, Any]
    ) -> None:
        self._require_open(task_id)
        evidence_id = identifier(evidence.get("id"))
        if evidence_id in self._evidence or len(evidence) > MAX_OBJECT_ITEMS:
            raise ProtocolError("framework evidence is duplicated or oversized")
        self._send(
            "evidence_shared",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            evidence=dict(evidence),
        )
        self._evidence.add(evidence_id)

    def propose_finding(
        self, agent_id: str, task_id: str, finding: Mapping[str, Any]
    ) -> None:
        self._require_open(task_id)
        finding_id = identifier(finding.get("id"))
        if finding_id in self._findings or len(finding) > MAX_OBJECT_ITEMS:
            raise ProtocolError("framework finding is duplicated or oversized")
        self._send(
            "finding_proposed",
            agent_id=identifier(agent_id),
            task_id=identifier(task_id),
            finding=dict(finding),
        )
        self._findings.add(finding_id)

    def review_finding(
        self, agent_id: str, finding_id: str, accepted: bool, reason_code: str
    ) -> None:
        if finding_id not in self._findings or not isinstance(accepted, bool):
            raise ProtocolError("finding review references an unknown finding")
        _bounded_text(reason_code, "finding review reason", 256)
        self._send(
            "finding_reviewed",
            agent_id=identifier(agent_id),
            finding_id=identifier(finding_id),
            accepted=accepted,
            reason_code=reason_code,
        )

    def _send(self, message_type: str, **payload: Any) -> str:
        message_id, timestamp = self.clock.next()
        self.peer.send(
            {
                "protocol_version": "0.3",
                "message_id": message_id,
                "run_id": self.clock.run_id,
                "timestamp": timestamp,
                "type": message_type,
                **payload,
            }
        )
        return message_id

    def _require_pending(self, task_id: str) -> None:
        if task_id not in self._tasks or task_id in self._started_tasks \
                or task_id in self._closed_tasks:
            raise ProtocolError("framework task must exist and be pending")

    def _require_started(self, task_id: str) -> None:
        if task_id not in self._started_tasks or task_id in self._closed_tasks:
            raise ProtocolError("framework task must be started and open")

    def _require_open(self, task_id: str) -> None:
        if task_id not in self._tasks or task_id in self._closed_tasks:
            raise ProtocolError("framework task must exist and be open")

    def _close_task(self, task_id: str) -> None:
        self._started_tasks.discard(task_id)
        self._closed_tasks.add(task_id)


def begin_framework_run(
    peer: DeploymentPeer, config: FrameworkAdapterConfig, framework: str
) -> FrameworkContext:
    """Perform the runner handshake and return a bounded framework context."""
    started = peer.receive()
    _validate_started(started)
    run_id = identifier(started.get("run_id"))
    clock = MessageClock(run_id, _parse_utc(started.get("timestamp")), config.message_prefix)
    message_id, timestamp = clock.next()
    peer.send(
        RegistrationMessage(
            message_id, run_id, timestamp, dict(config.deployment)
        ).to_dict()
    )
    accepted = peer.receive()
    if (
        accepted.get("type") != "registration_accepted"
        or accepted.get("caused_by_message_id") != message_id
    ):
        raise ProtocolError("framework deployment registration was not accepted")
    return FrameworkContext(peer, clock, started, identifier(framework))


def finish_framework_run(
    context: FrameworkContext, coordinator_agent_id: str, output: Any
) -> None:
    """Validate a structured output and finish the runner handshake."""
    final_id, timestamp = context.clock.next()
    context.peer.send(
        FinalSubmissionMessage(
            final_id,
            context.clock.run_id,
            timestamp,
            coordinator_agent_id,
            normalize_submission(output),
        ).to_dict()
    )
    terminated = context.peer.receive()
    if (
        terminated.get("type") != "run_terminated"
        or terminated.get("caused_by_message_id") != final_id
    ):
        raise ProtocolError("runner did not terminate the framework deployment cleanly")


def normalize_submission(output: Any) -> dict[str, Any]:
    """Normalize only an explicit structured final submission."""
    value = output
    if hasattr(value, "pydantic") and value.pydantic is not None:
        value = value.pydantic
    if hasattr(value, "model_dump"):
        value = value.model_dump(mode="json")
    if not isinstance(value, Mapping):
        raise ProtocolError("framework output must be a structured final submission")
    required = {
        "status",
        "summary",
        "finding_ids",
        "malicious_event_ids",
        "malicious_entity_ids",
        "attack_path",
        "attack_techniques",
        "confidence",
        "limitations",
    }
    if set(value) - (required | {"timeline"}) or not required.issubset(value):
        raise ProtocolError("framework final submission has missing or unknown fields")
    return dict(value)


def resolve_framework_output(output: Any) -> Any:
    """Resolve a framework coroutine without nesting an active event loop."""
    if not inspect.isawaitable(output):
        return output
    try:
        asyncio.get_running_loop()
    except RuntimeError:
        return asyncio.run(_await_output(cast(Awaitable[Any], output)))
    if inspect.iscoroutine(output):
        output.close()
    raise ProtocolError("async framework adapter cannot run inside an active event loop")


async def _await_output(output: Awaitable[Any]) -> Any:
    return await output


def _bounded_text(value: Any, name: str, maximum: int = MAX_TEXT_BYTES) -> str:
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > maximum:
        raise ProtocolError(f"{name} is empty or oversized")
    return value


def _parse_utc(value: Any) -> datetime:
    if not isinstance(value, str) or not value.endswith("Z"):
        raise ProtocolError("runner timestamp is not UTC RFC 3339")
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise ProtocolError("runner timestamp is not UTC RFC 3339") from error


def _validate_started(value: Mapping[str, Any]) -> None:
    minimum = value.get("supported_minimum")
    maximum = value.get("supported_maximum")
    valid = (
        value.get("type") == "run_started"
        and minimum == "0.3"
        and maximum == "0.3"
        and isinstance(value.get("objective"), str)
        and 0 < len(value["objective"].encode("utf-8")) <= MAX_TEXT_BYTES
        and isinstance(value.get("tables"), list)
        and all(isinstance(table, str) for table in value["tables"])
        and isinstance(value.get("limits"), Mapping)
        and isinstance(value.get("seed"), int)
        and not isinstance(value.get("seed"), bool)
    )
    if not valid:
        raise ProtocolError("runner does not offer a valid supported framework run")
