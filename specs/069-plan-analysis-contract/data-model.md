# Data Model: Plan Analysis Contract

## Entity: Planning Analysis Assessment

Represents the effective end-to-end coherence decision for one active
goal-derived planning session.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `state` | `clean`, `findings`, or `blocked` | Yes | `clean` only when no coherence finding remains; `findings` for non-blocking mismatches that operators should still inspect; `blocked` for execution-unsafe coherence defects. |
| `findings` | Ordered list of `PlanningAnalysisFinding` | No | Findings must be deduplicated by material issue so one defect does not appear as multiple blockers without meaningful distinction. |
| `coverage` | `PlanningAnalysisCoverageSummary` | No | Present when the analysis ran and could compute execution-readiness coverage metrics from the current planning state. |

The assessment is persisted additively inside the existing `GoalPlan` record
and omitted entirely for snapshots where planning analysis has never run.

## Entity: Planning Analysis Finding

Represents one explicit coherence defect or warning discovered by the
read-only planning-analysis gate.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `severity` | `critical` or `medium` | Yes | `critical` blocks execution handoff; `medium` remains visible without blocking by itself. |
| `source` | `goal`, `plan`, `backlog`, `validation`, `risk`, `constraint`, `execution_readiness`, or `governed_evidence` | Yes | Source identifies which planning surface primarily owns the defect. |
| `code` | Stable finding identifier | Yes | Codes remain machine-readable and stable across assistant/runtime surfaces. |
| `message` | Operator-readable summary | Yes | Message explains the defect and the repair target without rewriting the underlying artifact. |
| `source_refs` | Ordered list of `PlanningAnalysisSourceRef` | No | Each ref points to the artifact, field, or governed document that made the finding visible. |

Initial-slice finding classes:

| Code | Source | Blocking | Meaning |
|---|---|---|---|
| `success_criterion_uncovered` | `goal` or `plan` | Yes | A required success criterion is not covered by the plan or governed backlog evidence. |
| `validation_coverage_missing` | `validation` | Yes | A required outcome lacks a matching validation strategy or acceptance anchor. |
| `artifact_contradiction` | `plan`, `backlog`, `risk`, or `constraint` | Yes | Typed planning artifacts disagree on an execution-critical point such as order, scope, or required input. |
| `execution_input_missing` | `execution_readiness` | Yes | Execution depends on a required input or handoff signal that is not present. |
| `producer_contract_gap` | `governed_evidence` | Yes | Canon-owned evidence needed for execution readiness is absent and cannot be safely inferred by Boundline. |
| `expected_outcome_missing` | `plan` | No | A planned task lacks an explicit expected outcome or success trace even though the surrounding plan remains executable. |
| `coverage_signal_partial` | `validation`, `risk`, or `constraint` | No | Coverage is incomplete but not critical enough to block the initial execution handoff. |

## Entity: Planning Analysis Source Ref

Represents one source-attribution pointer attached to a planning-analysis
finding.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `artifact_kind` | Stable label such as `goal_plan`, `verification_strategy`, `backlog_document`, `execution_handoff`, or `canon_contract` | Yes | The label names the artifact family without relying on prose-only explanation. |
| `artifact_ref` | Relative artifact identifier | Yes | Use a stable task id, packet document name, session field path, or governed document filename. |
| `anchor` | Optional stable sub-reference | No | May name a task id, slice id, success criterion label, or packet section when available. |

Source refs keep the projection explainable in status, inspect, and assistant
surfaces without forcing operators to diff raw packet files.

## Entity: Planning Analysis Coverage Summary

Represents the additive execution-readiness coverage metrics shown when the
analysis runs successfully.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `success_criteria_total` | Integer | Yes | Count of active success criteria or required outcomes visible to the gate. |
| `success_criteria_covered` | Integer | Yes | Count covered by the current plan and governed backlog evidence. |
| `backlog_slice_total` | Integer or `null` | No | Present when Canon backlog evidence includes stable delivery slices. |
| `backlog_slice_covered` | Integer or `null` | No | Count of slices with enough execution-ready evidence for this gate. |
| `validation_anchor_total` | Integer or `null` | No | Present when acceptance or validation anchors are visible. |
| `validation_anchor_covered` | Integer or `null` | No | Count aligned with required outcomes. |
| `risk_total` | Integer or `null` | No | Present when planning-risk evidence is available. |
| `risk_covered` | Integer or `null` | No | Count of risks with matching mitigation or validation coverage. |
| `constraint_total` | Integer or `null` | No | Present when explicit constraints are captured in typed planning state. |
| `constraint_covered` | Integer or `null` | No | Count of constraints reflected in execution-ready planning artifacts. |
| `governed_evidence_ready` | Boolean | Yes | `true` only when required Canon-owned evidence for execution readiness is present. |

The first slice uses deterministic counts and booleans derived from currently
typed or governed inputs. It does not attempt fuzzy semantic scoring.

## Entity: Producer Contract Gap

Represents the specific blocked condition where Canon-owned execution evidence
is required but absent from the active governed packet.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `required_artifact` | Stable identifier | Yes | Names the missing Canon-authored field or document. |
| `consumer_reason` | Stable message | Yes | Explains why Boundline cannot safely continue without that evidence. |
| `fallback_allowed` | Boolean | Yes | Always `false` in this slice. Boundline must not invent Canon data. |

Producer contract gaps are reported as standard planning-analysis findings with
`code = producer_contract_gap`, but they remain a distinct business concept in
the domain model because the repair action lives upstream in Canon.

## State Transitions

```text
goal quality unresolved
  -> preserve earlier gate

goal quality ready + plan quality ready + backlog quality ready
  -> evaluate planning analysis

no coherence finding
  -> clean
  -> execution handoff may be offered

non-blocking coherence finding
  -> findings
  -> execution handoff may still be offered
  -> status and assistant surfaces must preserve the warning

critical coherence finding or producer contract gap
  -> blocked
  -> execution handoff withheld
  -> route back to planning continuation / phase request path

older snapshot without planning analysis
  -> fields omitted
  -> no synthetic blocked or clean state introduced
```

## Compatibility Rules

- Older session snapshots without `planning_analysis` must deserialize and
  render successfully.
- Session, status, inspect, and assistant consumers that ignore additive
  planning-analysis fields must continue to work.
- The effective analysis must be recomputed from the current plan and active
  governed backlog evidence before execution-admission decisions.
- Plan quality and backlog quality remain separate earlier gates; planning
  analysis consumes their ready state rather than replacing them.
