#!/usr/bin/env python3
"""Verify one isolated upstream framework package against the adapter surface."""

from __future__ import annotations

import argparse
import importlib.metadata
import inspect
from collections.abc import Callable


def require_parameters(callable_value: Callable[..., object], names: set[str]) -> None:
    actual = set(inspect.signature(callable_value).parameters)
    missing = sorted(names - actual)
    if missing:
        raise SystemExit(f"upstream callable is missing parameters: {missing}")


def crewai() -> None:
    from crewai import Crew

    require_parameters(Crew.kickoff, {"inputs"})


def langgraph() -> None:
    from langgraph.graph import StateGraph

    require_parameters(StateGraph.compile, {"checkpointer", "interrupt_after", "interrupt_before"})


def autogen() -> None:
    from autogen_agentchat.teams import BaseGroupChat

    require_parameters(BaseGroupChat.run, {"task", "cancellation_token"})


def google_adk() -> None:
    from google.adk.runners import Runner

    require_parameters(
        Runner.run,
        {"user_id", "session_id", "new_message", "state_delta", "run_config"},
    )


def semantic_kernel() -> None:
    from semantic_kernel.agents import SequentialOrchestration

    require_parameters(SequentialOrchestration.invoke, {"task", "runtime"})


CHECKS: dict[str, tuple[str, str, Callable[[], None]]] = {
    "autogen": ("autogen-agentchat", "0.7.5", autogen),
    "crewai": ("crewai", "1.15.5", crewai),
    "google-adk": ("google-adk", "2.6.3", google_adk),
    "langgraph": ("langgraph", "1.2.10", langgraph),
    "semantic-kernel": ("semantic-kernel", "1.44.1", semantic_kernel),
}


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("framework", choices=sorted(CHECKS))
    arguments = parser.parse_args()
    distribution, expected, check = CHECKS[arguments.framework]
    actual = importlib.metadata.version(distribution)
    if actual != expected:
        raise SystemExit(f"expected {distribution} {expected}, found {actual}")
    check()
    print(f"{arguments.framework} {actual}: public adapter surface available")


if __name__ == "__main__":
    main()
