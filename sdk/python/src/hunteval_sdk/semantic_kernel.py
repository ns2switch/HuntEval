"""Preview Semantic Kernel connector over its observable public wrapper API."""

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


class SemanticKernelLike(Protocol):
    """Structural boundary for a supported Semantic Kernel orchestration wrapper."""

    def invoke(self, *, inputs: Mapping[str, Any]) -> Any: ...


SemanticKernelFactory = Callable[[FrameworkContext], SemanticKernelLike]


@dataclass(frozen=True, slots=True)
class SemanticKernelPreviewAdapter:
    """Run an explicitly preview-labeled Semantic Kernel orchestration."""

    config: FrameworkAdapterConfig
    orchestration_factory: SemanticKernelFactory

    support_status: str = "preview"

    def run(self, peer: DeploymentPeer) -> None:
        if self.support_status != "preview":
            raise ValueError("Semantic Kernel connector must remain preview")
        context = begin_framework_run(peer, self.config, "semantic_kernel")
        raw = self.orchestration_factory(context).invoke(inputs=context.kickoff_inputs)
        finish_framework_run(
            context, self.config.coordinator_agent_id, resolve_framework_output(raw)
        )
