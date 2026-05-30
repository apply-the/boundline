# Command: /boundline-govern

Shared guidance: `assistant/README.md`

## Intent
Use optional Canon governance through Boundline's governed stage surface.

## Required Context
- `workspace_ref`
- Desired governed mode only if the user already provided one
- Goal, brief, base/head, or other authored input when the selected mode needs it

## Shell-Enabled Path
Run `boundline govern --workspace <workspace> --mode <mode> --json` when the user supplied a mode. If the user did not supply a mode, run `boundline govern --workspace <workspace> --json` and present the CLI mode choices. Do not promote per-mode `/boundline:<mode>` aliases as the primary UX.

## Chat-Only Path
If shell execution is unavailable, provide the exact copyable command that matches the available input:

`boundline govern --workspace <workspace> --json`

Add `--mode <mode>` only when the user already named one.

Wait for pasted output before discussing governance.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
For the next step or follow-up commands, provide them as clickable buttons or action links (e.g., Markdown command links) instead of plain text recommendations.
`.boundline/session.json` remains authoritative for delivery state. Canon produces governed packets while Boundline owns orchestration. Preserve governed stage refs, packet refs, approval or missing-input state, `next_command`, and blocked, clarification-required, failed, exhausted, or terminal state before invoking follow-up actions.
