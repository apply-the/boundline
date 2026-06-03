# Data Model: Plan Quality Contract

## Entity: Plan Quality Assessment

Represents the effective readiness decision for one active goal-derived plan.

| Field | Shape | Required | Rules |
|---|---|---|---|
| `state` | `ready`, `clarification_required`, or `blocked` | Yes | `ready` only when no finding blocks handoff; `clarification_required` for recoverable missing plan input; `blocked` for non-credible context or another explicit non-recoverable condition. |
| `findings` | Ordered list of stable labels | No | Labels remain concise and machine-readable. Ordering determines the single question asked first. |
| `assumptions` | Ordered list of accepted defaults | No | Assumptions remain visible but do not block handoff by themselves. |

The assessment is persisted additively inside the existing goal-plan record and
is recomputed from current plan contents before admission or projection.

## Entity: Plan Quality Finding

Represents one missing or weak planning input.

| Label | Initial-slice behavior | Recovery |
|---|---|---|
| `verification_strategy` | Recoverable clarification; blocks execution handoff | Ask for an explicit validation strategy, then re-evaluate. |
| `planning_rationale` | Existing recoverable clarification retained for compatibility | Ask for the rationale supporting the selected plan targets, then re-evaluate. |
| `context_pack_insufficient` | Explicit blocked state | Surface the context summary and require new credible input. |
| `context_pack_stale` | Explicit blocked state | Surface the staleness reason and require context refresh. |

The first release slice newly formalizes the validation-strategy gate. Existing
rationale and context findings remain visible so the feature does not regress
already-landed scaffolding.

## Entity: Plan Quality Assumption

Represents a low-impact inferred default accepted by the runtime.

| Initial assumption | Meaning |
|---|---|
| `no explicit route override is required for this plan` | The operator did not request a route override, so the normal configured routing policy remains valid. |

Assumptions must remain inspectable in status, orchestration snapshots, and
trace-backed reasoning without becoming implicit control flow.

## Entity: Planning Clarification Request

Reuses the existing structured `phase_request` handoff.

| Field | Purpose |
|---|---|
| `request_id` | Stable resume identity for the active question. |
| `kind` | Clarification classification understood by assistant hosts. |
| `reason` | Concise explanation of why planning cannot advance. |
| `question` | Exactly one operator question for the current highest-priority finding. |
| `expected_answer` | Existing answer contract used by host surfaces. |
| `resume_command` | Raw CLI continuation that resumes the same session. |
| `assistant_resume_command` | Host-safe assistant continuation when available. |

## State Transitions

```text
goal quality unresolved
  -> preserve goal-quality gate

goal quality ready + plan available
  -> evaluate plan quality

no blocking finding
  -> ready
  -> execution handoff may be offered

recoverable missing plan input
  -> clarification_required
  -> emit one phase_request
  -> wait for operator input
  -> re-evaluate same session

non-credible context
  -> blocked
  -> surface reason and stop pending explicit operator action
```

## Compatibility Rules

- Older session snapshots without `plan_quality` must deserialize successfully.
- Consumers that ignore new session-status fields must continue to work.
- The effective assessment is recomputed before admission decisions so stale
  persisted state cannot bypass the gate.
- Backlog quality and planning analysis remain separate later gates; they are
  not folded into the plan-quality assessment.
