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
- attack-path precision and recall;
- timeline precision and recall;
- ATT&CK technique precision and recall;
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
| graceful degradation | `[0,1]` | higher is better | paired fault-run quality, capped at baseline quality | paired baseline-run quality | a zero baseline denominator is not applicable; non-finite or out-of-range inputs are rejected; runs must share episode, seed, and configuration except for the declared fault profile |

Event and entity inputs are treated as sets after identifier validation, so duplicate submitted identifiers do not increase the numerator or denominator. Raw duplicate counts remain available as diagnostics. A run without a paired fault run records resilience as `null` with `requires_fault_pair` applicability. Attack-path, timeline, structured-conclusion, and technique contracts are implemented by R2-08; evidence and coordination by R2-09; provenance-aware efficiency plus cross-run stability by R2-10; and registry-backed scoring and compatibility by R2-11.

## 3. Dimension scores

Each normalized score lies in `[0, 1]`. A v0.4 scoring profile selects exact metric name/version pairs and assigns weights that sum to one. Metric direction is obtained from the compiled contract registry and cannot be authored or overridden by the profile.

Example:

```yaml
schema_version: "0.4"
id: investigation-example
missing_metric_policy: reject
metrics:
  event_recall: {version: "0.3", weight: 0.40}
  event_precision: {version: "0.3", weight: 0.20}
  attack_path_recall: {version: "0.4", weight: 0.25}
  conclusion_correctness: {version: "0.4", weight: 0.15}
constraints: []
```

The missing-value policy is one of `reject`, `renormalize`, or `zero`. Missing resilience, graceful degradation, submission stability, metric stability, or verified cost cannot be renormalized away: `reject` and `renormalize` produce no aggregate, while `zero` applies an explicit worst contribution. Reproducibility is represented by the exact `submission_stability` and `metric_stability` contracts rather than a generic selectable metric.

## 4. Aggregate profiles

Example accuracy-first profile:

```yaml
schema_version: "0.4"
id: verified-cost-aware
missing_metric_policy: reject
metrics:
  event_recall: {version: "0.3", weight: 0.70}
  evidence_event_coverage: {version: "0.4", weight: 0.20}
  verified_cost_utilization: {version: "0.4", weight: 0.10}
constraints:
  - kind: metric_threshold
    code: maximum_verified_cost
    metric: {name: verified_cost_utilization, version: "0.4"}
    comparison: maximum
    threshold: 0.80
    disqualifying: true
    required_resource_provenance: verified_adapter
```

Aggregate score:

```text
score = sum(dimension_score[d] * profile_weight[d])
```

A constraint result is `satisfied`, `violated`, or `unverifiable`. An unverifiable hard resource constraint is never considered satisfied. Disqualifying results remain explicit even when the weighted score is high.

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

## 14. Schema 0.4 evaluation boundary

Schema `0.4` introduces the structured inputs required by the remaining metric contracts without enabling those metrics implicitly. Evaluation consumes only a trusted normalized view produced after protocol replay, provenance validation, submission validation, artifact verification, and evaluator-only ground-truth loading. Metric modules do not parse files, execute queries, or inspect free-form summaries for hidden labels.

Run-level metrics consume one validated run and its private evaluation view. Benchmark-level metrics consume explicitly eligible cells and paired repetitions. A run-level result cannot claim stability, reproducibility, or comparative efficiency by itself.

The following applicability reasons are reserved for the `0.4` metrics:

- `timeline_not_submitted`;
- `timeline_truth_unavailable`;
- `acceptable_statuses_unavailable`;
- `insufficient_evidence_requirements`;
- `requires_repeated_runs`;
- `requires_verified_resource_usage`;
- `requires_comparable_cells`;
- `requires_fault_pair`.

Structured timeline entries preserve submitted order, event identity, observable time, and evidence references. Expected timeline windows remain evaluator-only. A `0.3` artifact adapted into `0.4` does not gain a timeline or acceptable status set; its dependent metrics remain `null` with the corresponding applicability reason. Attack-path precision and recall use exact longest-common-subsequence matching; the implementation fails closed before an unbounded quadratic comparison. Timeline matching is one-to-one by event identifier with inclusive UTC windows. Conclusion correctness compares only the structured status. Technique precision and recall accept exact ATT&CK technique or sub-technique identifiers and reject unsupported forms.

| R2-08 metric | Range / direction | Numerator / denominator and normalization | Applicability and tested edges |
|---|---|---|---|
| `attack_path_precision` | `[0,1]`, higher | exact LCS length / submitted path length | empty submitted path against non-empty truth is `0`; exact, partial, reordered, duplicate, empty, and benign fixtures |
| `attack_path_recall` | `[0,1]`, higher | exact LCS length / expected path length | empty expected path is `not_required`, except two empty benign paths score `1`; bounded matching-pair expansion fails closed |
| `timeline_precision` | `[0,1]`, higher | distinct submitted entries inside their event's inclusive expected UTC window / submitted entries | `timeline_not_submitted` and `timeline_truth_unavailable` remain distinct; boundary, outside-window, duplicate, empty, and malformed-time fixtures |
| `timeline_recall` | `[0,1]`, higher | one-to-one matched expected windows / expected windows | zero expected entries is `not_required`, except an explicitly empty benign pair scores `1` |
| `conclusion_correctness` | `{0,1}`, higher | exact submitted structured status membership / one episode | `acceptable_statuses_unavailable` for v0.3; matching, non-matching, empty-invalid, and unavailable fixtures; summary text is ignored |
| `technique_precision` | `[0,1]`, higher | exact submitted ATT&CK technique intersection / submitted techniques | set semantics, benign empty behavior, sub-technique support, and unsupported-identifier rejection |
| `technique_recall` | `[0,1]`, higher | exact submitted ATT&CK technique intersection / expected techniques | empty expected set follows event-recall applicability; exact and partial fixtures |

| R2-09 metric | Range / direction | Numerator / denominator and normalization | Applicability and tested edges |
|---|---|---|---|
| `evidence_event_coverage` | `[0,1]`, higher | truth events cited by grounded evidence reachable from submitted findings / truth events | forged or unreferenced evidence cannot enter the numerator; empty truth follows event-recall benign applicability |
| `evidence_entity_coverage` | `[0,1]`, higher | truth entities cited by grounded evidence reachable from submitted findings / truth entities | noise entities do not increase coverage; grounded, partial, forged, empty, and benign fixtures |
| `evidence_sufficiency` | `[0,1]`, higher | distinct reachable grounded evidence items capped at the private minimum / minimum required items | zero required items is `insufficient_evidence_requirements`; numerator is capped rather than rewarded above the requirement |
| `duplicate_tool_work` | `[0,1]`, lower | repeated canonical tool fingerprints adding no new grounded evidence ID / completed tool calls | zero calls is `zero_denominator`; equivalent object-key ordering, new evidence, no evidence, invalid tool names, and bounded arguments are tested |
| `useful_communication` | `[0,1]`, higher | operational messages directly cited by a later target-agent action or task transition / operational messages | zero messages is `zero_denominator`; prose-only, wrong-target, unknown, future, reassignment, and cancellation paths are explicit |

| R2-10 metric | Range / direction | Numerator / denominator and normalization | Applicability and tested edges |
|---|---|---|---|
| `measured_duration_utilization` | `[0,1]`, lower | runner-measured process milliseconds capped at the configured duration cap / duration cap | zero cap is `zero_denominator`; zero, partial, at-cap, and exceeded-cap measurements are deterministic; the uncapped observation remains in resource usage |
| `verified_cost_utilization` | `[0,1]`, lower | verified-adapter cost / configured cost cap, capped at `1` | self-reported or unavailable cost is `requires_verified_resource_usage`; missing cap is `unavailable_resource`; zero cap is `zero_denominator`; malformed provenance and non-finite values fail closed |
| `submission_stability` | `[0,1]`, higher | mean pairwise Jaccard similarity of canonical structured claims over the exact listed seed set | one seed is `requires_repeated_runs`; missing, failed, invalid, or unverified cells are `requires_comparable_cells`; free-form text and run-local provenance IDs are excluded |
| `metric_stability` | `[0,1]`, higher | one minus mean absolute pairwise difference across identical applicable bounded metric vectors | metric-key mismatch, empty vectors, or unavailable required cells are `requires_comparable_cells`; deterministic seed ordering, identical, divergent, missing, failed, and tampered fixtures are tested |

Comparison and report claims must retain typed source references. Statistical summaries cite comparison IDs and contributing cell IDs; run metric claims cite metric pointers and, where applicable, trajectory sequences. A renderer cannot turn an uncited diagnostic statement into a benchmark conclusion.

R2-08 through R2-10 provide normative contracts and positive and negative fixtures for investigation, evidence, coordination, efficiency, and stability. R2-11 registers their exact names, versions, directions, resource provenance requirements, missing-value behavior, typed constraints, and v0.3 compatibility policy. Profile normalization is deterministic and never modifies the source artifact.
