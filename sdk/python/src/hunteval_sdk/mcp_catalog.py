"""Finite MCP tool catalog exposed by HuntEval interoperability sessions."""

from __future__ import annotations

from typing import Any


def tool_catalog() -> list[dict[str, Any]]:
    """Return a fresh, immutable-by-caller description of supported MCP tools."""
    return [
        _tool(
            "hunteval.task.create",
            "Declare a bounded HuntEval task owned by a registered agent.",
            {
                "agent_id": _identifier(),
                "task_id": _identifier(),
                "objective": _text(1, 4096),
            },
            ["agent_id", "task_id", "objective"],
        ),
        _tool(
            "hunteval.task.delegate",
            "Delegate a pending HuntEval task to another registered agent.",
            {
                "agent_id": _identifier(),
                "task_id": _identifier(),
                "target_agent_id": _identifier(),
            },
            ["agent_id", "task_id", "target_agent_id"],
        ),
        _tool(
            "hunteval.task.start",
            "Mark a declared HuntEval task as started.",
            {"agent_id": _identifier(), "task_id": _identifier()},
            ["agent_id", "task_id"],
        ),
        _tool(
            "hunteval.task.complete",
            "Mark a started HuntEval task as complete.",
            {"agent_id": _identifier(), "task_id": _identifier()},
            ["agent_id", "task_id"],
        ),
        _tool(
            "hunteval.tool.call",
            "Invoke one runner-authorized managed tool for a started task.",
            {
                "agent_id": _identifier(),
                "task_id": _identifier(),
                "action_id": _identifier(),
                "tool": _identifier(),
                "purpose": _text(1, 4096),
                "arguments": {"type": "object", "maxProperties": 4096},
            },
            ["agent_id", "task_id", "action_id", "tool", "purpose", "arguments"],
        ),
        _tool(
            "hunteval.evidence.share",
            "Share structured observable evidence for an open task.",
            {
                "agent_id": _identifier(),
                "task_id": _identifier(),
                "evidence": {"type": "object", "maxProperties": 4096},
            },
            ["agent_id", "task_id", "evidence"],
        ),
        _tool(
            "hunteval.finding.propose",
            "Propose a structured finding grounded in declared evidence.",
            {
                "agent_id": _identifier(),
                "task_id": _identifier(),
                "finding": {"type": "object", "maxProperties": 4096},
            },
            ["agent_id", "task_id", "finding"],
        ),
        _tool(
            "hunteval.submission.set",
            "Set the structured final submission returned to the HuntEval adapter.",
            {"submission": {"type": "object", "maxProperties": 11}},
            ["submission"],
        ),
    ]


def _tool(
    name: str,
    description: str,
    properties: dict[str, Any],
    required: list[str],
) -> dict[str, Any]:
    return {
        "name": name,
        "description": description,
        "inputSchema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": False,
        },
    }


def _identifier() -> dict[str, Any]:
    return {
        "type": "string",
        "pattern": "^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$",
        "maxLength": 128,
    }


def _text(minimum: int, maximum: int) -> dict[str, Any]:
    return {"type": "string", "minLength": minimum, "maxLength": maximum}
