"""Optional Google ADK deployment connector without a hard dependency."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from typing import Any, Callable, Protocol

from .framework import (
    FrameworkAdapterConfig,
    FrameworkContext,
    begin_framework_run,
    finish_framework_run,
    resolve_framework_output,
)
from .protocol import DeploymentPeer

_MAX_EVENTS = 10_000


class GoogleAdkRunnerLike(Protocol):
    """Public structural subset of ``google.adk.runners.Runner``."""

    def run(
        self,
        *,
        user_id: str,
        session_id: str,
        new_message: Any,
        state_delta: Mapping[str, Any] | None = None,
        run_config: Any | None = None,
    ) -> Iterable[Any]: ...


GoogleAdkFactory = Callable[[FrameworkContext], GoogleAdkRunnerLike]
GoogleAdkContentFactory = Callable[[str], Any]
GoogleAdkResultMapper = Callable[[FrameworkContext, Sequence[Any]], Mapping[str, Any]]


@dataclass(frozen=True, slots=True)
class GoogleAdkAdapter:
    """Run the documented local ADK Runner surface; remote A2A stays disabled."""

    config: FrameworkAdapterConfig
    runner_factory: GoogleAdkFactory
    content_factory: GoogleAdkContentFactory
    result_mapper: GoogleAdkResultMapper

    def run(self, peer: DeploymentPeer) -> None:
        context = begin_framework_run(peer, self.config, "google_adk")
        objective = str(context.kickoff_inputs["objective"])
        stream = self.runner_factory(context).run(
            user_id=self.config.coordinator_agent_id,
            session_id=context.clock.run_id,
            new_message=self.content_factory(objective),
            state_delta=None,
            run_config=None,
        )
        events = _bounded_events(stream)
        submission = self.result_mapper(context, events)
        finish_framework_run(
            context,
            self.config.coordinator_agent_id,
            resolve_framework_output(submission),
        )


def _bounded_events(stream: Iterable[Any]) -> tuple[Any, ...]:
    events: list[Any] = []
    for event in stream:
        if len(events) == _MAX_EVENTS:
            raise ValueError("Google ADK event stream exceeds its bound")
        events.append(event)
    return tuple(events)
