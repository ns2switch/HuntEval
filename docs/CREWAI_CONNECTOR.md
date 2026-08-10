# CrewAI connector

## Purpose

The HuntEval CrewAI connector runs a CrewAI `Crew` as an external deployment adapter speaking the HuntEval JSONL protocol. CrewAI keeps responsibility for agent collaboration and task orchestration. HuntEval remains responsible for scored-tool execution, budgets, evidence provenance, protocol validation, and benchmark isolation.

The connector uses CrewAI's public `kickoff(inputs=...)` boundary and structural typing. HuntEval does not depend on CrewAI, an LLM provider, CrewAI AMP, or a particular CrewAI process topology.

## Installation

Use Python 3.11 or newer for the HuntEval SDK. Install CrewAI in the deployment's isolated environment following its supported Python range:

```bash
uv venv
uv pip install ./sdk/python
uv pip install crewai
export OTEL_SDK_DISABLED=true
```

Disabling third-party telemetry is recommended for benchmark deployments. Provider credentials belong in the deployment environment and must never be committed to HuntEval manifests or artifacts.

## Adapter entry point

A deployment executable creates its agents and tasks in a factory. The factory receives a `CrewAIContext`, which exposes public episode inputs and runner-mediated managed tools:

```python
import sys

from crewai import Agent, Crew, Process, Task
from crewai.tools import tool
from hunteval_sdk import CrewAIAdapter, CrewAIAdapterConfig, DeploymentPeer
from pydantic import BaseModel


class HuntEvalSubmission(BaseModel):
    status: str
    summary: str
    finding_ids: list[str]
    malicious_event_ids: list[str]
    malicious_entity_ids: list[str]
    attack_path: list[str]
    attack_techniques: list[str]
    confidence: float
    limitations: list[str]


DEPLOYMENT = {
    "id": "crewai-threat-hunt",
    "architecture": "single_agent",
    "version": "0.1.0",
    "agents": [{
        "id": "investigator",
        "role": "investigator",
        "capabilities": ["sql_query", "synthesis"],
        "prompt_version": "1.0.0",
        "prompt_sha256": "<sha256>",
        "model": "deployment-configured",
        "model_parameters": {},
    }],
}


def build_crew(context):
    context.create_task("investigator", "hunt-task", context.kickoff_inputs["objective"])
    context.start_task("investigator", "hunt-task")

    @tool("hunteval_sql")
    def hunteval_sql(query: str) -> dict:
        """Execute bounded scored SQL through HuntEval."""
        result = context.managed_tool(
            agent_id="investigator",
            task_id="hunt-task",
            action_id="sql-action",
            tool="duckdb_sql",
            purpose="Inspect public benchmark telemetry",
            arguments={"query": query},
        )
        context.complete_task("investigator", "hunt-task")
        return dict(result)

    investigator = Agent(
        role="Threat-hunt investigator",
        goal="Investigate the supplied objective using observable evidence",
        backstory="Return only evidence-backed investigative results.",
        tools=[hunteval_sql],
    )
    task = Task(
        description="Investigate {objective} in the declared tables.",
        expected_output="A structured HuntEval final submission.",
        agent=investigator,
        output_pydantic=HuntEvalSubmission,
    )
    return Crew(agents=[investigator], tasks=[task], process=Process.sequential)


adapter = CrewAIAdapter(
    CrewAIAdapterConfig(DEPLOYMENT, coordinator_agent_id="investigator"),
    build_crew,
)
adapter.run(DeploymentPeer(sys.stdin, sys.stdout))
```

The final CrewAI task must use structured output whose fields match HuntEval's `final_submission` contract. Plain text, missing or unknown fields, invalid tool correlation, duplicate tasks/actions, and protocol downgrade attempts fail closed.

Allocate a unique action identifier for every tool call and complete every started task with `context.complete_task(...)`. The example uses one deterministic call to keep the trust boundary visible.

## Security boundary

- CrewAI agents receive only the public objective, tables, seed, limits, and run identity.
- Ground truth and hidden-test information are never passed to the crew.
- Scored tools are invoked only through `CrewAIContext.managed_tool`.
- Tool responses remain untrusted input and retain runner correlation identifiers.
- The connector grants no filesystem or network capability.
- The executable must still pass the R7 manifest, digest, capability-policy, sandbox, and conformance checks.
- CrewAI internal traces and private reasoning are not collected. Only explicit HuntEval protocol events are normative.

## Limitations

The connector does not configure model providers, install CrewAI, expose production SIEM connectivity, translate arbitrary CrewAI traces into HuntEval evidence, or grant direct tools to agents. CrewAI Flows can use the same boundary when the factory result provides `kickoff(inputs=...)` and returns the required structured result.
