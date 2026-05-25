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
Prefer the CLI-reported `next_command`; if the session is missing or invalid, route to `/boundline-goal`.
Allowed follow-up commands: `/boundline-step`, `/boundline-status`, `/boundline-next`, `/boundline-inspect`, `/boundline-plan`, `/boundline-goal`.