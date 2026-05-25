---
description: "Advance a Boundline workflow by choosing one explicit next action"
---

# Command: /boundline-step

Shared guidance: `assistant/README.md`

## Intent
Advance the active Boundline session by executing exactly one planned step.

## Required Context
- `workspace_ref`
- Captured goal or active session state; preserve confirmed context instead of asking for it again

## Shell-Enabled Path
If the workspace is known, run `cargo run --bin boundline -- step --workspace <workspace> --json` exactly once. If the workspace or active session is missing, ask only for the missing context or route to `/boundline-goal` or `/boundline-plan`.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace details and then provide this exact copyable command:

`cargo run --bin boundline -- step --workspace <workspace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): `/boundline-step` — continue stepping.
**Secondary** (shown only when the step encountered a stop condition or progress review is needed): `/boundline-status` — review session status.

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present — it overrides the primary.

Allowed follow-up commands: `/boundline-step`, `/boundline-status`, `/boundline-next`, `/boundline-inspect`, `/boundline-plan`, `/boundline-goal`.