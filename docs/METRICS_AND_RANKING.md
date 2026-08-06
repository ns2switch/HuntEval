# Metrics, scoring, and ranking

## 1. Principles

1. Preserve raw metrics and dimension scores.
2. Do not hide trade-offs behind one number.
3. Use aggregate scores only through versioned profiles.
4. Compare stochastic deployments using repeated runs.
5. Report uncertainty and stability.
6. Apply hard constraints before ranking.
7. Avoid claiming a winner when evidence is inconclusive.

## 2. Metric dimensions

### 2.1 Investigation quality

Core metrics:

- malicious event precision;
- malicious event recall;
- malicious event F1;
- malicious entity precision;
- malicious entity recall;
- attack-path accuracy;
- timeline accuracy;
- ATT&CK technique coverage;
- conclusion correctness;
- false-positive rate;
- false-negative rate.

For event sets:

```text
precision = true_positive_events / submitted_malicious_events
recall    = true_positive_events / ground_truth_malicious_events
F1        = 2 * precision * recall / (precision + recall)
```

When both submitted and ground-truth sets are empty, the episode definition must specify whether the correct score is one or not applicable.

### 2.2 Evidence quality

- evidence completeness;
- evidence-grounding rate;
- provenance validity;
- unsupported-claim rate;
- benign-alternative coverage;
- finding-to-event traceability;
- temporal consistency;
- citation validity for optional knowledge documents.

A finding is fully grounded when every required claim references valid evidence derived from HuntEval-issued tool results.

### 2.3 Coordination quality

- task completion rate;
- delegation success rate;
- task allocation quality;
- duplicate task rate;
- redundant query rate;
- orphan task rate;
- useful communication rate;
- evidence propagation latency;
- conflict detection rate;
- conflict resolution rate;
- supervisor bottleneck ratio;
- coordination overhead;
- agent utilization;
- achieved parallelism.

#### Task completion rate

```text
completed_tasks / created_tasks
```

Cancelled tasks that were superseded intentionally should be reported separately rather than counted automatically as failures.

#### Redundant work rate

The ratio of semantically equivalent tasks or tool calls that do not add new evidence. The MVP may use deterministic duplicate keys and normalized SQL fingerprints; semantic duplicate detection can be added later.

#### Useful communication rate

A message is useful when it is referenced by a subsequent action, changes a hypothesis, shares new valid evidence, resolves a conflict, prevents duplicate work, or contributes to a final finding.

### 2.4 Resilience

- successful recovery after agent timeout;
- successful task reassignment;
- final-score degradation under fault;
- malformed-message recovery;
- tool-error recovery;
- noisy-agent resistance;
- missing-agent graceful degradation;
- retry efficiency.

Resilience is evaluated by paired runs with and without a named fault profile.

### 2.5 Efficiency

- wall-clock duration;
- total input and output tokens;
- estimated monetary cost;
- tool-call count;
- SQL-query count;
- rows scanned and returned;
- messages exchanged;
- tokens per true positive;
- queries per true positive;
- duplicated query cost;
- idle time;
- peak active agents.

Efficiency metrics should be normalized against benchmark-defined caps or reference deployments before aggregation.

### 2.6 Reproducibility

- run success rate;
- score variance;
- finding stability;
- entity stability;
- attack-path stability;
- conclusion consistency;
- sensitivity to seed;
- sensitivity to agent scheduling.

Reproducibility is calculated across repetitions, not within a single run.

### 2.7 Normative MVP metric contracts

Every stored raw metric includes `value`, `applicability`, `direction`, `range`, and its numerator and denominator when the metric is a ratio. `value` is `null` when the applicability reason is not `applicable`. Implementations must not replace `null` with zero or one outside an explicit scoring-profile policy.

| Metric | Range | Direction | Numerator | Denominator | Required edge behavior |
|---|---:|---|---|---|---|
| event precision | `[0,1]` | higher is better | submitted malicious event IDs present in truth | unique submitted malicious event IDs | empty submission with non-empty truth is `0`; both empty is `1` only when the episode declares a benign scored case, otherwise not applicable |
| event recall | `[0,1]` | higher is better | submitted malicious event IDs present in truth | unique ground-truth malicious event IDs | empty submission with non-empty truth is `0`; empty truth is not applicable unless the episode declares a benign scored case |
| entity precision | `[0,1]` | higher is better | submitted malicious entity IDs present in truth | unique submitted malicious entity IDs | same empty-set policy as event precision |
| entity recall | `[0,1]` | higher is better | submitted malicious entity IDs present in truth | unique ground-truth malicious entity IDs | same empty-set policy as event recall |
| evidence grounding rate | `[0,1]` | higher is better | evidence items whose action and event references all validate | submitted evidence items | zero evidence is `0` when findings were submitted; otherwise not applicable |
| provenance validity | `{0,1}` | higher is better | one when all submitted provenance validates, otherwise zero | one completed or incomplete run | any missing, future, cross-run, forged, or wrong-owner reference produces `0` |
| task completion rate | `[0,1]` | higher is better | completed tasks | created tasks excluding explicitly superseded cancellations | zero denominator is not applicable |
| tool-call utilization | `[0,1]` | lower is better | managed tool calls used | configured tool-call cap | zero cap and zero use is not applicable; use above the cap is a budget violation and the raw ratio is clamped only for dimension normalization |

Event and entity inputs are treated as sets after identifier validation, so duplicate submitted identifiers do not increase the numerator or denominator. Raw duplicate counts remain available as diagnostics. Attack-path, timeline, conclusion, semantic coordination, resilience, cost-normalized efficiency, and reproducibility metrics are deferred until their individual contracts define the same fields and tests.

## 3. Dimension scores

Each dimension score lies in `[0, 1]`. The scoring profile defines metric normalization and weights within a dimension.

Example:

```yaml
investigation_quality:
  metrics:
    event_recall: 0.30
    event_precision: 0.20
    entity_recall: 0.15
    entity_precision: 0.10
    attack_path_accuracy: 0.15
    conclusion_correctness: 0.10
```

Weights within a dimension must sum to one. Missing non-applicable metrics are renormalized according to an explicit profile policy.

## 4. Aggregate profiles

Example accuracy-first profile:

```yaml
name: accuracy-first
version: 1.0.0
weights:
  investigation_quality: 0.40
  evidence_quality: 0.25
  coordination_quality: 0.15
  resilience: 0.10
  efficiency: 0.05
  reproducibility: 0.05
constraints:
  min_event_recall: 0.70
  max_false_positive_rate: 0.20
  no_ground_truth_leakage: true
  no_critical_policy_violation: true
non_applicable:
  dimension_policy: reject
```

Aggregate score:

```text
score = sum(dimension_score[d] * profile_weight[d])
```

A deployment that violates a disqualifying constraint is excluded or clearly marked, even when its weighted score is high.

## 5. Supported profile families

- `accuracy-first`;
- `evidence-first`;
- `production-cost-aware`;
- `resilience-first`;
- custom organization profiles.

HuntEval must display rankings under the selected profile and preserve the full metric vector.

## 6. Repetitions and experiment matrix

A comparison consists of:

```text
deployment x episode x seed x configuration
```

The ordered list of unique seeds defines the repetitions. A legacy `repetitions` field may be accepted only when it equals the number of listed seeds; otherwise manifest validation fails. Seeds are paired across deployments for the same episode and declared configuration. Missing and failed cells remain visible and are never imputed silently.

Example:

```text
4 deployments x 9 episodes x 5 repetitions = 180 runs
```

All compared deployments must use equivalent episode versions and budgets unless the comparison is explicitly labeled non-equivalent.

## 7. Statistical summaries

For each metric and aggregate score, report:

- count;
- mean;
- median;
- standard deviation;
- minimum and maximum;
- percentile range;
- confidence interval;
- success rate;
- failure rate.

The initial implementation may use bootstrap confidence intervals because metric distributions may not be normal.

## 8. Pairwise comparison

For two deployments, report:

- mean and median difference;
- confidence interval for the difference;
- effect size;
- paired episode-level wins, ties, and losses;
- cost difference;
- constraint violations.

Example conclusion:

```text
specialists-v2 improved the accuracy-first score by 0.035 over supervisor-workers-v1.
The confidence interval overlaps zero, so HuntEval cannot identify a conclusive winner.
```

The report must not overstate statistical evidence.

## 9. Ranking policy

Recommended order:

1. Remove or flag deployments that violate disqualifying constraints.
2. Rank remaining deployments by the selected aggregate profile.
3. Use confidence intervals for paired differences and pairwise tests to group statistically indistinguishable deployments; overlap of separate marginal confidence intervals is not a decision rule.
4. Present cost and latency as explicit trade-offs.
5. Show per-provider and per-scenario-family rankings.

A leaderboard row should include:

- deployment;
- aggregate score;
- confidence interval;
- stability;
- success rate;
- average cost;
- average duration;
- constraint status.

## 10. Agent-level attribution

Per-agent metrics may include:

- completed tasks;
- valid evidence produced;
- true-positive events introduced;
- false-positive findings introduced;
- invalid queries;
- redundant actions;
- messages used downstream;
- cost and latency;
- findings accepted, rejected, or corrected;
- marginal contribution estimates.

Agent-level scores are diagnostic and should not be used as a universal ranking without controlling for assigned roles.

## 11. Marginal contribution

A future version may estimate contribution through ablation:

```text
full deployment score - deployment score without agent or capability
```

Because removing an agent changes coordination dynamics, this value is an experimental estimate rather than causal proof.

## 12. Regression detection

Given a baseline and candidate deployment or prompt version, HuntEval reports:

- improved metrics;
- regressed metrics;
- unchanged metrics;
- constraint changes;
- cost changes;
- uncertainty.

A candidate cannot be accepted automatically when it improves the aggregate score but violates a critical minimum metric.

## 13. Report example

```text
Best deployment under accuracy-first@1.0.0: supervisor-specialists-v3

Strengths:
- highest event recall;
- strongest evidence completeness;
- stable results across seeds;
- successful recovery from query-agent timeout.

Weaknesses:
- high coordination traffic;
- duplicate work between the identity and query specialists;
- higher cost than the supervisor-worker baseline.

Statistical status:
- conclusive improvement over single-agent-v1;
- inconclusive difference from supervisor-specialists-v2.
```
