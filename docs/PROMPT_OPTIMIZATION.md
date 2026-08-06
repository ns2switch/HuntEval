# Prompt diagnosis and optimization

## 1. Objective

HuntEval should help improve agent prompts without confusing correlation with causation. The mandatory workflow is:

```text
observable traces
-> failure classification
-> agent attribution
-> prompt inspection
-> improvement hypothesis
-> candidate patch
-> A/B experiment
-> validation
```

A generated patch is a recommendation, not a verified improvement.

## 2. Required inputs

Prompt diagnosis requires:

- versioned prompt text or immutable prompt artifact;
- prompt hash;
- agent role and capabilities;
- model and parameters;
- run trajectories;
- metric results;
- failure classifications;
- affected episodes and seeds;
- comparison baseline.

Without prompt artifacts and provenance, HuntEval must not claim that a prompt caused a behavior.

## 3. Failure taxonomy

### 3.1 Investigation failures

- `missed_malicious_event`;
- `incorrect_entity_attribution`;
- `incorrect_attack_path`;
- `unsupported_conclusion`;
- `premature_conclusion`;
- `missed_alternative_hypothesis`;
- `incorrect_timeline`;
- `failure_to_validate_benign_explanation`.

### 3.2 Tool-use failures

- `invalid_sql`;
- `wrong_table_selection`;
- `wrong_field_mapping`;
- `overly_broad_query`;
- `overly_narrow_query`;
- `repeated_query`;
- `unnecessary_tool_call`;
- `ignored_tool_error`;
- `unbounded_result_request`.

### 3.3 Coordination failures

- `incorrect_delegation`;
- `duplicate_task`;
- `orphan_task`;
- `evidence_not_shared`;
- `evidence_ignored`;
- `conflict_not_resolved`;
- `supervisor_bottleneck`;
- `excessive_message_loop`;
- `task_not_reassigned`;
- `capability_mismatch`.

### 3.4 Prompt weaknesses

- `role_ambiguity`;
- `missing_output_schema`;
- `missing_acceptance_criteria`;
- `missing_stop_condition`;
- `missing_evidence_requirements`;
- `unclear_tool_policy`;
- `insufficient_error_handling`;
- `insufficient_delegation_rules`;
- `overly_broad_responsibility`;
- `overly_verbose_communication`;
- `missing_conflict_resolution_policy`;
- `missing_provenance_requirement`.

## 4. Failure-to-prompt mapping

Initial diagnosis may use deterministic rules.

Examples:

| Observed failures | Candidate prompt weakness |
|---|---|
| unsupported conclusions and missing event IDs | missing evidence requirements |
| duplicate tasks and repeated queries | insufficient delegation rules |
| invalid SQL after tool errors | insufficient error handling |
| findings accepted without benign alternatives | missing acceptance criteria |
| excessive messages without downstream references | overly verbose communication |
| tasks assigned to agents without matching capabilities | unclear delegation policy |
| unresolved contradictory findings | missing conflict resolution policy |

These mappings generate hypotheses that require review and testing.

## 5. Structured recommendation

```json
{
  "agent_id": "evidence-critic",
  "prompt_version": "1.2.0",
  "issue": "unsupported_conclusion",
  "evidence": {
    "affected_runs": 14,
    "affected_episodes": 5,
    "false_positive_findings": 9,
    "example_run_ids": ["run-014", "run-021"]
  },
  "diagnosis": {
    "candidate_prompt_weakness": "missing_acceptance_criteria",
    "confidence": 0.87
  },
  "recommendation": {
    "target_section": "Finding acceptance criteria",
    "change_type": "add_constraint",
    "rationale": "The critic accepts findings that do not contain event identifiers or a tested benign alternative.",
    "expected_effects": [
      "reduce false positives",
      "increase evidence completeness"
    ],
    "possible_trade_offs": [
      "increase missed findings",
      "increase tool usage"
    ]
  }
}
```

## 6. Example prompt patch

```diff
- Review proposed findings and determine whether they are reasonable.
+ Evaluate every proposed finding against the available telemetry.
+ Return ACCEPTED, REJECTED, or INSUFFICIENT_EVIDENCE.
+ Do not accept a finding unless it includes:
+ - at least one supporting event identifier;
+ - an affected principal or resource;
+ - a timestamp or bounded time range;
+ - a causal explanation linking evidence to the hypothesis;
+ - consideration of at least one plausible benign explanation.
```

## 7. Prompt artifact model

```yaml
deployment:
  id: supervisor-specialists-v2
agents:
  - id: supervisor
    prompt:
      path: deployment/supervisor.md
      version: 2.1.0
      sha256: "..."
      immutable_sections:
        - authorization_policy
        - tool_access_policy
        - data_handling_policy
  - id: evidence-critic
    prompt:
      path: deployment/evidence-critic.md
      version: 1.4.0
      sha256: "..."
```

The resolved prompt, variables, tool instructions, and output schema must be hashed for each run.

## 8. A/B prompt comparison

The comparison changes one prompt or a declared set of prompt sections while keeping other relevant variables fixed.

```bash
hunteval prompt compare \
  --deployment supervisor-specialists \
  --agent evidence-critic \
  --baseline critic-v1.md \
  --candidate critic-v2.md \
  --benchmark cloud-mvp \
  --repetitions 10
```

Example result:

```text
Metric                    baseline   candidate   difference
False-positive rate       18.4%      9.7%        -8.7 pp
Evidence completeness     0.71       0.86        +0.15
Missed findings           7.2%       8.0%        +0.8 pp
Average tokens            1,842      2,031       +10.3%
```

HuntEval should recommend adoption only when the candidate satisfies predefined constraints and validation criteria.

## 9. Dataset partitions

Prompt development requires:

- **training episodes:** available for diagnosis and candidate generation;
- **validation episodes:** used to select candidates;
- **hidden test episodes:** used only for final evaluation.

Candidate prompts must never receive hidden test ground truth or evaluator feedback.

## 10. Optimization constraints

```yaml
prompt_optimization:
  immutable_sections:
    - authorization_policy
    - tool_access_policy
    - data_handling_policy
    - benchmark_integrity_policy
  allowed_targets:
    - task_planning
    - evidence_requirements
    - delegation_strategy
    - stopping_conditions
    - communication_format
    - error_recovery
  constraints:
    max_prompt_growth_percent: 25
    preserve_output_schema: true
    no_ground_truth_references: true
    no_hidden_episode_references: true
    human_review_required: true
```

## 11. Safety and integrity rules

A candidate prompt must be rejected when it:

- references hidden ground truth;
- memorizes episode-specific answers;
- weakens authorization or tool restrictions;
- requests private chain of thought;
- bypasses HuntEval-managed tools;
- removes required provenance;
- gains score through excessive cost beyond profile limits;
- is evaluated only on the episodes used to generate it.

## 12. Future automatic candidate generation

A future optimizer may:

1. select a recurring failure pattern;
2. identify an allowed prompt section;
3. generate multiple bounded candidates;
4. run them on training episodes;
5. reject unsafe or constraint-violating candidates;
6. select candidates on validation episodes;
7. evaluate the final candidate on hidden tests.

The optimizer must store every candidate, patch, rationale, run, and selection decision.

## 13. RAG over generated reports

A later report-query capability may index:

- deployment reports;
- metrics;
- coordination events;
- failure classifications;
- prompt versions;
- recommendations;
- A/B comparisons.

Example questions:

- Why did deployment A outperform deployment B?
- Which prompt changes reduced false positives?
- Which agent creates the most redundant queries?
- Which deployment is most robust when the query specialist fails?

Answers must cite concrete run and artifact identifiers.
