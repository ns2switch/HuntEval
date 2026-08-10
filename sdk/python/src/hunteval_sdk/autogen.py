"""Optional AutoGen AgentChat deployment connector without a hard dependency."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Mapping, Protocol

from .framework import (
    FrameworkAdapterConfig,
    FrameworkContext,
    begin_framework_run,
    finish_framework_run,
    resolve_framework_output,
)
from .protocol import DeploymentPeer


class AutoGenTeamLike(Protocol):
    """Structural subset of an AutoGen AgentChat team."""

    def run(self, *, task: str) -> Any: ...


AutoGenTeamFactory = Callable[[FrameworkContext, Mapping[str, Any]], AutoGenTeamLike]


@dataclass(frozen=True, slots=True)
class AutoGenAdapter:
    """Run an AgentChat team with HuntEval services captured by its factory."""

    config: FrameworkAdapterConfig
    team_factory: AutoGenTeamFactory

    def run(self, peer: DeploymentPeer) -> None:
        context = begin_framework_run(peer, self.config, "autogen")
        inputs = context.kickoff_inputs
        team = self.team_factory(context, inputs)
        raw = team.run(task=str(inputs["objective"]))
        finish_framework_run(
            context, self.config.coordinator_agent_id, resolve_framework_output(raw)
        )
