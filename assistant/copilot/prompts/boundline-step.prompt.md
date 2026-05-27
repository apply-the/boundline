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
If the workspace is known, run `boundline step --workspace <workspace> --json` exactly once. If the workspace or active session is missing, ask only for the missing context or route to `/boundline-goal` or `/boundline-plan`.

## Chat-Only Path
If shell execution is unavailable, ask only for missing workspace details and then provide this exact copyable command:

`boundline step --workspace <workspace> --json`

Wait for pasted output before continuing.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Summarize `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Surface exactly two action links: one **primary** (advance) and one **secondary** (refine/inspect, shown only when the condition is met).

**Primary** (always shown): render the CLI-reported `next_command`; default to this clickable link when the runtime points back to step:
[▶ Run /boundline-step](command:github.copilot.chat.execute?%5B%22%2Fboundline-step%22%5D)

**Secondary** (shown only when the step encountered a stop condition or progress review is needed): render this clickable link:
[▶ Run /boundline-status](command:github.copilot.chat.execute?%5B%22%2Fboundline-status%22%5D)

If the secondary condition is not met, show only the primary button.
Before the action links, include one brief natural-language sentence summarizing why these actions are offered.
Prefer an emitted `phase_request.assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Render whichever assistant-safe route wins using `command:github.copilot.chat.execute`.

Allowed follow-up commands: `/boundline-step`, `/boundline-status`, `/boundline-next`, `/boundline-inspect`, `/boundline-plan`, `/boundline-goal`.
