"""Safe CrewAI deployment adapter for the HuntEval process protocol."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Mapping, Protocol

from .framework import (
    FrameworkAdapterConfig,
    FrameworkContext,
    begin_framework_run,
    finish_framework_run,
)
from .protocol import DeploymentPeer


class CrewLike(Protocol):
    """Structural boundary implemented by ``crewai.Crew``."""

    def kickoff(self, *, inputs: Mapping[str, Any]) -> Any: ...


CrewAIContext = FrameworkContext
CrewFactory = Callable[[CrewAIContext], CrewLike]


@dataclass(frozen=True, slots=True)
class CrewAIAdapterConfig:
    """Content-addressable deployment metadata sent to HuntEval."""

    deployment: Mapping[str, Any]
    coordinator_agent_id: str

    def as_framework_config(self) -> FrameworkAdapterConfig:
        return FrameworkAdapterConfig(
            self.deployment, self.coordinator_agent_id, "crewai"
        )

    def __post_init__(self) -> None:
        self.as_framework_config()


@dataclass(frozen=True, slots=True)
class CrewAIAdapter:
    """Run a CrewAI crew as a bounded HuntEval deployment process."""

    config: CrewAIAdapterConfig
    crew_factory: CrewFactory

    def run(self, peer: DeploymentPeer) -> None:
        context = begin_framework_run(peer, self.config.as_framework_config(), "crewai")
        raw = self.crew_factory(context).kickoff(inputs=context.kickoff_inputs)
        finish_framework_run(context, self.config.coordinator_agent_id, raw)
