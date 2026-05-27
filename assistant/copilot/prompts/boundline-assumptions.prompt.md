---
description: "Inspect the current Boundline assumptions with explicit grouping and risk"
---

# Command: /boundline-assumptions

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Inspect the current plan and explain the assumptions currently influencing it.
Group assumptions by Boundline's reported category and keep source, status, and risk visible.

## Required Context
- `workspace_ref` or `trace_ref`
- Preserve any confirmed `latest_trace_ref` from prior turns

## Shell-Enabled Path
If `trace_ref` is known, run `boundline inspect --trace <trace> --json`.
Otherwise, if `workspace_ref` is known, run `boundline inspect --workspace <workspace> --json`.
If the assistant is already anchored in the workspace, run `boundline inspect --json` exactly once.

## Chat-Only Path
Ask only for the missing `workspace_ref` or `trace_ref`, then provide one exact copyable command:

`boundline inspect --workspace <workspace> --json`

or

`boundline inspect --trace <trace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Summarize `assumptions_summary` first, then group each `assumption_group` by category. Preserve `source_attribution`, `fallback_disclosure`, `challenge_weakest_assumption`, and `next_command` verbatim when they appear. If Canon-governed input is missing or stale, say so plainly instead of promoting inferred agreement.

## Next-Step Routing
Prefer the CLI-reported `next_command`. If inspect reports a session error, route to `/boundline-goal`.
Allowed follow-up commands: `/boundline-hidden-impact`, `/boundline-challenge`, `/boundline-explain-plan`, `/boundline-risk`, `/boundline-inspect`, `/boundline-status`, `/boundline-goal`.
