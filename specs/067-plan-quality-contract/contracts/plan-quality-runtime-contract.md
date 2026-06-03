# Contract: Plan Quality Runtime Projection

## Purpose

Define the additive Boundline-owned planning-readiness contract used by CLI
status, orchestration snapshots, execution admission, traces, and supported
assistant planning assets.

## Evaluation Order

```text
goal quality
  -> plan quality
  -> backlog quality
  -> planning analysis
  -> execution handoff
```

Each gate must stop the sequence when it is not ready. The runtime asks one
question at a time and must not expose execution handoff while plan quality is
`clarification_required` or `blocked`.

## Additive Session Projection

When a plan exists or planning is blocked, session status and orchestration
snapshots may include:

```json
{
  "plan_quality_state": "clarification_required",
  "plan_quality_findings": [
    "verification_strategy"
  ],
  "plan_quality_assumptions": [
    "no explicit route override is required for this plan"
  ]
}
```

Allowed `plan_quality_state` values:

| Value | Meaning | Execution handoff |
|---|---|---|
| `ready` | No blocking plan-quality finding remains. | May continue to later gates. |
| `clarification_required` | Focused operator input can resolve the current plan defect. | Withheld. |
| `blocked` | Planning context is explicitly non-credible or otherwise blocked. | Withheld. |

The fields are additive. Older snapshots may omit them, and existing consumers
may ignore them.

## Clarification Handoff

For a recoverable plan-quality finding, the runtime must:

1. Keep planning non-terminal.
2. Persist the effective assessment in the active session.
3. Mark the session blocked pending operator input.
4. Emit one existing structured `phase_request`.
5. Preserve the existing raw and assistant-safe resume routes.
6. Re-evaluate the same session after the answer is applied.

The initial release formalizes a missing `verification_strategy` as the newly
enforced blocking finding. Additional semantic-strength checks remain separate
future slices so this first gate stays bounded.

## Trace Expectations

Trace-visible planning decisions must preserve:

- session identity
- effective plan-quality state
- ordered findings
- accepted assumptions
- emitted clarification request identity
- withheld execution handoff
- recovered readiness after explicit operator input

Traces must not log secrets, tokens, or personally identifiable information.

## Assistant Asset Contract

Copilot, Claude, Codex, and Antigravity planning assets must preserve:

- `goal_quality_state`
- `plan_quality_state`
- `plan_quality_findings`
- `plan_quality_assumptions`
- emitted `phase_request`
- raw `resume_command`
- `assistant_resume_command`
- `assistant_next_command`

Each planning asset must contain these standardized sections:

- `User Input`
- `Pre-Execution Checks`
- `Execution Flow`
- `Plan Quality Validation`
- `Reasonable Defaults`
- `Gate Handling`
- `Output Interpretation`
- `Next-Step Routing`
- `Done When`

Assistant hosts must not synthesize an execution route from chat-only
assumptions when goal quality or plan quality is blocked.

## Explicit Non-Goals

- No new CLI command.
- No file-first Speckit runtime.
- No backlog-quality implementation.
- No cross-artifact planning-analysis implementation.
- No Canon-owned execution admission.
- No provider, sandbox, browser, gateway, memory, council, adaptive-governance,
  recursive-refinement, concurrency, or background-worker behavior.
