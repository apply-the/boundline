# Command: /boundline-recover

Shared guidance: `assistant/README.md`

## Install Boundary
Before using the workspace path in a fresh environment, prefer `boundline doctor --install` and the README quick path.
Boundline owns orchestration; Canon is only the optional governed companion.

## Intent
Recover from a Boundline session that is blocked, clarification-required, failed, exhausted, or terminal by asking the real runtime for current state first.

## Required Context
- `workspace_ref`

## Shell-Enabled Path
Run `boundline status --workspace <workspace> --json` exactly once. If the output reports a `latest_checkpoint_restore_command`, `corrected_command`, or `next_command`, use that command as the recovery path. If status is insufficient, run `boundline inspect --workspace <workspace> --json` exactly once and preserve its guidance.

## Chat-Only Path
If shell execution is unavailable, provide this exact copyable command:

`boundline status --workspace <workspace> --json`

Wait for pasted output before recommending recovery.

## Output Interpretation
Provide a conversational, human-readable summary of the session state. Do NOT use raw JSON keys or snake_case field names (like `next_command`, `latest_status`, `authored_input_summary`, etc.) in your response. Translate all state into natural language.
When suggesting the next step, you MUST output a VS Code Copilot command link to render a clickable button. Use EXACTLY this syntax format:
`[Run /boundline-plan](command:github.copilot.chat.execute?%5B%22%2Fboundline-plan%22%5D)` (replace /boundline-plan with the actual command). Do not use plain text or unicode arrows.
Reply as a compact operator brief by default: preserve `execution_condition` when status or inspect reports it, recovery blockers or checkpoint restore guidance, `latest_status`, and the CLI-reported `next_command`. Only surface raw status or inspect dumps when the user explicitly asks for deeper detail or wants the CLI `--verbose` view. `.boundline/session.json` remains authoritative, and recovery must not be inferred from chat history. Preserve blocked, clarification-required, failed, exhausted, or terminal wording exactly.

## Next-Step Routing
Prefer the CLI-reported `next_command`, `latest_checkpoint_restore_command`, or `corrected_command`. Route to `/boundline-inspect` only when status says more evidence is needed, and route to `/boundline-goal` only when the runtime reports no usable active session.
Allowed follow-up commands: `/boundline-status`, `/boundline-inspect`, `/boundline-next`, `/boundline-goal`.
