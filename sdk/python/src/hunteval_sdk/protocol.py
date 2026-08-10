from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import datetime
from typing import IO, Any

from ._validation import ContractError, identifier

MAX_LINE_BYTES = 1_048_576
MAX_MESSAGES = 4_096
DEPLOYMENT_MESSAGES = {
    "register_deployment", "task_created", "task_delegated", "task_started",
    "task_completed", "task_failed", "task_reassigned", "task_cancelled",
    "operational_message", "hypothesis_updated", "tool_request", "evidence_shared",
    "finding_proposed", "finding_reviewed", "final_submission",
}
RUNNER_MESSAGES = {
    "run_started", "registration_accepted", "tool_result", "protocol_error", "run_terminated"
}


class ProtocolError(ValueError):
    """A bounded deployment-peer protocol operation failed."""


@dataclass(frozen=True, slots=True)
class RegistrationMessage:
    message_id: str
    run_id: str
    timestamp: str
    deployment: dict[str, Any]
    caused_by_message_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return _envelope(
            self.message_id,
            self.run_id,
            self.timestamp,
            self.caused_by_message_id,
            {
                "type": "register_deployment",
                "selected_protocol_version": "0.3",
                "deployment": self.deployment,
            },
        )


@dataclass(frozen=True, slots=True)
class FinalSubmissionMessage:
    message_id: str
    run_id: str
    timestamp: str
    agent_id: str
    submission: dict[str, Any]
    caused_by_message_id: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return _envelope(
            self.message_id,
            self.run_id,
            self.timestamp,
            self.caused_by_message_id,
            {
                "type": "final_submission",
                "agent_id": identifier(self.agent_id),
                "submission": self.submission,
            },
        )


@dataclass(slots=True)
class DeploymentPeer:
    input_stream: IO[str]
    output_stream: IO[str]
    registered: bool = False
    terminal: bool = False
    sent_messages: int = 0
    received_messages: int = 0
    message_ids: set[str] | None = None

    def __post_init__(self) -> None:
        if self.message_ids is None:
            self.message_ids = set()

    def receive(self) -> dict[str, Any]:
        line = self.input_stream.readline(MAX_LINE_BYTES + 1)
        if not line or len(line.encode("utf-8")) > MAX_LINE_BYTES or not line.endswith("\n"):
            raise ProtocolError("protocol input is missing, oversized, or unterminated")
        value = json.loads(line)
        if not isinstance(value, dict) or not isinstance(value.get("type"), str):
            raise ProtocolError("protocol message must be a typed object")
        if value["type"] not in RUNNER_MESSAGES:
            raise ProtocolError("deployment peer received a deployment-origin message")
        if self.received_messages >= MAX_MESSAGES:
            raise ProtocolError("protocol input exceeds the message limit")
        self._accept_identity(value)
        self.received_messages += 1
        return value

    def send(self, message: dict[str, Any]) -> None:
        if self.terminal:
            raise ProtocolError("cannot send after terminal submission")
        message_type = message.get("type")
        if not isinstance(message_type, str):
            raise ProtocolError("protocol message type is required")
        if message_type not in DEPLOYMENT_MESSAGES:
            raise ProtocolError("deployment peer cannot send this message type")
        if message_type == "register_deployment":
            if self.registered:
                raise ProtocolError("deployment is already registered")
        elif not self.registered:
            raise ProtocolError("deployment must register before sending messages")
        if self.sent_messages >= MAX_MESSAGES:
            raise ProtocolError("protocol output exceeds the message limit")
        encoded = json.dumps(message, separators=(",", ":"), ensure_ascii=False)
        if len(encoded.encode("utf-8")) + 1 > MAX_LINE_BYTES:
            raise ProtocolError("protocol output exceeds the byte limit")
        self._accept_identity(message)
        if message_type == "register_deployment":
            self.registered = True
        if message_type == "final_submission":
            self.terminal = True
        self.sent_messages += 1
        self.output_stream.write(encoded + "\n")
        self.output_stream.flush()

    def _accept_identity(self, message: dict[str, Any]) -> None:
        try:
            message_id = identifier(message.get("message_id"))
            identifier(message.get("run_id"))
        except ContractError as error:
            raise ProtocolError("protocol envelope identity is invalid") from error
        if message.get("protocol_version") != "0.3" or not _utc(message.get("timestamp")):
            raise ProtocolError("protocol envelope version or timestamp is invalid")
        if self.message_ids is None or message_id in self.message_ids:
            raise ProtocolError("protocol message identity is duplicated")
        self.message_ids.add(message_id)


def _envelope(
    message_id: str,
    run_id: str,
    timestamp: str,
    caused_by_message_id: str | None,
    payload: dict[str, Any],
) -> dict[str, Any]:
    value: dict[str, Any] = {
        "protocol_version": "0.3",
        "message_id": identifier(message_id),
        "run_id": identifier(run_id),
        "timestamp": timestamp,
        **payload,
    }
    if not _utc(timestamp):
        raise ProtocolError("protocol timestamp must be valid UTC RFC 3339")
    if caused_by_message_id is not None:
        value["caused_by_message_id"] = identifier(caused_by_message_id)
    return value


def _utc(value: Any) -> bool:
    if not isinstance(value, str) or not value.endswith("Z"):
        return False
    try:
        datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return False
    return True
