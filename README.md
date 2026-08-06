# HuntEval

HuntEval is an open-source framework for reproducibly evaluating multi-agent systems applied to threat-hunting scenarios in cloud environments.

The evaluated unit is a complete **deployment**, not an isolated model. A deployment may include one or more agents, prompts, models, tools, memory layers, coordination policies, and runtime configuration. HuntEval measures which implementation performs best and explains the trade-offs across investigation quality, evidence quality, coordination, resilience, efficiency, and reproducibility.

A future diagnostic layer will use observable run traces to identify systematic failures and propose concrete prompt improvements. Those recommendations must be validated through controlled A/B experiments before they are considered effective.

## MVP scope

- Rust core.
- CLI-first interface.
- DuckDB and Parquet as the canonical local analytics environment.
- Initial scenarios for AWS, Microsoft Azure, and Google Cloud.
- Evaluation of single-agent and multi-agent deployments.
- HuntEval-managed tool execution during scored runs.
- Ground truth hidden from the evaluated deployment.
- Structured recording of agents, tasks, messages, tool calls, evidence, hypotheses, and findings.
- Configurable scoring profiles and statistical deployment comparison.
- Optional RAG for knowledge supplied by the hunt author and, later, for querying HuntEval-generated reports.

## Explicitly out of scope for the initial release

- Self-RAG as the primary object of evaluation.
- Collection of private chain of thought.
- Direct execution against production SIEM platforms.
- Fully autonomous prompt optimization without experimental validation.
- A fixed universal score embedded in the codebase.
- A web dashboard, Kubernetes deployment, or distributed control plane.

## Documentation map

- `docs/SPECIFICATION.md`: functional and technical specification.
- `docs/ADR.md`: architecture decision records.
- `docs/CONTRACTS.md`: domain contracts and JSONL process protocol.
- `docs/METRICS_AND_RANKING.md`: metrics, scoring profiles, statistics, and ranking.
- `docs/PROMPT_OPTIMIZATION.md`: failure diagnosis and prompt improvement workflow.
- `docs/THREAT_MODEL.md`: threats against the framework and evaluated deployments.
- `docs/IMPLEMENTATION_PLAN.md`: milestones and acceptance criteria.
- `docs/EXECUTION_PLAN.md`: executable pull-request sequence, contracts, tests, and quality gates.
- `docs/ROADMAP.md`: evolution from the MVP to assisted optimization.
- `AGENTS.md`: permanent development-agent instructions.

## Short definition

> HuntEval is an open-source framework for evaluating, comparing, diagnosing, and improving multi-agent threat-hunting deployments against reproducible cloud-security scenarios.

## Project principles

1. **Evidence over narrative.** Findings must be traceable to observable telemetry and tool results.
2. **Deployment-level evaluation.** Architecture and coordination are part of the system being tested.
3. **Framework neutrality.** HuntEval must not require a particular agent SDK or LLM provider.
4. **Reproducibility.** Datasets, prompts, configurations, binaries, and random seeds are versioned or hashed.
5. **Safe evaluation.** Scored tools are mediated by HuntEval and ground truth is isolated.
6. **Transparent ranking.** Metric vectors remain available even when an aggregate score is calculated.
7. **Validated improvement.** Prompt recommendations are hypotheses until confirmed by controlled experiments.
