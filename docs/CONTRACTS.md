# Domain contracts and process protocol

## 1. Versioning rules

Every protocol message includes:

- `protocol_version`;
- `message_id`;
- `run_id`;
- `timestamp`;
- `type`.

Backward-incompatible changes increment the major protocol version. New optional fields may increment the minor version. Unknown optional fields must be ignored; unknown message types must produce a structured protocol error. A session has a configured maximum UTF-8 line size; oversized, non-UTF-8, non-object, or multi-value lines are rejected before payload deserialization.

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

Runner messages may also include `caused_by_message_id`. The runner assigns the timestamp and append-only sequence of scored trajectory events. A deployment-supplied timestamp is retained only as untrusted protocol metadata and never controls ordering, duration, timeout, or budget calculations.

### 2.1 Session lifecycle

The runner initiates the bidirectional session:

```json
{
  "protocol_version": "0.3",
  "type": "run_started",
  "message_id": "msg-runner-001",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:00Z",
  "supported_protocol": {"minimum": "0.3", "maximum": "0.3"},
  "episode": {
    "id": "aws-iam-001",
    "objective": "Identify the compromised principal and reconstruct the escalation path.",
    "tables": ["aws_cloudtrail", "aws_iam_inventory"]
  },
  "limits": {
    "max_agents": 8,
    "max_tool_calls": 40,
    "max_messages": 100,
    "max_duration_seconds": 900
  },
  "seed": 11
}
```

The deployment must respond with exactly one `register_deployment`. After validation, the runner sends:

```json
{
  "protocol_version": "0.3",
  "type": "registration_accepted",
  "message_id": "msg-runner-002",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:01Z",
  "caused_by_message_id": "msg-001",
  "selected_protocol_version": "0.3",
  "remaining_budgets": {
    "tool_calls": 40,
    "messages": 99
  }
}
```

Scored actions are invalid before `registration_accepted`. Every runner response correlates to its causal deployment message when one exists. A session ends with `run_terminated`. EOF, process exit, or timeout before normal termination creates a structured terminal failure event and a normalized incomplete result.

## 3. Deployment registration

```json
{
  "protocol_version": "0.3",
  "type": "register_deployment",
  "message_id": "msg-001",
  "run_id": "run-001",
  "timestamp": "2026-08-06T18:00:00Z",
  "selected_protocol_version": "0.3",
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
  "caused_by_message_id": "msg-030",
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
  max_parallel_tool_calls: 2
  max_outstanding_tasks: 16
  max_delegation_depth: 4
  max_tool_calls: 40
  max_sql_queries: 25
  max_retrieved_documents: 0
  max_messages: 100
  max_duration_seconds: 900
  max_tokens: 150000
  max_estimated_cost: null
fault_profile: none
```

The public manifest is stored under `public/manifest.yaml`. A trusted package index, which is never exposed to the deployment, binds the physically separate roots:

```yaml
schema_version: "0.3"
episode_id: aws-iam-001
public_root: public
private_ground_truth: private/ground-truth.json
```

The runner resolves both roots without following unexpected symlinks. Private paths, ground-truth references, private hashes, and private labels are never included in the deployment-visible descriptor, environment, logs, or resolved public manifest. Scored execution fails closed when the configured isolation backend cannot enforce this boundary.

## 18. Result contract

```json
{
  "schema_version": "0.3",
  "run_id": "run-001",
  "episode_id": "aws-iam-001",
  "deployment_id": "supervisor-specialists-v1",
  "status": "completed",
  "raw_metrics": {
    "event_precision": {
      "value": 1.0,
      "applicability": "applicable",
      "direction": "higher_is_better",
      "range": {"minimum": 0.0, "maximum": 1.0},
      "numerator": 3,
      "denominator": 3
    },
    "reproducibility": {
      "value": null,
      "applicability": "requires_repeated_runs",
      "direction": "higher_is_better",
      "range": {"minimum": 0.0, "maximum": 1.0},
      "numerator": null,
      "denominator": null
    }
  },
  "metric_vector": {
    "investigation_quality": 0.84,
    "evidence_quality": 0.79,
    "coordination_quality": 0.71,
    "resilience": null,
    "efficiency": 0.62,
    "reproducibility": null
  },
  "aggregate_scores": {},
  "aggregate_score_omissions": {
    "accuracy-first@1.0.0": "required_dimension_not_applicable"
  },
  "constraint_violations": [],
  "resource_usage": {
    "duration_ms": 120000,
    "tool_calls": 18,
    "sql_queries": 12,
    "messages": 31,
    "input_tokens": 42000,
    "output_tokens": 9100,
    "token_provenance": "verified_adapter",
    "estimated_cost": {
      "value": 1.42,
      "provenance": "verified_adapter",
      "currency": "USD"
    }
  },
  "artifacts": {
    "trajectory": "trajectory.jsonl",
    "submission": "submission.json",
    "metrics": "metrics.json"
  }
}
```

Runner-observed resource fields are `measured`. Provider-dependent values such as tokens and monetary cost explicitly use provenance `verified_adapter`, `self_reported`, or `unavailable`. A scoring constraint cannot treat a self-reported or unavailable value as verified; the run must instead be marked non-comparable for that constraint.

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

After replay, the runner reduces stored artifacts into a `TrustedRunView` before metric code executes. The reducer:

- reads bounded, regular `trajectory.jsonl` and `submission.json` files without following symlinks;
- verifies both exact-byte digests against runner-owned metadata;
- requires one completed protocol session and an exact match between the stored and terminal submissions;
- projects typed actions, tasks, evidence, findings, operational messages, and their owners;
- preserves replay sequence numbers, task-state transitions, and explicit `caused_by_message_id` links for observable coordination metrics;
- rejects cross-run, future, duplicate, unknown, wrongly owned, or unissued references;
- requires finding events and entities to be supported by referenced evidence, and submitted events, entities, and attack-path entries to be supported by referenced findings;
- attaches evaluator-only ground truth only after producing the serializable deployment-safe observation projection.

`TrustedRunView` is not serializable. Its `ObservedRun` projection is serializable for diagnostics and contains no ground-truth field or private value. Metric inputs are derived from the validated view; evidence and provenance counts are never inferred from raw message counts.

Every projected causal reference must resolve to a lower trajectory sequence in the same run. Useful communication requires a later tool request or task transition that explicitly cites the operational message, belongs to its target agent, and matches its optional task scope. Message wording, reason-code wording, and temporal proximity cannot create a causal link.

Duplicate-work fingerprints contain a validated lowercase tool identifier and recursively canonical JSON arguments. Object keys are sorted, array order is preserved, insignificant whitespace is absent, and nesting, node count, and canonical byte size are bounded. A repeated fingerprint contributes duplicate work only when it introduces no previously unseen grounded evidence identifier for that fingerprint. The metric retains the duplicate count and completed-call denominator.

## 21. Schema 0.4 compatibility

Schema `0.4` is additive to the persisted `0.3` contracts. A reader advertises the exact minor versions it accepts and applies an explicit adapter for each older version. It must reject an unknown newer minor or incompatible major version before consuming payload fields. Adaptation never edits a source artifact.

A `0.3` final submission adapted to `0.4` has no structured timeline. Its timeline-dependent metrics use `value: null` and `applicability: timeline_not_submitted`; implementations must not derive timeline entries from the summary. A `0.3` ground-truth artifact has no expected timeline windows or structured acceptable statuses, so the corresponding metrics are likewise not applicable.

The in-memory compatibility adapter preserves those missing fields as `None`. A v0.4 ground-truth artifact must provide a non-empty set of acceptable submission statuses and an explicit expected-window array. A standalone v0.4 submission must provide an explicit timeline array, which may be empty. Adapters reject unknown versions, reversed or duplicate expected windows, duplicate submitted event IDs, malformed UTC timestamps, and unknown fields. Normalization never rewrites the source artifact.

All `0.4` JSON objects use `additionalProperties: false` at trust boundaries. Optional values represent declared absence; unknown fields are not treated as optional compatibility extensions.

## 22. Authored benchmark manifest

The human-authored YAML manifest contains safe relative references only:

```yaml
schema_version: "0.4"
id: cloud-r2
deployments:
  - deployments/single-agent-scripted
episodes:
  - datasets/aws/aws-iam-001
seeds: [11, 29]
scoring_profile: examples/scoring-profile-balanced.yaml
fault_profile: null
```

The runner validates identifiers, rejects absolute paths, parent traversal, symlink escape, duplicates, and an empty matrix, then resolves and hashes every referenced artifact. Filesystem paths do not enter the domain benchmark definition.

## 23. Benchmark cell identity

The resolved cell key is serialized as canonical JSON with lexicographically ordered object keys and no insignificant whitespace:

```json
{
  "benchmark_id": "cloud-r2",
  "deployment": {
    "id": "single-agent-scripted",
    "configuration_sha256": "0000000000000000000000000000000000000000000000000000000000000000"
  },
  "episode": {
    "id": "aws-iam-001",
    "package_sha256": "1111111111111111111111111111111111111111111111111111111111111111"
  },
  "seed": 11,
  "scoring_profile": {
    "id": "balanced@1.0.0",
    "sha256": "2222222222222222222222222222222222222222222222222222222222222222"
  },
  "fault_profile": null
}
```

`BenchmarkCellId` is `cell:` followed by the lowercase SHA-256 of those exact canonical bytes. `BenchmarkAttemptId` is an opaque stable identifier supplied by the controller. Attempt identity, timestamps, local paths, and machine details never affect cell identity.

## 24. Benchmark journal and state

Each line of `benchmark-events.jsonl` is one `BenchmarkEvent` with:

- schema version, benchmark ID, monotonically increasing sequence, UTC timestamp, and previous-event digest;
- event type and cell ID when the event is cell-scoped;
- attempt ID for attempt transitions;
- a concise typed reason code for failure, interruption, or non-comparability;
- run ID and normalized result digest only for a successfully completed attempt.

Allowed cell statuses are:

- `pending`: no attempt currently owns the cell;
- `running`: an attempt has started and has not emitted a terminal event;
- `completed`: a validated normalized result exists;
- `failed`: the latest attempt ended without a comparable result;
- `non_comparable`: execution may have completed, but declared equivalence or verification requirements failed.

The journal is authoritative. `benchmark-state.json` contains its last sequence and digest plus a deterministic, cell-ID-ordered projection. Resume first validates the complete hash chain. A prior `running` attempt is closed with reason `controller_interrupted` before a new attempt starts. Invalid transitions, reused attempt IDs, missing causal state, and altered history fail closed.

## 25. Comparison eligibility

Comparison eligibility is a structured result, not a boolean. `eligible` requires equal episode package, scoring profile, protocol, schema, declared budgets, deployment comparison configuration, seed, and fault pairing policy. Otherwise status is `ineligible` with one or more stable reason codes and the affected cell IDs. Missing and failed cells remain visible and are never imputed.

Initial reason codes are:

- `missing_cell`;
- `cell_not_completed`;
- `episode_hash_mismatch`;
- `scoring_profile_mismatch`;
- `schema_version_mismatch`;
- `protocol_version_mismatch`;
- `budget_mismatch`;
- `configuration_mismatch`;
- `seed_not_paired`;
- `fault_pair_mismatch`;
- `artifact_verification_failed`.

## 26. Deployment process configuration

An executable deployment declares a safe relative executable reference, fixed argument strings, and allowlisted environment variable names. It cannot store environment values, request additional tool authority, enable network access, or reference evaluator-only paths. The runner resolves the executable without following an unexpected symlink and hashes its exact bytes before execution. Standard output remains protocol-only; bounded operational diagnostics use standard error.

## 27. Structured timeline entries

A `0.4` final submission may include ordered timeline entries:

```json
{
  "event_id": "evt-0012",
  "observed_at": "2026-08-06T18:00:00Z",
  "summary": "The suspected principal assumed the administrative role.",
  "evidence_ids": ["evidence-004"]
}
```

Ground truth stores evaluator-only expected windows with an event ID, earliest and latest acceptable UTC timestamps, and optional acceptable submission statuses. Public schemas cannot represent expected windows, acceptable statuses, or any private reference. Timeline order is the submitted array order; duplicate event IDs are invalid.

## 28. Typed report source references

A report claim contains a stable claim ID, bounded text, and at least one typed source reference. Supported source kinds and required values are:

- `metric_pointer`: an RFC 6901 pointer rooted in normalized public result JSON;
- `trajectory_sequence`: run ID and positive sequence number;
- `run`: run ID;
- `benchmark_cell`: benchmark and cell IDs;
- `constraint`: run or comparison scope plus constraint ID;
- `statistical_comparison`: comparison ID;
- `artifact_digest`: artifact label and SHA-256 digest.

References never contain filesystem paths, raw private values, environment values, or arbitrary URLs. Renderers validate reference ownership and escape all claim text before output.

## 29. Efficiency and stability inputs

Run-level efficiency consumes trusted resource observations, not deployment prose. `measured_duration_utilization` divides runner-observed process duration by the configured duration cap and caps the normalized numerator at the cap. A zero duration cap is not applicable. The uncapped duration remains in `resource_usage`; timeout or cap enforcement is represented separately from the normalized metric.

`verified_cost_utilization` is applicable only when a configured adapter supplies a finite nonnegative cost with `verified_adapter` provenance and the episode supplies a positive finite cost cap. Self-reported and unavailable costs produce `requires_verified_resource_usage`. Missing caps produce `unavailable_resource`, and a zero cap produces `zero_denominator`. Normalization never upgrades self-reported data into verified data.

Benchmark stability groups cells by deployment and episode and consumes the benchmark's canonical listed seed order. Every required seed must resolve to exactly one digest-verified, schema-valid completed cell. Missing, failed, or invalid cells are retained as typed unavailable repetitions and make stability `requires_comparable_cells`; they are never replaced or imputed. A benchmark declaring fewer than two seeds produces `requires_repeated_runs`.

Submission stability is the mean pairwise Jaccard similarity of canonical structured claims: status, confidence, malicious event and entity IDs, ATT&CK technique IDs, ordered attack-path positions, and ordered timeline event/time pairs. Free-form summaries, limitations, finding IDs, and evidence IDs are excluded. Metric stability is one minus the mean absolute difference across identical sets of applicable bounded run metrics. A mismatched or empty applicable metric set is explicitly non-comparable. Results retain required sample count, completed sample count, pair counts, contributing cell IDs, and unavailable seeds. Pairwise aggregation enforces deterministic sample and comparison-work bounds and fails closed before allocating or executing an unbounded quadratic comparison.

## 30. Scoring profile v0.4

A v0.4 scoring profile contains a stable ID, an explicit missing-value policy, one or more metric selections, and a bounded constraint array. Every selection names a registered metric through its map key and supplies an exact contract version plus a finite nonnegative weight. All weights sum to one. Metric range and direction come only from the registry; an evaluated metric whose range or direction disagrees with its registered contract is rejected.

The missing-value policies are `reject`, `renormalize`, and `zero`. `reject` yields no aggregate when any selected metric is unavailable. `renormalize` omits ordinary unavailable metrics, but cannot omit resilience, graceful degradation, reproducibility, submission stability, metric stability, or verified cost. `zero` retains the selected weight with the worst normalized contribution. No policy silently converts an unavailable protected metric into success.

Constraints are either `observed_violation` or `metric_threshold`. Every threshold identifies a registered metric and version, comparison, bounded threshold, disqualifying flag, and an explicit resource-provenance requirement: `none`, `measured`, or `verified_adapter`. The requirement must exactly match the registry. A value with missing, self-reported, unavailable, or otherwise mismatched provenance produces `unverifiable`, never `satisfied`. Constraint codes are unique canonical identifiers.

The compatibility loader accepts the immutable v0.3 profile shape, maps every legacy weight to the registered v0.3 metric contract, maps legacy disqualifying codes to typed observed-violation constraints, and returns a normalized in-memory v0.4 profile. It hashes and preserves the original bytes and never rewrites, enriches, or infers values in the v0.3 source. Unknown profile versions, metric names, metric versions, fields, weights, directions, provenance requirements, or duplicate constraint codes fail closed.
