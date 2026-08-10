"""Optional LangGraph deployment connector without a hard dependency."""

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


class LangGraphLike(Protocol):
    """Structural subset implemented by a compiled LangGraph graph."""

    def invoke(self, input: Mapping[str, Any], config: Mapping[str, Any]) -> Any: ...


LangGraphFactory = Callable[[FrameworkContext], LangGraphLike]


@dataclass(frozen=True, slots=True)
class LangGraphAdapter:
    """Run a compiled graph while HuntEval retains tool and protocol authority."""

    config: FrameworkAdapterConfig
    graph_factory: LangGraphFactory

    def run(self, peer: DeploymentPeer) -> None:
        context = begin_framework_run(peer, self.config, "langgraph")
        graph = self.graph_factory(context)
        raw = graph.invoke(
            context.kickoff_inputs,
            {"configurable": {"hunteval_run_id": context.clock.run_id}},
        )
        finish_framework_run(
            context, self.config.coordinator_agent_id, resolve_framework_output(raw)
        )
