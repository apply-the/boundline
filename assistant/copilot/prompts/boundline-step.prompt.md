---
description: "Advance a Boundline workflow by choosing one explicit next action"
handoffs:
  - label: Next Action
    agent: boundline-next
    prompt: Recommend the next bounded action
    send: true
  - label: Check Status
    agent: boundline-status
    prompt: Show current session status
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
Summarize `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Prefer `assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`.
Render assistant-safe follow-up actions as Copilot command links or the defined handoff buttons instead of plain text shell guidance when a host route is available. For example, use `[Run /boundline-step](command:github.copilot.chat.execute?%5B%22%2Fboundline-step%22%5D)` when the current route stays on step.
Default to `/boundline-step` for continued stepping. Route to `/boundline-status` when the step hits a stop condition or the user asks for a progress snapshot. Route to `/boundline-plan` or `/boundline-goal` when the session is missing required planning or goal context.
Allowed follow-up commands: `/boundline-step`, `/boundline-status`, `/boundline-next`, `/boundline-inspect`, `/boundline-plan`, `/boundline-goal`.
