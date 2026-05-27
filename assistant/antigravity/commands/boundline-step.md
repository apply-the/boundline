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
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
Summarize `routing`, `execution_condition`, `latest_status`, any updated `latest_trace_ref`, and the CLI-reported `next_command`.

## Next-Step Routing
Surface host-native action links using `/boundline:*` command ids.
Prefer `assistant_resume_command` when present; otherwise prefer `assistant_next_command`; otherwise follow the CLI-reported `next_command`. Default to `/boundline:step` for continued stepping; route to `/boundline:status` only when the step hit a stop condition or progress review is needed. If the session is missing or invalid, route to `/boundline:goal`.
Allowed follow-up commands: `/boundline:step`, `/boundline:status`, `/boundline:continue`, `/boundline:inspect`, `/boundline:plan`, `/boundline:goal`.
