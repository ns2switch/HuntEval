"""Optional Google ADK deployment connector without a hard dependency."""

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


class GoogleAdkRunnerLike(Protocol):
    """Structural boundary for a local Google ADK runner wrapper."""

    def run(self, *, inputs: Mapping[str, Any]) -> Any: ...


GoogleAdkFactory = Callable[[FrameworkContext], GoogleAdkRunnerLike]


@dataclass(frozen=True, slots=True)
class GoogleAdkAdapter:
    """Run a local ADK deployment; remote A2A remains disabled by policy."""

    config: FrameworkAdapterConfig
    runner_factory: GoogleAdkFactory

    def run(self, peer: DeploymentPeer) -> None:
        context = begin_framework_run(peer, self.config, "google_adk")
        raw = self.runner_factory(context).run(inputs=context.kickoff_inputs)
        finish_framework_run(
            context, self.config.coordinator_agent_id, resolve_framework_output(raw)
        )
