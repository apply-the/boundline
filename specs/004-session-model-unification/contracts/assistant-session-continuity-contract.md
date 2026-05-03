# Contract: Assistant Session Continuity

## Purpose

Defines how assistant command packs must reuse and respect the active Boundline session introduced by the unified session model.

## Required Assistant Behavior

- Assistant commands MUST prefer the active Boundline session over asking the user to restate goal, workspace, or latest trace information that Boundline already preserves.
- Assistant commands MUST route through session-backed CLI commands when they need current status or next-step guidance.
- Assistant commands MAY still use explicit trace inspection when the user asks about a specific historical run instead of the active session.

## Command Alignment Rules

| Assistant Command | Preferred Session-Backed CLI Surface |
|-------------------|--------------------------------------|
| `/boundline-start` | `boundline start` |
| `/boundline-plan` | `boundline capture` followed by `boundline plan` when a goal must be established first |
| `/boundline-step` | `boundline step` |
| `/boundline-run` | `boundline run` |
| `/boundline-status` | `boundline status` |
| `/boundline-next` | `boundline next` |
| `/boundline-inspect` | `boundline inspect` |

## Continuity Guarantees

- If the active session contains a valid goal, assistant commands MUST NOT ask for that goal again unless the user explicitly changes it.
- If the active session contains a latest trace reference, assistant commands MUST reuse it before requesting manual trace lookup.
- If the active session is invalid or missing, assistant commands MUST say so explicitly and route the user to `boundline start` or another concrete recovery action.
- Assistant commands MUST preserve the rule that exactly one next command is recommended at a time.

## Non-Success Handling

- When the session is corrupted, stale, or workspace-mismatched, assistant commands MUST surface the session problem rather than inventing context.
- When execution reaches a terminal state, assistant commands MUST treat the session as complete and route the user to inspect, restart, or replace the goal explicitly.
- When the user refers to an explicit prior trace, assistant continuity MUST not overwrite the active session silently.