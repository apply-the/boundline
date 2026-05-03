# Contract: Session Command Surface

## Purpose

Defines the required behavior, inputs, and user-visible outputs of the session-backed Boundline CLI commands introduced by the session model feature.

## Command Set

| Command | Purpose |
|---------|---------|
| `boundline start` | Establish a new active session for the current workspace |
| `boundline capture` | Store or replace the current bounded goal in the active session |
| `boundline plan` | Create an executable plan from the active session goal |
| `boundline step` | Execute exactly one next step from the active session |
| `boundline run` | Continue execution until the task reaches a terminal state |
| `boundline status` | Summarize the active session state |
| `boundline next` | Recommend exactly one next valid command for the active session |

## Required Behavioral Rules

- Every session-backed command MUST resolve the active session automatically from the current workspace or an explicit workspace override when one is provided.
- `start` MUST create or replace the active session only through an explicit user action.
- `capture` MUST fail clearly when no active session exists.
- `plan` MUST fail clearly when the active session has no goal.
- `step` MUST execute at most one executable step and MUST persist updated session and trace state before returning.
- `run` MUST reuse the current active task snapshot when one exists; otherwise it may initialize execution from the active goal and a freshly created plan.
- `status` MUST surface goal, current execution position when available, latest status, and latest trace reference.
- `next` MUST return exactly one valid follow-up command and a short explanation.

## Required Non-Success Handling

| Situation | Required Result |
|-----------|-----------------|
| No active session | Explicit message telling the user to run `boundline start` |
| Session exists without goal | Explicit message telling the user to use `boundline capture` |
| Session exists without plan | Explicit message telling the user to use `boundline plan` |
| Session is corrupted or unreadable | Explicit recovery message and no hidden fallback |
| Latest trace reference is missing | Status or next output must surface the mismatch and guide the user to recover deliberately |
| Terminal session receives more execution commands | Command must fail clearly or require explicit reset rather than silently continuing |

## Output Guarantees

- `status` and `next` outputs MUST be readable without opening raw JSON files.
- Any command that updates the latest trace MUST surface the new trace reference.
- Any command that reaches a terminal outcome MUST surface the terminal reason.
- Any command that preserves a non-success recovery state MUST make the next recommended action explicit.