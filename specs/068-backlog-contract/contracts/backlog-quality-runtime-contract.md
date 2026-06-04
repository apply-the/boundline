# Contract: Backlog Quality Runtime Projection

## Purpose

Define the additive Boundline-owned backlog-readiness contract used by session
status, orchestration snapshots, traces, execution admission, and supported
assistant plan or run assets.

## Evaluation Order

```text
goal quality
  -> plan quality
  -> backlog quality
  -> planning analysis
  -> execution handoff
```

Each gate stops the sequence when it is not ready. The runtime asks one
question at a time and must not expose execution handoff while backlog quality
is `clarification_required` or `blocked`.

## Canon `0.67.0` Packet Expectations

Boundline consumes Canon backlog output as governed source material. For this
slice it may inspect only evidence already emitted by Canon `0.67.0`:

- `backlog-overview.md`
- `epic-tree.md`
- `capability-to-epic-map.md`
- `dependency-map.md`
- `delivery-slices.md`
- `sequencing-plan.md`
- `acceptance-anchors.md`
- `planning-risks.md`
- optional `execution-handoff.md`

If Canon emits only the closure-limited risk packet (`backlog-overview.md` plus
`planning-risks.md`), Boundline must treat backlog quality as `blocked`.

## Additive Session Projection

When a backlog packet is expected or available, session status and
orchestration snapshots may include:

```json
{
  "backlog_quality_state": "clarification_required",
  "backlog_quality_findings": [
    "missing_execution_handoff",
    "missing_independent_verification_anchors"
  ],
  "backlog_task_count": 3,
  "backlog_mvp_scope": "SLICE-AUTH-001",
  "backlog_unmapped_items": [
    "acceptance target"
  ]
}
```

Allowed `backlog_quality_state` values:

| Value | Meaning | Execution handoff |
|---|---|---|
| `ready` | The packet is credible enough for downstream execution admission. | May continue to planning analysis and later handoff. |
| `clarification_required` | Focused operator input or regenerated Canon output can resolve the current omission. | Withheld. |
| `blocked` | The packet is unsafe to interpret or Canon emitted only a closure-limited packet. | Withheld. |

The fields are additive. Older snapshots may omit them, and existing consumers
may ignore them.

## Clarification Handoff

For a recoverable backlog-quality finding, the runtime must:

1. Keep planning non-terminal.
2. Persist the effective assessment in the active session view.
3. Mark the session blocked pending operator input.
4. Emit one existing structured `phase_request`.
5. Preserve the existing raw and assistant-safe resume routes.
6. Re-evaluate the same session after the answer or regenerated Canon packet is
   applied.

Recoverable findings are limited to omissions in an otherwise credible full
packet. Closure-limited or contradictory packets do not use this path.

## Trace Expectations

Trace-visible backlog decisions must preserve:

- session identity
- effective backlog-quality state
- ordered findings
- derived task count
- derived MVP scope
- unmapped items
- emitted clarification request identity when present
- withheld execution handoff
- recovered readiness after explicit operator input or regenerated packet

Traces must not log secrets, tokens, or personally identifiable information.

## Assistant Asset Contract

Copilot, Claude, Codex, and Antigravity plan and run assets must preserve:

- `goal_quality_state`
- `plan_quality_state`
- `backlog_quality_state`
- `backlog_quality_findings`
- `backlog_task_count`
- `backlog_mvp_scope`
- `backlog_unmapped_items`
- emitted `phase_request`
- raw `resume_command`
- `assistant_resume_command`
- `assistant_next_command`

Assistant hosts must not synthesize a run route from chat-only assumptions when
backlog quality is not ready. They may only point the user to the planning
resume path or the emitted `phase_request` continuation.
