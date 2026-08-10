"""Safe CrewAI deployment adapter for the HuntEval process protocol."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from typing import Any, Callable, Mapping, Protocol

from ._validation import identifier
from .protocol import DeploymentPeer, FinalSubmissionMessage, ProtocolError, RegistrationMessage


class CrewLike(Protocol):
    """Structural boundary implemented by ``crewai.Crew``."""

    def kickoff(self, *, inputs: Mapping[str, Any]) -> Any: ...


CrewFactory = Callable[["CrewAIContext"], CrewLike]


@dataclass(frozen=True, slots=True)
class CrewAIAdapterConfig:
    """Content-addressable deployment metadata sent to HuntEval."""

    deployment: Mapping[str, Any]
    coordinator_agent_id: str

    def __post_init__(self) -> None:
        identifier(self.coordinator_agent_id)
        if not isinstance(self.deployment, Mapping) or not self.deployment.get("agents"):
            raise ValueError("CrewAI deployment metadata must declare agents")
        agents = self.deployment["agents"]
        if not isinstance(agents, list) or not any(
            isinstance(agent, Mapping) and agent.get("id") == self.coordinator_agent_id
            for agent in agents
        ):
            raise ValueError("CrewAI coordinator must be a declared deployment agent")


@dataclass(slots=True)
class _MessageClock:
    run_id: str
    initial: datetime
    sequence: int = 0

    def next(self) -> tuple[str, str]:
        self.sequence += 1
        timestamp = self.initial + timedelta(microseconds=self.sequence)
        value = timestamp.astimezone(timezone.utc).isoformat(timespec="microseconds")
        return f"crewai-{self.sequence:06d}", value.replace("+00:00", "Z")


@dataclass(slots=True)
class CrewAIContext:
    """Runner-mediated services available while a CrewAI crew is executing."""

    peer: DeploymentPeer
    clock: _MessageClock
    run_started: Mapping[str, Any]
    _tasks: set[str] = field(default_factory=set)
    _started_tasks: set[str] = field(default_factory=set)
    _actions: set[str] = field(default_factory=set)

    @property
    def kickoff_inputs(self) -> dict[str, Any]:
        return {
            "run_id": self.clock.run_id,
            "objective": self.run_started["objective"],
            "tables": list(self.run_started.get("tables", [])),
            "seed": self.run_started["seed"],
            "limits": dict(self.run_started["limits"]),
        }

    def create_task(self, agent_id: str, task_id: str, objective: str) -> None:
        if task_id in self._tasks or not objective or len(objective.encode()) > 4_096:
            raise ProtocolError("CrewAI task is duplicated, empty, or oversized")
        self._send("task_created", agent_id=agent_id, task={
            "id": identifier(task_id), "objective": objective, "priority": "normal",
            "dependencies": [], "required_capabilities": [], "parent_task_id": None,
        })
        self._tasks.add(task_id)

    def start_task(self, agent_id: str, task_id: str) -> None:
        if task_id not in self._tasks or task_id in self._started_tasks:
            raise ProtocolError("CrewAI task must exist and may start only once")
        self._send("task_started", agent_id=agent_id, task_id=task_id)
        self._started_tasks.add(task_id)

    def delegate_task(self, agent_id: str, task_id: str, target_agent_id: str) -> None:
        if task_id not in self._tasks or task_id in self._started_tasks:
            raise ProtocolError("CrewAI task must exist and be pending before delegation")
        self._send(
            "task_delegated", agent_id=agent_id, task_id=task_id,
            target_agent_id=target_agent_id, reason_code="crewai_assignment",
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
        if task_id not in self._started_tasks or action_id in self._actions:
            raise ProtocolError("managed tool requires a started task and a new action")
        message_id = self._send(
            "tool_request", agent_id=agent_id, task_id=task_id,
            action_id=identifier(action_id), tool=identifier(tool), purpose=purpose,
            arguments=dict(arguments),
        )
        self._actions.add(action_id)
        response = self.peer.receive()
        if (response.get("type") != "tool_result"
                or response.get("action_id") != action_id
                or response.get("caused_by_message_id") != message_id):
            raise ProtocolError("managed tool response is missing or has wrong correlation")
        return response

    def complete_task(self, agent_id: str, task_id: str) -> None:
        if task_id not in self._started_tasks:
            raise ProtocolError("only a started CrewAI task can complete")
        self._send("task_completed", agent_id=agent_id, task_id=task_id)
        self._started_tasks.remove(task_id)

    def _send(self, message_type: str, **payload: Any) -> str:
        message_id, timestamp = self.clock.next()
        message = {
            "protocol_version": "0.3", "message_id": message_id,
            "run_id": self.clock.run_id, "timestamp": timestamp,
            "type": message_type, **payload,
        }
        self.peer.send(message)
        return message_id


@dataclass(frozen=True, slots=True)
class CrewAIAdapter:
    """Run a CrewAI crew as a bounded HuntEval deployment process."""

    config: CrewAIAdapterConfig
    crew_factory: CrewFactory

    def run(self, peer: DeploymentPeer) -> None:
        started = peer.receive()
        _validate_started(started)
        run_id = identifier(started.get("run_id"))
        initial = _parse_utc(started.get("timestamp"))
        clock = _MessageClock(run_id, initial)
        message_id, timestamp = clock.next()
        peer.send(RegistrationMessage(
            message_id, run_id, timestamp, dict(self.config.deployment)
        ).to_dict())
        accepted = peer.receive()
        if accepted.get("type") != "registration_accepted" \
                or accepted.get("caused_by_message_id") != message_id:
            raise ProtocolError("CrewAI deployment registration was not accepted")
        context = CrewAIContext(peer, clock, started)
        raw = self.crew_factory(context).kickoff(inputs=context.kickoff_inputs)
        submission = _submission(raw)
        final_id, final_timestamp = clock.next()
        peer.send(FinalSubmissionMessage(
            final_id, run_id, final_timestamp, self.config.coordinator_agent_id, submission
        ).to_dict())
        terminated = peer.receive()
        if terminated.get("type") != "run_terminated" \
                or terminated.get("caused_by_message_id") != final_id:
            raise ProtocolError("runner did not terminate the CrewAI deployment cleanly")


def _submission(output: Any) -> dict[str, Any]:
    value = output
    if hasattr(value, "pydantic") and value.pydantic is not None:
        value = value.pydantic
    if hasattr(value, "model_dump"):
        value = value.model_dump(mode="json")
    if not isinstance(value, Mapping):
        raise ProtocolError("CrewAI output must be a structured final submission")
    required = {"status", "summary", "finding_ids", "malicious_event_ids",
                "malicious_entity_ids", "attack_path", "attack_techniques",
                "confidence", "limitations"}
    if set(value) - (required | {"timeline"}) or not required.issubset(value):
        raise ProtocolError("CrewAI final submission has missing or unknown fields")
    return dict(value)


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
        and isinstance(minimum, str) and isinstance(maximum, str)
        and minimum == "0.3" and maximum == "0.3"
        and isinstance(value.get("objective"), str)
        and 0 < len(value["objective"].encode()) <= 65_536
        and isinstance(value.get("tables"), list)
        and all(isinstance(table, str) for table in value["tables"])
        and isinstance(value.get("limits"), Mapping)
        and isinstance(value.get("seed"), int)
    )
    if not valid:
        raise ProtocolError("runner does not offer a valid supported CrewAI run")
