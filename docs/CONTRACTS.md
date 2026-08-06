# Domain contracts and process protocol

## 1. Versioning rules

Every protocol message includes:

- `protocol_version`;
- `message_id`;
- `run_id`;
- `timestamp`;
- `type`.

Backward-incompatible changes increment the major protocol version. New optional fields may increment the minor version. Unknown optional fields must be ignored; unknown message types must produce a structured protocol error.

## 2. Common envelope

```json
{
  "protocol_version": "0.3",
  "message_id": "msg-000001",
  "run_id": "run-000001",
  "timestamp": "2026-08-06T18:00:00Z",
  "type": "message_type"
}
```

## 3. Deployment registration

```json
{
  "protocol_version": "0.3",
  "type": "register_deployment",
  "message_id": "msg-001",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:00Z",
  "deployment": {
    "id": "supervisor-specialists-v1",
    "architecture": "hierarchical",
    "version": "1.0.0",
    "agents": [
      {
        "id": "supervisor",
        "role": "orchestrator",
        "capabilities": ["planning", "delegation", "synthesis"],
        "prompt_version": "1.0.0",
        "prompt_sha256": "...",
        "model": "provider/model",
        "model_parameters": {
          "temperature": 0.1
        }
      },
      {
        "id": "identity-specialist",
        "role": "investigator",
        "capabilities": ["iam_analysis", "entity_pivoting"],
        "prompt_version": "1.0.0",
        "prompt_sha256": "...",
        "model": "provider/model"
      }
    ]
  }
}
```

Registration fails when IDs are duplicated, required capabilities are invalid, the protocol is unsupported, or the deployment exceeds episode limits.

## 4. Coordination event types

Minimum types:

- `agent_registered`;
- `task_created`;
- `task_delegated`;
- `task_started`;
- `task_completed`;
- `task_failed`;
- `task_reassigned`;
- `task_cancelled`;
- `message_sent`;
- `hypothesis_created`;
- `hypothesis_updated`;
- `hypothesis_rejected`;
- `evidence_shared`;
- `finding_proposed`;
- `finding_challenged`;
- `finding_accepted`;
- `finding_rejected`;
- `conflict_detected`;
- `conflict_resolved`;
- `final_submission`.

## 5. Task creation

```json
{
  "protocol_version": "0.3",
  "type": "task_created",
  "message_id": "msg-010",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:02Z",
  "agent_id": "supervisor",
  "task": {
    "id": "task-019",
    "objective": "Determine how the suspected principal obtained administrative privileges.",
    "priority": "high",
    "dependencies": [],
    "required_capabilities": ["iam_analysis"],
    "parent_task_id": null
  }
}
```

## 6. Task delegation

```json
{
  "protocol_version": "0.3",
  "type": "task_delegated",
  "message_id": "msg-011",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:03Z",
  "agent_id": "supervisor",
  "task_id": "task-019",
  "target_agent_id": "identity-specialist",
  "reason_code": "capability_match"
}
```

## 7. Inter-agent message

```json
{
  "protocol_version": "0.3",
  "type": "message_sent",
  "message_id": "msg-020",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:08Z",
  "source_agent_id": "identity-specialist",
  "target_agent_id": "supervisor",
  "task_id": "task-019",
  "purpose": "Share a suspicious role-assumption pivot.",
  "body": "The role was assumed shortly after a policy attachment by the same principal.",
  "references": {
    "action_ids": ["action-074"],
    "evidence_ids": ["evidence-031"]
  }
}
```

The body is operational communication, not private chain of thought.

## 8. Tool request

```json
{
  "protocol_version": "0.3",
  "type": "tool_request",
  "message_id": "msg-030",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:10Z",
  "agent_id": "query-specialist",
  "task_id": "task-019",
  "action_id": "action-074",
  "tool": "duckdb_sql",
  "purpose": "Find role assumptions performed by the suspected principal.",
  "arguments": {
    "query": "SELECT event_id, event_time, principal, resource FROM aws_cloudtrail WHERE event_name = 'AssumeRole' AND principal = ? ORDER BY event_time",
    "parameters": ["arn:aws:iam::123456789012:user/suspected"]
  }
}
```

## 9. Tool result

Tool results are created by HuntEval, not the deployment.

```json
{
  "protocol_version": "0.3",
  "type": "tool_result",
  "message_id": "msg-031",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:11Z",
  "action_id": "action-074",
  "tool": "duckdb_sql",
  "status": "success",
  "schema": [
    {"name": "event_id", "type": "VARCHAR"},
    {"name": "event_time", "type": "TIMESTAMP"},
    {"name": "principal", "type": "VARCHAR"},
    {"name": "resource", "type": "VARCHAR"}
  ],
  "rows": [
    {
      "event_id": "evt-0019",
      "event_time": "2026-01-10T10:32:15Z",
      "principal": "arn:aws:iam::123456789012:user/suspected",
      "resource": "arn:aws:iam::987654321098:role/admin"
    }
  ],
  "row_count": 1,
  "truncated": false,
  "duration_ms": 18
}
```

## 10. Tool error

```json
{
  "protocol_version": "0.3",
  "type": "tool_result",
  "message_id": "msg-032",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:12Z",
  "action_id": "action-075",
  "tool": "duckdb_sql",
  "status": "error",
  "error": {
    "code": "sql_policy_violation",
    "message": "Only read-only SELECT statements are allowed.",
    "retryable": false
  }
}
```

## 11. Hypothesis

```json
{
  "protocol_version": "0.3",
  "type": "hypothesis_created",
  "message_id": "msg-040",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:15Z",
  "agent_id": "identity-specialist",
  "task_id": "task-019",
  "hypothesis": {
    "id": "hypothesis-008",
    "statement": "The compromised user escalated privileges by assuming an administrative role after modifying an IAM policy.",
    "confidence": 0.62,
    "status": "active",
    "evidence_ids": ["evidence-031"],
    "alternative_explanations": [
      "Approved emergency administrative activity"
    ]
  }
}
```

## 12. Evidence

```json
{
  "protocol_version": "0.3",
  "type": "evidence_shared",
  "message_id": "msg-050",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:18Z",
  "agent_id": "identity-specialist",
  "task_id": "task-019",
  "evidence": {
    "id": "evidence-031",
    "summary": "The suspected principal assumed the administrative role immediately after an IAM policy change.",
    "source_action_ids": ["action-070", "action-074"],
    "event_ids": ["evt-0012", "evt-0019"],
    "entity_ids": [
      "arn:aws:iam::123456789012:user/suspected",
      "arn:aws:iam::987654321098:role/admin"
    ],
    "time_range": {
      "start": "2026-01-10T10:30:00Z",
      "end": "2026-01-10T10:33:00Z"
    },
    "confidence": 0.86
  }
}
```

Evidence is rejected if it references an unknown action ID or event not present in a HuntEval-issued result.

## 13. Finding proposal

```json
{
  "protocol_version": "0.3",
  "type": "finding_proposed",
  "message_id": "msg-060",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:22Z",
  "agent_id": "identity-specialist",
  "task_id": "task-019",
  "finding": {
    "id": "finding-004",
    "title": "Suspicious cross-account administrative role assumption",
    "severity": "high",
    "status": "proposed",
    "hypothesis_ids": ["hypothesis-008"],
    "evidence_ids": ["evidence-031"],
    "event_ids": ["evt-0012", "evt-0019"],
    "entity_ids": [
      "arn:aws:iam::123456789012:user/suspected",
      "arn:aws:iam::987654321098:role/admin"
    ],
    "attack_techniques": ["T1078"],
    "benign_alternatives": [
      "Approved emergency role assumption"
    ],
    "confidence": 0.84
  }
}
```

## 14. Finding challenge

```json
{
  "protocol_version": "0.3",
  "type": "finding_challenged",
  "message_id": "msg-061",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:24Z",
  "agent_id": "evidence-critic",
  "finding_id": "finding-004",
  "challenge": {
    "reason_code": "missing_benign_validation",
    "summary": "The role is listed as an emergency administration role, but no approval context was checked.",
    "requested_actions": [
      "Verify approved administrative windows and source addresses."
    ]
  }
}
```

## 15. Knowledge retrieval request

```json
{
  "protocol_version": "0.3",
  "type": "tool_request",
  "message_id": "msg-070",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:26Z",
  "agent_id": "identity-specialist",
  "task_id": "task-019",
  "action_id": "action-080",
  "tool": "knowledge_retrieval",
  "purpose": "Check approved emergency administration conditions.",
  "arguments": {
    "query": "approved emergency administrative roles and source networks",
    "top_k": 5
  }
}
```

Returned documents include stable document IDs and citations. The corpus cannot contain hidden ground truth.

## 16. Final submission

```json
{
  "protocol_version": "0.3",
  "type": "final_submission",
  "message_id": "msg-100",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:02:00Z",
  "agent_id": "supervisor",
  "submission": {
    "status": "confirmed_malicious_activity",
    "summary": "A compromised IAM user modified policy permissions and assumed an administrative role in a second account.",
    "finding_ids": ["finding-004"],
    "malicious_event_ids": ["evt-0012", "evt-0019", "evt-0024"],
    "malicious_entity_ids": [
      "arn:aws:iam::123456789012:user/suspected",
      "arn:aws:iam::987654321098:role/admin"
    ],
    "attack_path": ["evt-0012", "evt-0019", "evt-0024"],
    "attack_techniques": ["T1078", "T1098"],
    "confidence": 0.89,
    "limitations": [
      "No endpoint telemetry was available."
    ]
  }
}
```

## 17. Episode manifest

```yaml
schema_version: "0.3"
id: aws-iam-001
title: Cross-account privilege escalation
provider: aws
category: identity_and_access
objective:
  primary: Identify the compromised principal and reconstruct the escalation path.
  secondary:
    - Identify persistence mechanisms.
    - Identify impacted resources.
    - Determine the earliest malicious event.
telemetry:
  tables:
    - name: aws_cloudtrail
      path: telemetry/cloudtrail.parquet
    - name: aws_iam_inventory
      path: telemetry/iam_inventory.parquet
knowledge:
  documents:
    - knowledge/environment-overview.md
    - knowledge/approved-admin-roles.md
limits:
  max_agents: 8
  max_parallel_agents: 4
  max_tool_calls: 40
  max_sql_queries: 25
  max_messages: 100
  max_duration_seconds: 900
  max_tokens: 150000
fault_profile: none
ground_truth_ref: private/ground-truth.json
```

The `ground_truth_ref` is resolved by the trusted runner and is never included in the deployment-visible resolved manifest.

## 18. Result contract

```json
{
  "schema_version": "0.3",
  "run_id": "run-001",
  "episode_id": "aws-iam-001",
  "deployment_id": "supervisor-specialists-v1",
  "status": "completed",
  "metric_vector": {
    "investigation_quality": 0.84,
    "evidence_quality": 0.79,
    "coordination_quality": 0.71,
    "resilience": 0.88,
    "efficiency": 0.62,
    "reproducibility": null
  },
  "aggregate_scores": {
    "accuracy-first@1.0.0": 0.792
  },
  "constraint_violations": [],
  "resource_usage": {
    "duration_ms": 120000,
    "tool_calls": 18,
    "sql_queries": 12,
    "messages": 31,
    "input_tokens": 42000,
    "output_tokens": 9100,
    "estimated_cost": 1.42
  },
  "artifacts": {
    "trajectory": "trajectory.jsonl",
    "submission": "submission.json",
    "metrics": "metrics.json"
  }
}
```

## 19. Protocol errors

Required error codes include:

- `unsupported_protocol_version`;
- `invalid_message`;
- `unknown_agent`;
- `unknown_task`;
- `unknown_action`;
- `duplicate_identifier`;
- `budget_exceeded`;
- `policy_violation`;
- `sql_policy_violation`;
- `tool_timeout`;
- `deployment_timeout`;
- `provenance_violation`;
- `ground_truth_leakage_detected`;
- `invalid_submission`.

## 20. Replay requirements

A trajectory replay must:

- validate sequence and causal references;
- reconstruct task and finding state;
- re-run deterministic evaluators;
- detect missing or altered events;
- never require the original LLM provider.
