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
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
`.boundline/session.json` remains authoritative for delivery state. Canon produces governed packets while Boundline owns orchestration. Preserve governed stage refs, packet refs, approval or missing-input state, `next_command`, and blocked, clarification-required, failed, exhausted, or terminal state before invoking follow-up actions.
